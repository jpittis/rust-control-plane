mod choose;
mod event;

use data_plane_api::envoy::config::cluster::v3::cluster::LbPolicy;
use indexmap::{IndexMap, IndexSet};

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Cluster {
    pub name: String,
    pub lb_policy: LbPolicy,
    pub endpoints: IndexSet<Endpoint>,
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Endpoint {
    pub port: u32,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Node {
    pub id: String,
    pub clusters: IndexMap<String, Cluster>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Fleet {
    pub nodes: IndexMap<String, Node>,
}

impl Fleet {
    fn new(node_ids: &[String]) -> Self {
        let mut nodes = IndexMap::new();
        for node_id in node_ids {
            nodes.insert(
                node_id.clone(),
                Node {
                    id: node_id.clone(),
                    clusters: IndexMap::new(),
                },
            );
        }
        Self { nodes }
    }
}
