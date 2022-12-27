use crate::cache::Cache;
use crate::service::common::{Service, StreamResponse};
use crate::snapshot::type_url::LISTENER;
use data_plane_api::envoy::service::discovery::v3::{
    DeltaDiscoveryRequest, DeltaDiscoveryResponse, DiscoveryRequest, DiscoveryResponse,
};
use data_plane_api::envoy::service::listener::v3::listener_discovery_service_server::ListenerDiscoveryService;
use tonic::{Request, Response, Status, Streaming};

#[tonic::async_trait]
impl<C: Cache> ListenerDiscoveryService for Service<C> {
    type StreamListenersStream = StreamResponse<DiscoveryResponse>;

    async fn stream_listeners(
        &self,
        req: Request<Streaming<DiscoveryRequest>>,
    ) -> Result<Response<Self::StreamListenersStream>, Status> {
        self.stream(req, LISTENER)
    }

    type DeltaListenersStream = StreamResponse<DeltaDiscoveryResponse>;

    async fn delta_listeners(
        &self,
        req: Request<Streaming<DeltaDiscoveryRequest>>,
    ) -> Result<Response<Self::DeltaListenersStream>, Status> {
        self.delta_stream(req, LISTENER)
    }

    async fn fetch_listeners(
        &self,
        req: Request<DiscoveryRequest>,
    ) -> Result<Response<DiscoveryResponse>, Status> {
        self.fetch(req.get_ref(), LISTENER).await
    }
}
