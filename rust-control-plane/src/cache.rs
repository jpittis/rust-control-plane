use crate::snapshot::{Resources, Snapshot};
use data_plane_api::envoy::config::core::v3::Node;
use data_plane_api::envoy::service::discovery::v3::{DiscoveryRequest, DiscoveryResponse};
use log::info;
use slab::Slab;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::time::Instant;
use tokio::sync::oneshot;

#[derive(Debug)]
pub struct Cache {
    inner: Mutex<Inner>,
}

#[derive(Debug)]
struct Inner {
    status: HashMap<String, NodeStatus>,
    snapshots: HashMap<String, Snapshot>,
}

#[derive(Debug)]
struct NodeStatus {
    last_request_time: Instant,
    watches: Slab<Watch>,
}

impl NodeStatus {
    fn new() -> Self {
        Self {
            last_request_time: Instant::now(),
            watches: Slab::new(),
        }
    }
}

#[derive(Debug)]
struct Watch {
    req: DiscoveryRequest,
    tx: oneshot::Sender<(DiscoveryRequest, DiscoveryResponse)>,
}

pub enum FetchError {
    VersionUpToDate,
    NotFound,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Inner::new()),
        }
    }

    // Either responds on tx immediately, or sets a watch, returning a watch ID.
    pub fn create_watch(
        &self,
        req: &DiscoveryRequest,
        tx: oneshot::Sender<(DiscoveryRequest, DiscoveryResponse)>,
        known_resource_names: &HashMap<String, HashSet<String>>,
    ) -> Option<usize> {
        let mut inner = self.inner.lock().unwrap();
        let node_id = hash_id(&req.node);
        inner.update_node_status(&node_id);
        if let Some(snapshot) = inner.snapshots.get(&node_id) {
            let resources = snapshot.resources(&req.type_url);
            let version = snapshot.version(&req.type_url);
            let type_known_resource_names = known_resource_names.get(&req.type_url);
            // Check if a different set of resources has been requested.
            if inner.is_requesting_new_resources(&req, resources, type_known_resource_names) {
                info!("responding: resource diff");
                respond(req, tx, resources, &version);
                return None;
            }
            if req.version_info == version {
                // Client is already at the latest version, so we have nothing to respond with.
                // Set a watch because we may receive a new version in the future.
                info!("set watch: latest version");
                Some(inner.set_watch(&node_id, req, tx))
            } else {
                // The version has changed, so we should respond.
                info!("responding: new version");
                respond(req, tx, resources, &version);
                None
            }
        } else {
            // No snapshot exists for this node, so we have nothing to respond with.
            // Set a watch because we may receive a snapshot for this node in the future.
            info!("set watch: no snapshot");
            Some(inner.set_watch(&node_id, req, tx))
        }
    }

    // Deletes a watch previously created with create_watch.
    pub fn cancel_watch(&mut self, node: &str, watch_id: usize) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(status) = inner.status.get_mut(node) {
            status.watches.remove(watch_id);
        }
    }

    // Updates snapshot associated with a given node so that future requests receive it.
    // Triggers existing watches for the given node.
    pub fn set_snapshot(&self, node: &str, snapshot: Snapshot) {
        let mut inner = self.inner.lock().unwrap();
        inner.snapshots.insert(node.to_string(), snapshot.clone());
        if let Some(status) = inner.status.get_mut(node) {
            let mut to_delete = Vec::new();
            for (watch_id, watch) in &mut status.watches {
                let version = snapshot.version(&watch.req.type_url);
                if version != watch.req.version_info {
                    to_delete.push(watch_id)
                }
            }

            for watch_id in to_delete {
                let watch = status.watches.remove(watch_id);
                let resources = snapshot.resources(&watch.req.type_url);
                let version = snapshot.version(&watch.req.type_url);
                info!("watch triggered version={}", version);
                respond(&watch.req, watch.tx, resources, &version);
            }
        }
    }

    pub fn fetch(
        &self,
        req: &DiscoveryRequest,
        type_url: &'static str,
    ) -> Result<DiscoveryResponse, FetchError> {
        let inner = self.inner.lock().unwrap();
        let node_id = hash_id(&req.node);
        let snapshot = inner.snapshots.get(&node_id).ok_or(FetchError::NotFound)?;
        let version = snapshot.version(&req.type_url);
        if req.version_info == version {
            return Err(FetchError::VersionUpToDate);
        }
        let resources = snapshot.resources(&type_url);
        return Ok(build_response(req, resources, version));
    }

    pub fn node_status(&self) -> HashMap<String, Instant> {
        let inner = self.inner.lock().unwrap();
        inner
            .status
            .iter()
            .map(|(k, v)| (k.clone(), v.last_request_time))
            .collect()
    }
}

impl Inner {
    fn new() -> Self {
        Self {
            status: HashMap::new(),
            snapshots: HashMap::new(),
        }
    }

    fn set_watch(
        &mut self,
        node_id: &str,
        req: &DiscoveryRequest,
        tx: oneshot::Sender<(DiscoveryRequest, DiscoveryResponse)>,
    ) -> usize {
        let watch = Watch {
            req: req.clone(),
            tx,
        };
        let status = self.status.get_mut(node_id).unwrap();
        status.watches.insert(watch)
    }

    fn update_node_status(&mut self, node_id: &str) {
        self.status
            .entry(node_id.to_string())
            .and_modify(|entry| entry.last_request_time = Instant::now())
            .or_insert_with(|| NodeStatus::new());
    }

    fn is_requesting_new_resources(
        &self,
        req: &DiscoveryRequest,
        resources: Option<&Resources>,
        type_known_resource_names: Option<&HashSet<String>>,
    ) -> bool {
        if let Some(resources) = resources {
            if let Some(known_resource_names) = type_known_resource_names {
                let mut diff = Vec::new();
                for name in &req.resource_names {
                    if known_resource_names.contains(name) {
                        diff.push(name)
                    }
                }
                for name in diff {
                    if resources.items.contains_key(name) {
                        return true;
                    }
                }
            }
        }
        false
    }
}

fn hash_id(node: &Option<Node>) -> String {
    node.as_ref().map_or(String::new(), |node| node.id.clone())
}

fn build_response(
    req: &DiscoveryRequest,
    resources: Option<&Resources>,
    version: &str,
) -> DiscoveryResponse {
    let mut filtered_resources = Vec::new();
    if let Some(resources) = resources {
        if req.resource_names.is_empty() {
            filtered_resources = resources
                .items
                .values()
                .map(|resource| resource.into_any())
                .collect();
        } else {
            for name in &req.resource_names {
                if let Some(resource) = resources.items.get(name) {
                    filtered_resources.push(resource.into_any())
                }
            }
        }
    }
    DiscoveryResponse {
        type_url: req.type_url.clone(),
        nonce: String::new(),
        version_info: version.to_string(),
        resources: filtered_resources,
        control_plane: None,
        canary: false,
    }
}

fn respond(
    req: &DiscoveryRequest,
    tx: oneshot::Sender<(DiscoveryRequest, DiscoveryResponse)>,
    resources: Option<&Resources>,
    version: &str,
) {
    let rep = build_response(req, resources, version);
    tx.send((req.clone(), rep)).unwrap();
}
