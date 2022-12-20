use crate::model::{to_snapshot, Cluster, Endpoint};
use crate::process::EnvoyProcess;
use log::info;
use rust_control_plane::cache::SnapshotCache;
use std::sync::Arc;

// 1. Begin with no snapshot.
// 2. Then provide a snapshot of one cluster.

pub fn init() -> Option<Vec<Cluster>> {
    None
}

pub async fn test(cache: Arc<SnapshotCache>, envoy: EnvoyProcess, ads: bool) {
    let snapshot1 = vec![Cluster {
        name: "my-cluster".to_string(),
        hidden: false,
        endpoints: vec![Endpoint {
            addr: "127.0.0.1".to_string(),
            port: 1234,
        }],
    }];
    info!("setting snapshot");
    cache
        .set_snapshot("lol", to_snapshot(&snapshot1, "snapshot1", ads))
        .await;
    envoy.poll_until_eq(snapshot1).await.unwrap();
    info!("snapshot equal");
}
