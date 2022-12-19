use crate::model;
use crate::model::Cluster;
use crate::process::EnvoyProcess;
use data_plane_api::envoy::service::cluster::v3::cluster_discovery_service_server::ClusterDiscoveryServiceServer;
use data_plane_api::envoy::service::endpoint::v3::endpoint_discovery_service_server::EndpointDiscoveryServiceServer;
use futures::future::FutureExt;
use rust_control_plane::cache::Cache;
use rust_control_plane::service::common::Service;
use std::future::Future;
use std::mem;
use std::sync::Arc;
use tokio::sync::oneshot;
use tonic::transport::Server;

const NODE: &str = "lol";
const XDS_ADDR: &str = "127.0.0.1:5678";

pub struct Test {
    addr: String,
    cache: Arc<Cache>,
    shutdown: Option<oneshot::Sender<()>>,
}

impl Test {
    pub async fn new(init_snapshot: Option<Vec<Cluster>>, ads: bool) -> Self {
        let cache = Arc::new(Cache::new(false));
        if let Some(clusters) = init_snapshot {
            cache
                .set_snapshot(NODE, model::to_snapshot(&clusters, "init", ads))
                .await;
        }
        Self {
            addr: XDS_ADDR.to_string(),
            cache,
            shutdown: None,
        }
    }

    pub async fn run<F, Fut>(&mut self, mut f: F, ads: bool)
    where
        F: FnMut(Arc<Cache>, EnvoyProcess, bool) -> Fut,
        Fut: Future<Output = ()>,
    {
        self.serve_with_shutdown();
        let mut envoy = EnvoyProcess::new(ads);
        envoy.spawn().unwrap();
        envoy.poll_until_started().await.unwrap();
        f(self.cache.clone(), envoy, ads).await;
    }

    fn serve_with_shutdown(&mut self) {
        let (tx, rx) = oneshot::channel::<()>();
        let addr = self.addr.parse().unwrap();
        let cds_service = Service::new(self.cache.clone());
        let eds_service = Service::new(self.cache.clone());
        let cds = ClusterDiscoveryServiceServer::new(cds_service);
        let eds = EndpointDiscoveryServiceServer::new(eds_service);
        let server = Server::builder().add_service(cds).add_service(eds);
        tokio::spawn(server.serve_with_shutdown(addr, rx.map(drop)));
        self.shutdown = Some(tx);
    }
}

impl Drop for Test {
    fn drop(&mut self) {
        if let Some(shutdown) = mem::take(&mut self.shutdown) {
            shutdown.send(()).unwrap();
        }
    }
}
