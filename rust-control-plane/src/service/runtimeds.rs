use crate::service::common::{Service, StreamResponse};
use crate::snapshot::type_url::RUNTIME;
use data_plane_api::envoy::service::discovery::v3::{
    DeltaDiscoveryRequest, DeltaDiscoveryResponse, DiscoveryRequest, DiscoveryResponse,
};
use data_plane_api::envoy::service::runtime::v3::runtime_discovery_service_server::RuntimeDiscoveryService;
use tonic::{Request, Response, Status, Streaming};

#[tonic::async_trait]
impl RuntimeDiscoveryService for Service {
    type StreamRuntimeStream = StreamResponse<DiscoveryResponse>;

    async fn stream_runtime(
        &self,
        req: Request<Streaming<DiscoveryRequest>>,
    ) -> Result<Response<Self::StreamRuntimeStream>, Status> {
        self.stream(req, RUNTIME)
    }

    type DeltaRuntimeStream = StreamResponse<DeltaDiscoveryResponse>;

    async fn delta_runtime(
        &self,
        _: Request<Streaming<DeltaDiscoveryRequest>>,
    ) -> Result<Response<Self::DeltaRuntimeStream>, Status> {
        unimplemented!()
    }

    async fn fetch_runtime(
        &self,
        req: Request<DiscoveryRequest>,
    ) -> Result<Response<DiscoveryResponse>, Status> {
        self.fetch(req.get_ref(), RUNTIME)
    }
}
