use crate::model::event::{self, FleetEvent, NodeEvent};
use crate::model::{Endpoint, Fleet, Node};
use data_plane_api::envoy::config::cluster::v3::cluster::LbPolicy;
use indexmap::IndexSet;
use rand::distributions::WeightedError;
use rand::seq::SliceRandom;
use rand::Rng;

#[derive(Debug, Eq, PartialEq, Clone)]
enum NodeEventType {
    InsertCluster,
    RemoveCluster,
    UpdateCluster,
    InsertEndpoint,
    RemoveEndpoint,
}

struct NodeEventWeights {
    insert_cluster: usize,
    remove_cluster: usize,
    update_cluster: usize,
    insert_endpoint: usize,
    remove_endpoint: usize,
}

impl Default for NodeEventWeights {
    fn default() -> Self {
        Self {
            insert_cluster: 1,
            remove_cluster: 1,
            update_cluster: 1,
            insert_endpoint: 1,
            remove_endpoint: 1,
        }
    }
}

enum ChooseError {
    NoNodes,
}

fn choose_node_event<R: Rng>(rng: &mut R, fleet: &Fleet, weights: &NodeEventWeights) -> FleetEvent {
    let event_type = choose_node_event_type(rng, fleet, weights).unwrap(); // TODO: unwrap
    let node_id = choose_node_id(rng, fleet).unwrap(); // TODO: unwrap
    let node = fleet.nodes.get(&node_id).unwrap(); // TODO: unwrap
    let node_event = fill_node_event(rng, node, &event_type);

    FleetEvent {
        node_id,
        node_event,
    }
}

fn choose_node_event_type<R: Rng>(
    rng: &mut R,
    fleet: &Fleet,
    weights: &NodeEventWeights,
) -> Result<NodeEventType, WeightedError> {
    let mut choices = Vec::new();
    let num_clusters: usize = fleet.nodes.values().map(|node| node.clusters.len()).sum();
    let num_endpoints: usize = fleet
        .nodes
        .values()
        .flat_map(|node| {
            node.clusters
                .values()
                .map(|cluster| cluster.endpoints.len())
        })
        .sum();
    use NodeEventType::*;
    choices.push((InsertCluster, num_clusters * weights.insert_cluster));
    choices.push((RemoveCluster, num_clusters * weights.remove_cluster));
    choices.push((UpdateCluster, num_clusters * weights.update_cluster));
    choices.push((InsertEndpoint, num_endpoints * weights.insert_endpoint));
    choices.push((RemoveEndpoint, num_endpoints * weights.remove_endpoint));
    let choice = choices.choose_weighted(rng, |item| item.1)?;
    Ok(choice.0.clone())
}

fn choose_node_id<R: Rng>(rng: &mut R, fleet: &Fleet) -> Option<String> {
    fleet
        .nodes
        .keys()
        .collect::<Vec<&String>>()
        .choose(rng)
        .map(|&n| n.clone())
}

fn fill_node_event<R: Rng>(rng: &mut R, node: &Node, event_type: &NodeEventType) -> NodeEvent {
    let cluster = choose_cluster(rng, node).unwrap(); // TODO: unwrap
    let port = choose_port(rng, node, &cluster).unwrap(); // TODO: unwrap
    use NodeEventType::*;
    match event_type {
        InsertCluster => NodeEvent::InsertCluster(event::InsertCluster {
            name: cluster,
            lb_policy: choose_lb_policy(rng),
        }),
        RemoveCluster => NodeEvent::RemoveCluster(event::RemoveCluster { name: cluster }),
        UpdateCluster => NodeEvent::UpdateCluster(event::UpdateCluster {
            name: cluster,
            lb_policy: choose_lb_policy(rng),
        }),
        InsertEndpoint => NodeEvent::InsertEndpoint(event::InsertEndpoint {
            cluster_name: cluster,
            endpoint: Endpoint {
                address: "127.0.0.1".to_string(),
                port,
            },
        }),
        RemoveEndpoint => NodeEvent::RemoveEndpoint(event::RemoveEndpoint {
            cluster_name: cluster,
            endpoint: Endpoint {
                address: "127.0.0.1".to_string(),
                port,
            },
        }),
    }
}

fn choose_cluster<R: Rng>(rng: &mut R, node: &Node) -> Option<String> {
    node.clusters
        .keys()
        .collect::<Vec<&String>>()
        .choose(rng)
        .map(|&n| n.clone())
}

fn choose_port<R: Rng>(rng: &mut R, node: &Node, cluster: &str) -> Option<u32> {
    node.clusters
        .get(cluster)
        .unwrap()
        .endpoints
        .iter()
        .map(|e| e.port)
        .collect::<Vec<u32>>()
        .choose(rng)
        .map(|p| *p)
}

fn choose_lb_policy<R: Rng>(rng: &mut R) -> LbPolicy {
    if rng.gen_bool(0.5) {
        LbPolicy::RoundRobin
    } else {
        LbPolicy::LeastRequest
    }
}
