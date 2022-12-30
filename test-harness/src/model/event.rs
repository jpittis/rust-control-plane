use crate::model::{Cluster, Endpoint, Fleet, Node};
use data_plane_api::envoy::config::cluster::v3::cluster::LbPolicy;
use indexmap::map::Entry;
use indexmap::IndexSet;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct FleetEvent {
    pub node_id: String,
    pub node_event: NodeEvent,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum NodeEvent {
    AddCluster(AddCluster),
    RemoveCluster(RemoveCluster),
    UpdateCluster(UpdateCluster),
    AddEndpoint(AddEndpoint),
    RemoveEndpoint(RemoveEndpoint),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct AddCluster {
    pub name: String,
    pub lb_policy: LbPolicy,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct RemoveCluster {
    pub name: String,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct UpdateCluster {
    pub name: String,
    pub lb_policy: LbPolicy,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct AddEndpoint {
    pub cluster_name: String,
    pub endpoint: Endpoint,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct RemoveEndpoint {
    pub cluster_name: String,
    pub endpoint: Endpoint,
}
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ApplyError {
    DuplicateCluster(String),
    ClusterNotFound(String),
    DuplicateEndpoint(Endpoint),
    EndpointNotFound(Endpoint),
    NodeNotFound(String),
}

pub fn apply_node_event(node: &mut Node, event: &NodeEvent) -> Result<(), ApplyError> {
    use NodeEvent::*;
    match event {
        AddCluster(e) => match node.clusters.entry(e.name.clone()) {
            Entry::Occupied(_) => Err(ApplyError::DuplicateCluster(e.name.clone())),
            Entry::Vacant(v) => {
                v.insert(Cluster {
                    name: e.name.clone(),
                    lb_policy: e.lb_policy,
                    endpoints: IndexSet::new(),
                });
                Ok(())
            }
        },
        RemoveCluster(e) => match node.clusters.remove(&e.name) {
            None => Err(ApplyError::ClusterNotFound(e.name.clone())),
            Some(_) => Ok(()),
        },
        UpdateCluster(e) => match node.clusters.get_mut(&e.name) {
            None => Err(ApplyError::ClusterNotFound(e.name.clone())),
            Some(cluster) => {
                cluster.lb_policy = e.lb_policy;
                Ok(())
            }
        },
        AddEndpoint(e) => match node.clusters.get_mut(&e.cluster_name) {
            None => Err(ApplyError::ClusterNotFound(e.cluster_name.clone())),
            Some(cluster) => match cluster.endpoints.insert(e.endpoint.clone()) {
                false => Err(ApplyError::DuplicateEndpoint(e.endpoint.clone())),
                true => Ok(()),
            },
        },
        RemoveEndpoint(e) => match node.clusters.get_mut(&e.cluster_name) {
            None => Err(ApplyError::ClusterNotFound(e.cluster_name.clone())),
            Some(cluster) => match cluster.endpoints.remove(&e.endpoint) {
                false => Err(ApplyError::EndpointNotFound(e.endpoint.clone())),
                true => Ok(()),
            },
        },
    }
}

pub fn apply_fleet_event(fleet: &mut Fleet, event: &FleetEvent) -> Result<(), ApplyError> {
    match fleet.nodes.get_mut(&event.node_id) {
        None => Err(ApplyError::NodeNotFound(event.node_id.clone())),
        Some(node) => apply_node_event(node, &event.node_event),
    }
}
