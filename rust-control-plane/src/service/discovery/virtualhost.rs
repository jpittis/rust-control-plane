use crate::cache::Cache;
use crate::service::common::{Service, StreamResponse};
use data_plane_api::envoy::service::discovery::v3::{
    DeltaDiscoveryRequest, DeltaDiscoveryResponse,
};
use data_plane_api::envoy::service::route::v3::virtual_host_discovery_service_server::VirtualHostDiscoveryService;
use tonic::{Request, Response, Status, Streaming};

#[tonic::async_trait]
impl<C: Cache> VirtualHostDiscoveryService for Service<C> {
    type DeltaVirtualHostsStream = StreamResponse<DeltaDiscoveryResponse>;

    async fn delta_virtual_hosts(
        &self,
        _: Request<Streaming<DeltaDiscoveryRequest>>,
    ) -> Result<Response<Self::DeltaVirtualHostsStream>, Status> {
        unimplemented!()
    }
}
