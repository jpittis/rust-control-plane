pub mod type_url;

use data_plane_api::envoy::config::cluster::v3::Cluster;
use data_plane_api::envoy::config::endpoint::v3::ClusterLoadAssignment;
use data_plane_api::envoy::config::listener::v3::Listener;
use data_plane_api::envoy::config::route::v3::Route;
use data_plane_api::envoy::extensions::transport_sockets::tls::v3::Secret;
use data_plane_api::envoy::service::runtime::v3::Runtime;
use data_plane_api::google::protobuf::Any;
use prost::Message;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub resources: HashMap<String, Resources>,
}

impl Snapshot {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    pub fn insert(&mut self, type_url: String, resources: Resources) {
        self.resources.insert(type_url, resources);
    }

    pub fn version(&self, type_url: &str) -> &str {
        self.resources
            .get(type_url)
            .map_or("", |resource| &resource.version)
    }

    pub fn resources(&self, type_url: &str) -> Option<&Resources> {
        self.resources.get(type_url)
    }
}

#[derive(Clone, Debug)]
pub struct Resources {
    pub version: String,
    pub items: HashMap<String, Resource>,
}

impl Resources {
    pub fn new(version: String) -> Self {
        Self {
            version,
            items: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Resource {
    Cluster(Cluster),
    Endpoint(ClusterLoadAssignment),
    Route(Route),
    Listener(Listener),
    Secret(Secret),
    Runtime(Runtime),
}

impl Resource {
    pub fn into_any(&self) -> Any {
        match self {
            Resource::Cluster(cluster) => Any {
                type_url: type_url::CLUSTER.to_string(),
                value: cluster.encode_to_vec(),
            },
            Resource::Endpoint(endpoint) => Any {
                type_url: type_url::ENDPOINT.to_string(),
                value: endpoint.encode_to_vec(),
            },
            Resource::Route(route) => Any {
                type_url: type_url::ROUTE.to_string(),
                value: route.encode_to_vec(),
            },
            Resource::Listener(listener) => Any {
                type_url: type_url::LISTENER.to_string(),
                value: listener.encode_to_vec(),
            },
            Resource::Secret(secret) => Any {
                type_url: type_url::SECRET.to_string(),
                value: secret.encode_to_vec(),
            },
            Resource::Runtime(runtime) => Any {
                type_url: type_url::RUNTIME.to_string(),
                value: runtime.encode_to_vec(),
            },
        }
    }
}
