use crate::cache::Cache;
use crate::service::common::{Service, StreamResponse};
use crate::snapshot::type_url::EXTENSION_CONFIG;
use data_plane_api::envoy::service::discovery::v3::{
    DeltaDiscoveryRequest, DeltaDiscoveryResponse, DiscoveryRequest, DiscoveryResponse,
};
use data_plane_api::envoy::service::extension::v3::extension_config_discovery_service_server::ExtensionConfigDiscoveryService;
use tonic::{Request, Response, Status, Streaming};

#[tonic::async_trait]
impl<C: Cache> ExtensionConfigDiscoveryService for Service<C> {
    type StreamExtensionConfigsStream = StreamResponse<DiscoveryResponse>;

    async fn stream_extension_configs(
        &self,
        req: Request<Streaming<DiscoveryRequest>>,
    ) -> Result<Response<Self::StreamExtensionConfigsStream>, Status> {
        self.stream(req, EXTENSION_CONFIG)
    }

    type DeltaExtensionConfigsStream = StreamResponse<DeltaDiscoveryResponse>;

    async fn delta_extension_configs(
        &self,
        _: Request<Streaming<DeltaDiscoveryRequest>>,
    ) -> Result<Response<Self::DeltaExtensionConfigsStream>, Status> {
        unimplemented!()
    }

    async fn fetch_extension_configs(
        &self,
        req: Request<DiscoveryRequest>,
    ) -> Result<Response<DiscoveryResponse>, Status> {
        self.fetch(req.get_ref(), EXTENSION_CONFIG).await
    }
}
