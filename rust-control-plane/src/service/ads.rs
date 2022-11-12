use crate::service::common::{Service, StreamResponse};
use data_plane_api::envoy::service::discovery::v3::aggregated_discovery_service_server::AggregatedDiscoveryService;
use data_plane_api::envoy::service::discovery::v3::{
    DeltaDiscoveryRequest, DeltaDiscoveryResponse, DiscoveryRequest, DiscoveryResponse,
};

use tonic::{Request, Response, Status, Streaming};

#[tonic::async_trait]
impl AggregatedDiscoveryService for Service {
    type StreamAggregatedResourcesStream = StreamResponse<DiscoveryResponse>;

    async fn stream_aggregated_resources(
        &self,
        _: Request<Streaming<DiscoveryRequest>>,
    ) -> Result<Response<Self::StreamAggregatedResourcesStream>, Status> {
        unimplemented!()
    }

    type DeltaAggregatedResourcesStream = StreamResponse<DeltaDiscoveryResponse>;

    async fn delta_aggregated_resources(
        &self,
        _: Request<Streaming<DeltaDiscoveryRequest>>,
    ) -> Result<Response<Self::DeltaAggregatedResourcesStream>, Status> {
        unimplemented!()
    }
}
