use crate::model::{to_snapshot, Cluster, Endpoint};
use crate::process::EnvoyProcess;
use log::info;
use rust_control_plane::cache::Cache;
use std::sync::Arc;

// 1. Begin with a snapshot of one cluster, but endpoints for another cluster.
// 2. Then provide a snapshot with the new cluster, so it's endpoints are re-fetched.

pub fn init() -> Option<Vec<Cluster>> {
    Some(vec![
        Cluster {
            name: "my-cluster".to_string(),
            hidden: false,
            endpoints: vec![Endpoint {
                addr: "127.0.0.1".to_string(),
                port: 1234,
            }],
        },
        Cluster {
            name: "my-second-cluster".to_string(),
            hidden: true,
            endpoints: vec![Endpoint {
                addr: "127.0.0.1".to_string(),
                port: 1234,
            }],
        },
    ])
}

pub async fn test(cache: Arc<Cache>, envoy: EnvoyProcess, ads: bool) {
    envoy.poll_until_eq(init().unwrap()).await.unwrap();
    info!("init equal");
    let snapshot1 = vec![
        Cluster {
            hidden: false,
            name: "my-cluster".to_string(),
            endpoints: vec![Endpoint {
                addr: "127.0.0.1".to_string(),
                port: 1234,
            }],
        },
        Cluster {
            name: "my-second-cluster".to_string(),
            hidden: false,
            endpoints: vec![Endpoint {
                addr: "127.0.0.1".to_string(),
                port: 1234,
            }],
        },
    ];
    info!("setting snapshot");
    cache
        .set_snapshot("lol", to_snapshot(&snapshot1, "snapshot1", ads))
        .await;
    envoy.poll_until_eq(snapshot1).await.unwrap();
    info!("snapshot equal");
}
