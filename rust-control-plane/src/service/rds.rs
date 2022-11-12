use crate::service::common::{Service, StreamResponse};
use crate::snapshot::type_url::ROUTE;
use data_plane_api::envoy::service::discovery::v3::{
    DeltaDiscoveryRequest, DeltaDiscoveryResponse, DiscoveryRequest, DiscoveryResponse,
};
use data_plane_api::envoy::service::route::v3::route_discovery_service_server::RouteDiscoveryService;
use tonic::{Request, Response, Status, Streaming};

#[tonic::async_trait]
impl RouteDiscoveryService for Service {
    type StreamRoutesStream = StreamResponse<DiscoveryResponse>;

    async fn stream_routes(
        &self,
        req: Request<Streaming<DiscoveryRequest>>,
    ) -> Result<Response<Self::StreamRoutesStream>, Status> {
        self.stream(req, ROUTE)
    }

    type DeltaRoutesStream = StreamResponse<DeltaDiscoveryResponse>;

    async fn delta_routes(
        &self,
        _: Request<Streaming<DeltaDiscoveryRequest>>,
    ) -> Result<Response<Self::DeltaRoutesStream>, Status> {
        unimplemented!()
    }

    async fn fetch_routes(
        &self,
        req: Request<DiscoveryRequest>,
    ) -> Result<Response<DiscoveryResponse>, Status> {
        self.fetch(req.get_ref(), ROUTE)
    }
}
