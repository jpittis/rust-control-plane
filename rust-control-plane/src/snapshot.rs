pub mod type_url;

use data_plane_api::envoy::config::cluster::v3::Cluster;
use data_plane_api::envoy::config::core::v3::TypedExtensionConfig;
use data_plane_api::envoy::config::endpoint::v3::ClusterLoadAssignment;
use data_plane_api::envoy::config::listener::v3::Listener;
use data_plane_api::envoy::config::route::v3::RouteConfiguration;
use data_plane_api::envoy::config::route::v3::ScopedRouteConfiguration;
use data_plane_api::envoy::extensions::transport_sockets::tls::v3::Secret;
use data_plane_api::envoy::service::runtime::v3::Runtime;
use data_plane_api::google::protobuf::Any;
use prost::Message;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub resources: HashMap<String, Resources>,
    pub version_map: Option<HashMap<String, HashMap<String, String>>>,
}

impl Default for Snapshot {
    fn default() -> Self {
        Self::new()
    }
}

impl Snapshot {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
            version_map: None,
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

    pub fn build_version_map(&mut self) {
        if self.version_map.is_none() {
            let mut map: HashMap<String, HashMap<String, String>> = HashMap::new();
            for (type_url, resources) in &self.resources {
                let entry = map.entry(type_url.clone()).or_insert_with(HashMap::new);
                for (name, resource) in &resources.items {
                    let hash = Sha256::digest(resource.encode_to_vec());
                    entry.insert(name.clone(), format!("{:x}", hash));
                }
            }
            self.version_map = Some(map);
        }
    }
}

pub fn hash_resource(resource: Resource) -> String {
    let hash = Sha256::digest(resource.encode_to_vec());
    format!("{:x}", hash)
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

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum Resource {
    Cluster(Cluster),
    Endpoint(ClusterLoadAssignment),
    Route(RouteConfiguration),
    Listener(Listener),
    Secret(Secret),
    Runtime(Runtime),
    ScopedRoute(ScopedRouteConfiguration),
    ExtensionConfig(TypedExtensionConfig),
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
            Resource::ScopedRoute(route) => Any {
                type_url: type_url::SCOPED_ROUTE.to_string(),
                value: route.encode_to_vec(),
            },
            Resource::ExtensionConfig(config) => Any {
                type_url: type_url::EXTENSION_CONFIG.to_string(),
                value: config.encode_to_vec(),
            },
        }
    }

    fn encode_to_vec(&self) -> Vec<u8> {
        match self {
            Resource::Cluster(cluster) => cluster.encode_to_vec(),
            Resource::Endpoint(endpoint) => endpoint.encode_to_vec(),
            Resource::Route(route) => route.encode_to_vec(),
            Resource::Listener(listener) => listener.encode_to_vec(),
            Resource::Secret(secret) => secret.encode_to_vec(),
            Resource::Runtime(runtime) => runtime.encode_to_vec(),
            Resource::ScopedRoute(route) => route.encode_to_vec(),
            Resource::ExtensionConfig(config) => config.encode_to_vec(),
        }
    }
}
