mod choose;
mod event;
mod generate;
mod to_proto;

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

impl Default for Fleet {
    fn default() -> Self {
        Self {
            nodes: IndexMap::from_iter(vec![
                (
                    "node1".to_string(),
                    Node {
                        id: "node1".to_string(),
                        clusters: IndexMap::from_iter(vec![
                            (
                                "cluster1".to_string(),
                                Cluster {
                                    name: "cluster1".to_string(),
                                    lb_policy: LbPolicy::LeastRequest,
                                    endpoints: IndexSet::from_iter(vec![
                                        Endpoint { port: 2222 },
                                        Endpoint { port: 3333 },
                                    ]),
                                },
                            ),
                            (
                                "cluster2".to_string(),
                                Cluster {
                                    name: "cluster2".to_string(),
                                    lb_policy: LbPolicy::LeastRequest,
                                    endpoints: IndexSet::from_iter(vec![
                                        Endpoint { port: 2222 },
                                        Endpoint { port: 3333 },
                                    ]),
                                },
                            ),
                            (
                                "cluster3".to_string(),
                                Cluster {
                                    name: "cluster3".to_string(),
                                    lb_policy: LbPolicy::LeastRequest,
                                    endpoints: IndexSet::from_iter(vec![
                                        Endpoint { port: 2222 },
                                        Endpoint { port: 3333 },
                                    ]),
                                },
                            ),
                        ]),
                    },
                ),
                (
                    "node2".to_string(),
                    Node {
                        id: "node2".to_string(),
                        clusters: IndexMap::from_iter(vec![
                            (
                                "cluster1".to_string(),
                                Cluster {
                                    name: "cluster1".to_string(),
                                    lb_policy: LbPolicy::RoundRobin,
                                    endpoints: IndexSet::from_iter(vec![
                                        Endpoint { port: 4444 },
                                        Endpoint { port: 5555 },
                                        Endpoint { port: 6666 },
                                    ]),
                                },
                            ),
                            (
                                "cluster4".to_string(),
                                Cluster {
                                    name: "cluster4".to_string(),
                                    lb_policy: LbPolicy::LeastRequest,
                                    endpoints: IndexSet::from_iter(vec![
                                        Endpoint { port: 7777 },
                                        Endpoint { port: 8888 },
                                        Endpoint { port: 9999 },
                                    ]),
                                },
                            ),
                        ]),
                    },
                ),
            ]),
        }
    }
}
