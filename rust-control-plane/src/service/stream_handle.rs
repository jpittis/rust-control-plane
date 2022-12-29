use data_plane_api::envoy::service::discovery::v3::DeltaDiscoveryRequest;
use std::collections::{HashMap, HashSet};

pub struct StreamHandle {
    known_resource_names: HashMap<String, HashSet<String>>,
}

impl StreamHandle {
    pub fn new() -> Self {
        Self {
            known_resource_names: HashMap::new(),
        }
    }

    pub fn known_resource_names(&self, type_url: &str) -> Option<&HashSet<String>> {
        self.known_resource_names.get(type_url)
    }

    pub fn add_known_resource_names(&mut self, type_url: &str, names: &[String]) {
        self.known_resource_names
            .entry(type_url.to_string())
            .and_modify(|entry| {
                names.iter().for_each(|name| {
                    entry.insert(name.clone());
                })
            })
            .or_insert_with(|| {
                let mut entry = HashSet::new();
                names.iter().for_each(|name| {
                    entry.insert(name.clone());
                });
                entry
            });
    }
}

impl Default for StreamHandle {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct DeltaStreamHandle {
    wildcard: bool,
    subscribed_resource_names: HashSet<String>,
    resource_versions: HashMap<String, String>,
    first: bool,
}

impl DeltaStreamHandle {
    pub fn new(req: &DeltaDiscoveryRequest) -> Self {
        Self {
            wildcard: req.resource_names_subscribe.is_empty()
                && req.resource_names_unsubscribe.is_empty(),
            subscribed_resource_names: HashSet::new(),
            resource_versions: req.initial_resource_versions.clone(),
            first: true,
        }
    }

    pub fn apply_subscriptions(&mut self, req: &DeltaDiscoveryRequest) {
        self.first = false;
        self.subscribe(&req.resource_names_subscribe);
        self.unsubscribe(&req.resource_names_unsubscribe);
    }

    pub fn subscribe(&mut self, resources: &[String]) {
        for name in resources {
            if name == "*" {
                self.wildcard = true;
                continue;
            }
            self.subscribed_resource_names.insert(name.clone());
        }
    }

    pub fn unsubscribe(&mut self, resources: &[String]) {
        for name in resources {
            if name == "*" {
                self.wildcard = false;
                continue;
            }
            if self.subscribed_resource_names.contains(name) && self.wildcard {
                self.resource_versions.insert(name.clone(), String::new());
            }
            self.subscribed_resource_names.remove(name);
        }
    }

    pub fn is_wildcard(&self) -> bool {
        self.wildcard
    }

    pub fn is_first(&self) -> bool {
        self.first
    }

    pub fn resource_versions(&self) -> &HashMap<String, String> {
        &self.resource_versions
    }

    pub fn subscribed_resource_names(&self) -> &HashSet<String> {
        &self.subscribed_resource_names
    }

    pub fn set_resource_versions(&mut self, versions: HashMap<String, String>) {
        self.resource_versions = versions
    }
}
