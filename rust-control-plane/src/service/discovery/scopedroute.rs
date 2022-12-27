use crate::cache::Cache;
use crate::service::common::{Service, StreamResponse};
use crate::snapshot::type_url::SCOPED_ROUTE;
use data_plane_api::envoy::service::discovery::v3::{
    DeltaDiscoveryRequest, DeltaDiscoveryResponse, DiscoveryRequest, DiscoveryResponse,
};
use data_plane_api::envoy::service::route::v3::scoped_routes_discovery_service_server::ScopedRoutesDiscoveryService;
use tonic::{Request, Response, Status, Streaming};

#[tonic::async_trait]
impl<C: Cache> ScopedRoutesDiscoveryService for Service<C> {
    type StreamScopedRoutesStream = StreamResponse<DiscoveryResponse>;

    async fn stream_scoped_routes(
        &self,
        req: Request<Streaming<DiscoveryRequest>>,
    ) -> Result<Response<Self::StreamScopedRoutesStream>, Status> {
        self.stream(req, SCOPED_ROUTE)
    }

    type DeltaScopedRoutesStream = StreamResponse<DeltaDiscoveryResponse>;

    async fn delta_scoped_routes(
        &self,
        req: Request<Streaming<DeltaDiscoveryRequest>>,
    ) -> Result<Response<Self::DeltaScopedRoutesStream>, Status> {
        self.delta_stream(req, SCOPED_ROUTE)
    }

    async fn fetch_scoped_routes(
        &self,
        req: Request<DiscoveryRequest>,
    ) -> Result<Response<DiscoveryResponse>, Status> {
        self.fetch(req.get_ref(), SCOPED_ROUTE).await
    }
}
