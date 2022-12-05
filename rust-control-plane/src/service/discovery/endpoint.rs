use crate::service::common::{Service, StreamResponse};
use crate::snapshot::type_url::ENDPOINT;
use data_plane_api::envoy::service::discovery::v3::{
    DeltaDiscoveryRequest, DeltaDiscoveryResponse, DiscoveryRequest, DiscoveryResponse,
};
use data_plane_api::envoy::service::endpoint::v3::endpoint_discovery_service_server::EndpointDiscoveryService;
use tonic::{Request, Response, Status, Streaming};

#[tonic::async_trait]
impl EndpointDiscoveryService for Service {
    type StreamEndpointsStream = StreamResponse<DiscoveryResponse>;

    async fn stream_endpoints(
        &self,
        req: Request<Streaming<DiscoveryRequest>>,
    ) -> Result<Response<Self::StreamEndpointsStream>, Status> {
        self.stream(req, ENDPOINT)
    }

    type DeltaEndpointsStream = StreamResponse<DeltaDiscoveryResponse>;

    async fn delta_endpoints(
        &self,
        _: Request<Streaming<DeltaDiscoveryRequest>>,
    ) -> Result<Response<Self::DeltaEndpointsStream>, Status> {
        unimplemented!()
    }

    async fn fetch_endpoints(
        &self,
        req: Request<DiscoveryRequest>,
    ) -> Result<Response<DiscoveryResponse>, Status> {
        self.fetch(req.get_ref(), ENDPOINT).await
    }
}
