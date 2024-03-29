use crate::cache::Cache;
use crate::service::common::{Service, StreamResponse};
use crate::snapshot::type_url::CLUSTER;
use data_plane_api::envoy::service::cluster::v3::cluster_discovery_service_server::ClusterDiscoveryService;
use data_plane_api::envoy::service::discovery::v3::{
    DeltaDiscoveryRequest, DeltaDiscoveryResponse, DiscoveryRequest, DiscoveryResponse,
};
use tonic::{Request, Response, Status, Streaming};

#[tonic::async_trait]
impl<C: Cache> ClusterDiscoveryService for Service<C> {
    type StreamClustersStream = StreamResponse<DiscoveryResponse>;

    async fn stream_clusters(
        &self,
        req: Request<Streaming<DiscoveryRequest>>,
    ) -> Result<Response<Self::StreamClustersStream>, Status> {
        self.stream(req, CLUSTER)
    }

    type DeltaClustersStream = StreamResponse<DeltaDiscoveryResponse>;

    async fn delta_clusters(
        &self,
        req: Request<Streaming<DeltaDiscoveryRequest>>,
    ) -> Result<Response<Self::DeltaClustersStream>, Status> {
        self.delta_stream(req, CLUSTER)
    }

    async fn fetch_clusters(
        &self,
        req: Request<DiscoveryRequest>,
    ) -> Result<Response<DiscoveryResponse>, Status> {
        self.fetch(req.get_ref(), CLUSTER).await
    }
}
