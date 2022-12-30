use crate::event::{
    AddCluster, AddEndpoint, Event, Model, RemoveCluster, RemoveEndpoint, UpdateCluster,
};
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashMap;

struct Pool<T> {
    pool: Vec<T>,
}

impl<T> Pool<T> {
    fn take(&mut self, rng: &mut ThreadRng) -> Option<T> {
        if self.pool.is_empty() {
            return None;
        }
        let index = rng.gen_range(0..self.pool.len());
        Some(self.pool.remove(index))
    }

    fn put(&mut self, value: T) {
        self.pool.push(value);
    }
}

struct Generator {
    cluster_pool: Pool<String>,
    endpoint_pools: HashMap<String, Pool<u32>>,
}

impl Generator {
    fn next(&mut self, rng: &mut ThreadRng, model: &Model) -> Event {
        let mut possible = Vec::new();
        if let Some(name) = self.cluster_pool.take(rng) {
            possible.push(Event::AddCluster(AddCluster {
                name,
                lb_policy: random_lb_policy(rng),
            }));
        }
        if let Some(name) = choose_cluster(rng, model) {
            possible.push(Event::UpdateCluster(UpdateCluster {
                name: name.clone(),
                lb_policy: random_lb_policy(rng),
            }));
            possible.push(Event::RemoveCluster(RemoveCluster { name: name.clone() }));
        }
        for (name, cluster) in &model.clusters {
            if let Some(port) = self.endpoint_pools.get_mut(name).unwrap().take(rng) {
                possible.push(Event::AddEndpoint(AddEndpoint {
                    cluster_name: name.clone(),
                    address: "127.0.0.1".to_string(),
                    port,
                }));
            }
            if let Some(endpoint) = cluster.endpoints.choose(rng) {
                possible.push(Event::RemoveEndpoint(RemoveEndpoint {
                    cluster_name: name.clone(),
                    address: "127.0.0.1".to_string(),
                    port: endpoint.port,
                }));
            }
        }
        let chosen = possible.choose(rng).unwrap();
        match chosen {
            Event::RemoveCluster(e) => self.cluster_pool.put(e.name.clone()),
            Event::RemoveEndpoint(e) => self
                .endpoint_pools
                .get_mut(&e.cluster_name)
                .unwrap()
                .put(e.port),
            _ => (),
        }
        chosen.clone()
    }
}

fn choose_cluster(rng: &mut ThreadRng, model: &Model) -> Option<String> {
    let keys: Vec<&String> = model.clusters.keys().collect();
    keys.choose(rng).map(|chosen| chosen.to_string())
}

fn random_lb_policy(rng: &mut ThreadRng) -> String {
    if rng.gen_bool(0.5) {
        "ROUND_ROBIN".to_string()
    } else {
        "LEAST_REQUEST".to_string()
    }
}

#[cfg(test)]
mod test {
    use super::{Generator, Pool};
    use crate::event::Model;
    use std::collections::HashMap;

    #[test]
    fn generate_multiple_events() {
        let mut model = Model::new();
        let mut gen = Generator {
            cluster_pool: Pool {
                pool: vec!["foo".to_string(), "bar".to_string(), "baz".to_string()],
            },
            endpoint_pools: HashMap::from_iter(vec![
                (
                    "foo".to_string(),
                    Pool {
                        pool: vec![3333, 4444, 5555, 6666],
                    },
                ),
                (
                    "bar".to_string(),
                    Pool {
                        pool: vec![3333, 4444, 5555, 6666],
                    },
                ),
                (
                    "baz".to_string(),
                    Pool {
                        pool: vec![3333, 4444, 5555, 6666],
                    },
                ),
            ]),
        };
        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            let event = gen.next(&mut rng, &model);
            model.apply_one(&event).unwrap();
        }
        panic!("{:?}", model.clusters);
    }
}
