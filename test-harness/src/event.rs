use std::collections::hash_map::Entry;
use std::collections::HashMap;

#[derive(Clone)]
pub enum Event {
    AddCluster(AddCluster),
    RemoveCluster(RemoveCluster),
    UpdateCluster(UpdateCluster),
    AddEndpoint(AddEndpoint),
    RemoveEndpoint(RemoveEndpoint),
}

#[derive(Clone)]
pub struct AddCluster {
    pub name: String,
    pub lb_policy: String,
}

#[derive(Clone)]
pub struct RemoveCluster {
    pub name: String,
}

#[derive(Clone)]
pub struct UpdateCluster {
    pub name: String,
    pub lb_policy: String,
}

#[derive(Clone)]
pub struct AddEndpoint {
    pub cluster_name: String,
    pub address: String,
    pub port: u32,
}

#[derive(Clone)]
pub struct RemoveEndpoint {
    pub cluster_name: String,
    pub address: String,
    pub port: u32,
}

#[derive(Debug, PartialEq)]
pub struct Cluster {
    pub name: String,
    pub lb_policy: String,
    pub endpoints: Vec<Endpoint>,
}

#[derive(Debug, PartialEq)]
pub struct Endpoint {
    pub address: String,
    pub port: u32,
}

pub struct Model {
    pub clusters: HashMap<String, Cluster>,
}

#[derive(Debug, PartialEq)]
pub enum ApplyError {
    DuplicateCluster(String),
    ClusterNotFound(String),
    EndpointNotFound(String, u32),
}

impl Model {
    pub fn new() -> Self {
        Self {
            clusters: HashMap::new(),
        }
    }

    pub fn apply_all(&mut self, events: &[Event]) -> Result<(), ApplyError> {
        for event in events {
            self.apply_one(event)?;
        }
        Ok(())
    }

    pub fn apply_one(&mut self, event: &Event) -> Result<(), ApplyError> {
        match event {
            Event::AddCluster(e) => match self.clusters.entry(e.name.clone()) {
                Entry::Occupied(_) => Err(ApplyError::DuplicateCluster(e.name.clone())),
                Entry::Vacant(v) => {
                    v.insert(Cluster {
                        name: e.name.clone(),
                        lb_policy: e.lb_policy.clone(),
                        endpoints: Vec::new(),
                    });
                    Ok(())
                }
            },
            Event::RemoveCluster(e) => match self.clusters.remove(&e.name) {
                None => Err(ApplyError::ClusterNotFound(e.name.clone())),
                Some(_) => Ok(()),
            },
            Event::UpdateCluster(e) => match self.clusters.get_mut(&e.name) {
                None => Err(ApplyError::ClusterNotFound(e.name.clone())),
                Some(cluster) => {
                    cluster.lb_policy = e.lb_policy.clone();
                    Ok(())
                }
            },
            Event::AddEndpoint(e) => match self.clusters.get_mut(&e.cluster_name) {
                None => Err(ApplyError::ClusterNotFound(e.cluster_name.clone())),
                Some(cluster) => {
                    cluster.endpoints.push(Endpoint {
                        address: e.address.clone(),
                        port: e.port,
                    });
                    Ok(())
                }
            },
            Event::RemoveEndpoint(e) => match self.clusters.get_mut(&e.cluster_name) {
                None => Err(ApplyError::ClusterNotFound(e.cluster_name.clone())),
                Some(cluster) => {
                    let found = cluster
                        .endpoints
                        .iter()
                        .enumerate()
                        .find(|(_, item)| item.address == e.address && item.port == e.port);
                    match found {
                        None => Err(ApplyError::EndpointNotFound(e.address.clone(), e.port)),
                        Some((index, _)) => {
                            cluster.endpoints.remove(index);
                            Ok(())
                        }
                    }
                }
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::{
        AddCluster, AddEndpoint, ApplyError, Cluster, Endpoint, Event, Model, RemoveCluster,
        RemoveEndpoint, UpdateCluster,
    };
    use std::collections::HashMap;

    #[test]
    fn apply_multiple_events_without_error() {
        let events = vec![
            Event::AddCluster(AddCluster {
                name: "foobar".to_string(),
                lb_policy: "ROUND_ROBIN".to_string(),
            }),
            Event::AddCluster(AddCluster {
                name: "another".to_string(),
                lb_policy: "ROUND_ROBIN".to_string(),
            }),
            Event::AddCluster(AddCluster {
                name: "yet-another".to_string(),
                lb_policy: "ROUND_ROBIN".to_string(),
            }),
            Event::AddEndpoint(AddEndpoint {
                cluster_name: "foobar".to_string(),
                address: "127.0.0.1".to_string(),
                port: 4444,
            }),
            Event::AddEndpoint(AddEndpoint {
                cluster_name: "foobar".to_string(),
                address: "127.0.0.1".to_string(),
                port: 5555,
            }),
            Event::UpdateCluster(UpdateCluster {
                name: "foobar".to_string(),
                lb_policy: "LEAST_REQUEST".to_string(),
            }),
            Event::RemoveEndpoint(RemoveEndpoint {
                cluster_name: "foobar".to_string(),
                address: "127.0.0.1".to_string(),
                port: 4444,
            }),
            Event::RemoveCluster(RemoveCluster {
                name: "yet-another".to_string(),
            }),
        ];
        let mut model = Model {
            clusters: HashMap::new(),
        };
        let result = model.apply_all(&events);
        assert_eq!(result, Ok(()));
        assert_eq!(
            model.clusters,
            HashMap::from_iter(vec![
                (
                    "foobar".to_string(),
                    Cluster {
                        name: "foobar".to_string(),
                        lb_policy: "LEAST_REQUEST".to_string(),
                        endpoints: vec![Endpoint {
                            address: "127.0.0.1".to_string(),
                            port: 5555,
                        }],
                    }
                ),
                (
                    "another".to_string(),
                    Cluster {
                        name: "another".to_string(),
                        lb_policy: "ROUND_ROBIN".to_string(),
                        endpoints: Vec::new(),
                    }
                ),
            ])
        );
    }

    #[test]
    fn remove_endpoint_for_non_existent_cluster() {
        let events = vec![Event::RemoveEndpoint(RemoveEndpoint {
            cluster_name: "foobar".to_string(),
            address: "127.0.0.1".to_string(),
            port: 4444,
        })];
        let mut model = Model {
            clusters: HashMap::new(),
        };
        let result = model.apply_all(&events);
        assert_eq!(
            result,
            Err(ApplyError::ClusterNotFound("foobar".to_string()))
        );
    }

    #[test]
    fn remove_non_existent_endpoint() {
        let events = vec![Event::RemoveEndpoint(RemoveEndpoint {
            cluster_name: "foobar".to_string(),
            address: "127.0.0.1".to_string(),
            port: 4444,
        })];
        let mut model = Model {
            clusters: HashMap::from_iter(vec![(
                "foobar".to_string(),
                Cluster {
                    name: "foobar".to_string(),
                    lb_policy: "ROUND_ROBIN".to_string(),
                    endpoints: Vec::new(),
                },
            )]),
        };
        let result = model.apply_all(&events);
        assert_eq!(
            result,
            Err(ApplyError::EndpointNotFound("127.0.0.1".to_string(), 4444))
        );
    }

    #[test]
    fn remove_non_existent_cluster() {
        let events = vec![Event::RemoveCluster(RemoveCluster {
            name: "foobar".to_string(),
        })];
        let mut model = Model {
            clusters: HashMap::new(),
        };
        let result = model.apply_all(&events);
        assert_eq!(
            result,
            Err(ApplyError::ClusterNotFound("foobar".to_string()))
        );
    }

    #[test]
    fn update_non_existent_cluster() {
        let events = vec![Event::UpdateCluster(UpdateCluster {
            name: "foobar".to_string(),
            lb_policy: "LEAST_REQUEST".to_string(),
        })];
        let mut model = Model {
            clusters: HashMap::new(),
        };
        let result = model.apply_all(&events);
        assert_eq!(
            result,
            Err(ApplyError::ClusterNotFound("foobar".to_string()))
        );
    }

    #[test]
    fn add_endpoint_for_non_existent_cluster() {
        let events = vec![Event::AddEndpoint(AddEndpoint {
            cluster_name: "foobar".to_string(),
            address: "127.0.0.1".to_string(),
            port: 4444,
        })];
        let mut model = Model {
            clusters: HashMap::new(),
        };
        let result = model.apply_all(&events);
        assert_eq!(
            result,
            Err(ApplyError::ClusterNotFound("foobar".to_string()))
        );
    }

    #[test]
    fn add_the_same_cluster_twice() {
        let events = vec![
            Event::AddCluster(AddCluster {
                name: "foobar".to_string(),
                lb_policy: "LEAST_REQUEST".to_string(),
            }),
            Event::AddCluster(AddCluster {
                name: "foobar".to_string(),
                lb_policy: "LEAST_REQUEST".to_string(),
            }),
        ];
        let mut model = Model {
            clusters: HashMap::new(),
        };
        let result = model.apply_all(&events);
        assert_eq!(
            result,
            Err(ApplyError::DuplicateCluster("foobar".to_string()))
        );
    }
}
