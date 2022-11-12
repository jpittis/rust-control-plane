use crate::service::common::{Service, StreamResponse};
use crate::snapshot::type_url::SECRET;
use data_plane_api::envoy::service::discovery::v3::{
    DeltaDiscoveryRequest, DeltaDiscoveryResponse, DiscoveryRequest, DiscoveryResponse,
};
use data_plane_api::envoy::service::secret::v3::secret_discovery_service_server::SecretDiscoveryService;
use tonic::{Request, Response, Status, Streaming};

#[tonic::async_trait]
impl SecretDiscoveryService for Service {
    type StreamSecretsStream = StreamResponse<DiscoveryResponse>;

    async fn stream_secrets(
        &self,
        req: Request<Streaming<DiscoveryRequest>>,
    ) -> Result<Response<Self::StreamSecretsStream>, Status> {
        self.stream(req, SECRET)
    }

    type DeltaSecretsStream = StreamResponse<DeltaDiscoveryResponse>;

    async fn delta_secrets(
        &self,
        _: Request<Streaming<DeltaDiscoveryRequest>>,
    ) -> Result<Response<Self::DeltaSecretsStream>, Status> {
        unimplemented!()
    }

    async fn fetch_secrets(
        &self,
        req: Request<DiscoveryRequest>,
    ) -> Result<Response<DiscoveryResponse>, Status> {
        self.fetch(req.get_ref(), SECRET)
    }
}
