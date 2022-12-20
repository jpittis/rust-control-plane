use crate::cache::Cache;
use crate::service::common::{Service, StreamResponse};
use crate::snapshot::type_url::ANY_TYPE;
use data_plane_api::envoy::service::discovery::v3::aggregated_discovery_service_server::AggregatedDiscoveryService;
use data_plane_api::envoy::service::discovery::v3::{
    DeltaDiscoveryRequest, DeltaDiscoveryResponse, DiscoveryRequest, DiscoveryResponse,
};
use tonic::{Request, Response, Status, Streaming};

#[tonic::async_trait]
impl<C: Cache> AggregatedDiscoveryService for Service<C> {
    type StreamAggregatedResourcesStream = StreamResponse<DiscoveryResponse>;

    async fn stream_aggregated_resources(
        &self,
        req: Request<Streaming<DiscoveryRequest>>,
    ) -> Result<Response<Self::StreamAggregatedResourcesStream>, Status> {
        self.stream(req, ANY_TYPE)
    }

    type DeltaAggregatedResourcesStream = StreamResponse<DeltaDiscoveryResponse>;

    async fn delta_aggregated_resources(
        &self,
        _: Request<Streaming<DeltaDiscoveryRequest>>,
    ) -> Result<Response<Self::DeltaAggregatedResourcesStream>, Status> {
        unimplemented!()
    }
}
