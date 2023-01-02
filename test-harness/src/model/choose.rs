use crate::model::event::{self, FleetEvent, NodeEvent};
use crate::model::{Endpoint, Fleet, Node};
use data_plane_api::envoy::config::cluster::v3::cluster::LbPolicy;
use rand::distributions::WeightedError;
use rand::seq::SliceRandom;
use rand::Rng;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum NodeEventType {
    InsertCluster,
    RemoveCluster,
    UpdateCluster,
    InsertEndpoint,
    RemoveEndpoint,
}

pub struct NodeEventWeights {
    insert_cluster: usize,
    remove_cluster: usize,
    update_cluster: usize,
    insert_endpoint: usize,
    remove_endpoint: usize,
}

impl Default for NodeEventWeights {
    fn default() -> Self {
        Self {
            insert_cluster: 2,
            remove_cluster: 1,
            update_cluster: 2,
            insert_endpoint: 2,
            remove_endpoint: 1,
        }
    }
}

pub enum ChooseError {
    WeightedError(WeightedError),
    NoNodes,
    NoClusters(String),
    NoEndpoints(String, String),
}

pub fn choose_fleet_event<R: Rng>(
    rng: &mut R,
    fleet: &Fleet,
    weights: &NodeEventWeights,
) -> Result<FleetEvent, ChooseError> {
    let node_event_type =
        choose_node_event_type(rng, fleet, weights).map_err(ChooseError::WeightedError)?;
    let node_id = choose_node_id(rng, fleet).ok_or(ChooseError::NoNodes)?;
    let node = fleet.nodes.get(&node_id).expect("chosen node_id not found");
    let node_event = fill_node_event(rng, node, &node_event_type)?;
    Ok(FleetEvent {
        node_id,
        node_event,
    })
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

fn fill_node_event<R: Rng>(
    rng: &mut R,
    node: &Node,
    node_event_type: &NodeEventType,
) -> Result<NodeEvent, ChooseError> {
    let cluster =
        choose_cluster(rng, node).ok_or_else(|| ChooseError::NoClusters(node.id.clone()))?;
    let port = choose_port(rng, node, &cluster)
        .ok_or_else(|| ChooseError::NoEndpoints(node.id.clone(), cluster.clone()))?;
    use NodeEventType::*;
    let chosen = match node_event_type {
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
            endpoint: Endpoint { port },
        }),
        RemoveEndpoint => NodeEvent::RemoveEndpoint(event::RemoveEndpoint {
            cluster_name: cluster,
            endpoint: Endpoint { port },
        }),
    };
    Ok(chosen)
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
        .expect("chosen cluster not found")
        .endpoints
        .iter()
        .map(|e| e.port)
        .collect::<Vec<u32>>()
        .choose(rng)
        .copied()
}

fn choose_lb_policy<R: Rng>(rng: &mut R) -> LbPolicy {
    if rng.gen_bool(0.5) {
        LbPolicy::RoundRobin
    } else {
        LbPolicy::LeastRequest
    }
}
