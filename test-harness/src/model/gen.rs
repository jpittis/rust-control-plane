use crate::model::event::{self, FleetEvent, NodeEvent};
use crate::model::{Endpoint, Fleet, Node};
use data_plane_api::envoy::config::cluster::v3::cluster::LbPolicy;
use indexmap::IndexMap;
use rand::seq::SliceRandom;
use rand::Rng;

type Weighted<T> = (T, usize);

fn choose_weighted<R: Rng, T: Clone>(rng: &mut R, choices: &[Weighted<T>]) -> T {
    choices
        .choose_weighted(rng, |item| item.1)
        .map(|item| item.0.clone())
        .unwrap()
}

fn choose<R: Rng, T: Clone>(rng: &mut R, choices: &[T]) -> T {
    choices.choose(rng).map(|item| item.clone()).unwrap()
}

struct Pool<T> {
    pool: Vec<T>,
}

impl<T> Pool<T> {
    fn new(seed: Vec<T>) -> Self {
        Self { pool: seed }
    }

    fn remove_random<R: Rng>(&mut self, rng: &mut R) -> Option<T> {
        if self.pool.is_empty() {
            return None;
        }
        let index = rng.gen_range(0..self.pool.len());
        Some(self.pool.remove(index))
    }

    fn insert(&mut self, value: T) {
        self.pool.push(value);
    }

    fn len(&self) -> usize {
        self.pool.len()
    }
}

struct NodePool {
    pub cluster_pool: Pool<String>,
    pub endpoint_pools: IndexMap<String, Pool<u32>>,
}

impl NodePool {
    fn new(cluster_seed: Vec<String>, endpoint_seed: Vec<u32>) -> Self {
        let mut endpoint_pools = IndexMap::new();
        for cluster_name in &cluster_seed {
            endpoint_pools.insert(cluster_name.clone(), Pool::new(endpoint_seed.clone()));
        }
        Self {
            cluster_pool: Pool::new(cluster_seed),
            endpoint_pools,
        }
    }
}

struct FleetPool {
    pub fleet_pool: Pool<String>,
    pub node_pools: IndexMap<String, NodePool>,
}

impl FleetPool {
    fn new(node_seed: Vec<String>, cluster_seed: Vec<String>, endpoint_seed: Vec<u32>) -> Self {
        let mut node_pools = IndexMap::new();
        for node_id in &node_seed {
            node_pools.insert(
                node_id.clone(),
                NodePool::new(cluster_seed.clone(), endpoint_seed.clone()),
            );
        }
        Self {
            fleet_pool: Pool::new(node_seed),
            node_pools,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
enum NodeEventChoice {
    AddCluster,
    RemoveCluster,
    UpdateCluster,
    AddEndpoint,
    RemoveEndpoint,
}

struct Generator<R: Rng> {
    rng: R,
    fleet_pool: FleetPool,
}

impl<R: Rng> Generator<R> {
    pub fn new(
        rng: R,
        node_seed: Vec<String>,
        cluster_seed: Vec<String>,
        endpoint_seed: Vec<u32>,
    ) -> Self {
        Self {
            rng,
            fleet_pool: FleetPool::new(node_seed, cluster_seed, endpoint_seed),
        }
    }

    pub fn next(&mut self, fleet: &Fleet) -> FleetEvent {
        let nodes = fleet.nodes.keys().collect::<Vec<&String>>();
        let node_id = choose(&mut self.rng, &nodes);
        let node = fleet.nodes.get(node_id).unwrap();
        let event_choice = self.choose_node_event(node_id, node);
        self.fill_event(node_id, event_choice, node)
    }

    fn choose_node_event(&mut self, node_id: &str, node: &Node) -> NodeEventChoice {
        let mut choices = Vec::new();
        let node_pool = self.fleet_pool.node_pools.get(node_id).unwrap();
        use NodeEventChoice::*;
        choices.push((AddCluster, node_pool.cluster_pool.len() * 6));
        choices.push((RemoveCluster, node.clusters.len()));
        choices.push((UpdateCluster, node.clusters.len() * 3));
        choices.push((
            AddEndpoint,
            node_pool
                .endpoint_pools
                .iter()
                .filter(|(cluster, _)| node.clusters.contains_key(*cluster))
                .map(|(_, pool)| pool.len())
                .sum::<usize>()
                * 6,
        ));
        choices.push((
            RemoveEndpoint,
            node.clusters
                .values()
                .map(|cluster| cluster.endpoints.len())
                .sum(),
        ));
        choose_weighted(&mut self.rng, &choices)
    }

    fn fill_event(
        &mut self,
        node_id: &str,
        event_choice: NodeEventChoice,
        node: &Node,
    ) -> FleetEvent {
        use NodeEventChoice::*;
        let node_event = match event_choice {
            AddCluster => {
                let lb_policy = self.choose_lb_policy();
                let node_pool = self.fleet_pool.node_pools.get_mut(node_id).unwrap();
                let name = node_pool.cluster_pool.remove_random(&mut self.rng).unwrap();
                NodeEvent::AddCluster(event::AddCluster { name, lb_policy })
            }
            RemoveCluster => {
                let clusters = node.clusters.keys().collect::<Vec<&String>>();
                let name = choose(&mut self.rng, &clusters).clone();
                let node_pool = self.fleet_pool.node_pools.get_mut(node_id).unwrap();
                node_pool.cluster_pool.insert(name.clone());
                NodeEvent::RemoveCluster(event::RemoveCluster { name })
            }
            UpdateCluster => {
                let lb_policy = self.choose_lb_policy();
                let clusters = node.clusters.keys().collect::<Vec<&String>>();
                let name = choose(&mut self.rng, &clusters).clone();
                NodeEvent::UpdateCluster(event::UpdateCluster { name, lb_policy })
            }
            AddEndpoint => {
                let node_pool = self.fleet_pool.node_pools.get_mut(node_id).unwrap();
                let clusters = node_pool
                    .endpoint_pools
                    .iter()
                    .filter(|(key, value)| value.len() > 0 && node.clusters.contains_key(*key))
                    .map(|(key, _)| key)
                    .collect::<Vec<&String>>();
                let cluster_name = choose(&mut self.rng, &clusters).clone();
                let port = node_pool
                    .endpoint_pools
                    .get_mut(&cluster_name)
                    .unwrap()
                    .remove_random(&mut self.rng)
                    .unwrap();
                NodeEvent::AddEndpoint(event::AddEndpoint {
                    cluster_name,
                    endpoint: Endpoint {
                        address: "127.0.0.1".to_string(),
                        port,
                    },
                })
            }
            RemoveEndpoint => {
                let clusters = node
                    .clusters
                    .iter()
                    .filter(|(_, value)| !value.endpoints.is_empty())
                    .map(|(key, _)| key)
                    .collect::<Vec<&String>>();
                let cluster_name = choose(&mut self.rng, &clusters).clone();
                let ports = node
                    .clusters
                    .get(&cluster_name)
                    .unwrap()
                    .endpoints
                    .iter()
                    .map(|endpoint| endpoint.port)
                    .collect::<Vec<u32>>();
                let port = choose(&mut self.rng, &ports);
                let node_pool = self.fleet_pool.node_pools.get_mut(node_id).unwrap();
                node_pool
                    .endpoint_pools
                    .get_mut(&cluster_name)
                    .unwrap()
                    .insert(port);
                NodeEvent::RemoveEndpoint(event::RemoveEndpoint {
                    cluster_name,
                    endpoint: Endpoint {
                        address: "127.0.0.1".to_string(),
                        port,
                    },
                })
            }
        };
        FleetEvent {
            node_id: node_id.to_string(),
            node_event,
        }
    }

    fn choose_lb_policy(&mut self) -> LbPolicy {
        if self.rng.gen_bool(0.5) {
            LbPolicy::RoundRobin
        } else {
            LbPolicy::LeastRequest
        }
    }
}

#[cfg(test)]
mod test {
    use super::Generator;
    use crate::model::event::apply_fleet_event;
    use crate::model::{Cluster, Endpoint, Fleet, Node};
    use data_plane_api::envoy::config::cluster::v3::cluster::LbPolicy;
    use indexmap::{IndexMap, IndexSet};
    use rand_pcg::Pcg64;
    use rand_seeder::Seeder;

    #[test]
    fn test_generator() {
        let rng: Pcg64 = Seeder::from("test generator").make_rng();
        let mut gen = Generator::new(
            rng,
            vec!["node1".to_string(), "node2".to_string()],
            vec![
                "cluster1".to_string(),
                "cluster2".to_string(),
                "cluster3".to_string(),
            ],
            vec![4444, 5555],
        );
        let node_ids = vec!["node1".to_string(), "node2".to_string()];
        let mut fleet = Fleet::new(&node_ids);
        for _ in 0..100 {
            let event = gen.next(&fleet);
            apply_fleet_event(&mut fleet, &event).unwrap();
        }
        let expected = Fleet {
            nodes: IndexMap::from_iter(vec![
                (
                    "node1".to_string(),
                    Node {
                        id: "node1".to_string(),
                        clusters: IndexMap::from_iter(vec![
                            (
                                "cluster3".to_string(),
                                Cluster {
                                    name: "cluster3".to_string(),
                                    lb_policy: LbPolicy::LeastRequest,
                                    endpoints: IndexSet::new(),
                                },
                            ),
                            (
                                "cluster1".to_string(),
                                Cluster {
                                    name: "cluster1".to_string(),
                                    lb_policy: LbPolicy::LeastRequest,
                                    endpoints: IndexSet::new(),
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
                                    endpoints: IndexSet::new(),
                                },
                            ),
                            (
                                "cluster2".to_string(),
                                Cluster {
                                    name: "cluster2".to_string(),
                                    lb_policy: LbPolicy::LeastRequest,
                                    endpoints: IndexSet::from_iter(vec![Endpoint {
                                        address: "127.0.0.1".to_string(),
                                        port: 5555,
                                    }]),
                                },
                            ),
                        ]),
                    },
                ),
            ]),
        };
        use pretty_assertions::assert_eq;
        assert_eq!(fleet, expected);
    }
}
