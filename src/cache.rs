use futures::sync::oneshot;
use futures::Future;
use std::collections::HashMap;

use rust_xds_grpc::cds::Cluster;
use rust_xds_grpc::eds::ClusterLoadAssignment;
use rust_xds_grpc::lds::Listener;
use rust_xds_grpc::rds::RouteConfiguration;
use rust_xds_grpc::sds::SdsDummy;

use protobuf;
use rust_xds_grpc::base::Node;
use rust_xds_grpc::discovery::{DiscoveryRequest, DiscoveryResponse};

trait Cache {
    fn fetch(&mut self, req: DiscoveryRequest) -> CacheResult<DiscoveryResponse>;
    fn watch(&mut self, req: DiscoveryRequest) -> oneshot::Receiver<DiscoveryResponse>;
}

struct NodeStatus {
    node: Node,
}

const typePrefix: &'static str = "type.googleapis.com/envoy.api.v2.";
const EndpointType: &'static str = concat!("type.googleapis.com/envoy.api.v2.", "ClusterLoadAssignment");
const ClusterType: &'static str = concat!("type.googleapis.com/envoy.api.v2.", "Cluster");
const RouteType: &'static str = concat!("type.googleapis.com/envoy.api.v2.", "RouteConfiguration");
const ListenerType: &'static str = concat!("type.googleapis.com/envoy.api.v2.", "Listener");
const SecretType: &'static str = concat!("type.googleapis.com/envoy.api.v2.", "auth.Secret");

struct Resource {
    msg: Box<protobuf::Message>,
}

type ResourceSet = HashMap<String, Resource>;

struct Resources {
    version: String,
    items: ResourceSet,
}

struct Snapshot {
    endpoints: Resources,
    clusters: Resources,
    routes: Resources,
    listeners: Resources,
    secrets: Resources,
}

impl Snapshot {
    fn get_version(&self, type_url: &str) -> &str {
        match type_url {
            EndpointType => &self.endpoints.version,
            ClusterType => &self.endpoints.version,
            RouteType => &self.endpoints.version,
            ListenerType => &self.endpoints.version,
            SecretType => &self.endpoints.version,
        }
    }

    fn get_resources(&self, type_url: &str) -> ResourceSet {
        match type_url {
            EndpointType => self.endpoints.items,
            ClusterType => self.endpoints.items,
            RouteType => self.endpoints.items,
            ListenerType => self.endpoints.items,
            SecretType => self.endpoints.items,
        }
    }
}

trait NodeHash {
    fn hash(&self, node: &Node) -> String;
}

struct SnapshotCache<H: NodeHash> {
    snapshots: HashMap<String, Snapshot>,
    node_status: HashMap<String, NodeStatus>,
    node_hash: H,
}

#[derive(Debug)]
pub enum CacheError {
    SkipFetch,
    MissingNodeHash,
}

pub type CacheResult<T> = Result<T, CacheError>;

impl<H: NodeHash> Cache for SnapshotCache<H> {
    fn fetch(&mut self, req: DiscoveryRequest) -> CacheResult<DiscoveryResponse> {
        let hash = self.node_hash.hash(req.get_node());

        match self.snapshots.get(&hash) {
            Some(snapshot) => {
                let version = snapshot.get_version(req.get_type_url());
                if req.get_version_info() == version {
                    return Err(CacheError::SkipFetch);
                }

                let resources = snapshot.get_resources(req.get_type_url());
                Ok(build_response(req, resources, version))
            }
            None => Err(CacheError::MissingNodeHash),
        }
    }

    fn watch(&mut self, req: DiscoveryRequest) -> oneshot::Receiver<DiscoveryResponse> {
        unimplemented!()
    }
}

fn build_response(
    req: DiscoveryRequest,
    resources: ResourceSet,
    version: &str,
) -> DiscoveryResponse {
    // TODO: If request has a resource_names then don't send all of them.
    let vec = resources.iter().map(|(_, resource)| resource.clone().msg).collect();
    let filtered = protobuf::RepeatedField::from_vec(vec);

    let mut resp = DiscoveryResponse::new();
    resp.set_version_info(version.to_string());
    resp.set_resources(filtered);
    resp
}
