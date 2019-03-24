//use crate::snapshot::NodeHash;
//
//use futures::sync::oneshot::Receiver;
//use futures::Future;
//use std::collections::HashMap;
//
//use rust_xds_grpc::cds::Cluster;
//use rust_xds_grpc::eds::ClusterLoadAssignment;
//use rust_xds_grpc::lds::Listener;
//use rust_xds_grpc::rds::RouteConfiguration;
//use rust_xds_grpc::sds::SdsDummy;
//
//use protobuf::Message;
//
//use rust_xds_grpc::base::Node;
//use rust_xds_grpc::discovery::{DiscoveryRequest, DiscoveryResponse};
//
//const typePrefix: &'static str = "type.googleapis.com/envoy.api.v2.";
//const EndpointType: &'static str = concat!("type.googleapis.com/envoy.api.v2.", "ClusterLoadAssignment");
//const ClusterType: &'static str = concat!("type.googleapis.com/envoy.api.v2.", "Cluster");
//const RouteType: &'static str = concat!("type.googleapis.com/envoy.api.v2.", "RouteConfiguration");
//const ListenerType: &'static str = concat!("type.googleapis.com/envoy.api.v2.", "Listener");
//const SecretType: &'static str = concat!("type.googleapis.com/envoy.api.v2.", "auth.Secret");
//
//struct Response<M: Message> {
//    version: String,
//    resources: Resources<M>,
//}
//
//trait Cache {
//    fn fetch_endpoints(&mut self, req: DiscoveryRequest) -> Response<ClusterLoadAssignment>;
//    // fn watch_endpoints(&mut self, req: DiscoveryRequest) -> Receiver<Response<Endpoint>>;
//}
//
//struct NodeStatus {
//    node: Node,
//}
//
//impl<H: NodeHash> Cache for SnapshotCache<H> {
//    fn fetch_endpoints(&mut self, req: DiscoveryRequest) -> Response<ClusterLoadAssignment> {
//        self.fetch(req)
//    }
//}

// fn build_response(
//     req: DiscoveryRequest,
//     resources: ResourceSet,
//     version: &str,
// ) -> DiscoveryResponse {
//     // TODO: If request has a resource_names then don't send all of them.
//     let vec = resources.iter().map(|(_, resource)| resource.clone().msg).collect();
//     let filtered = protobuf::RepeatedField::from_vec(vec);
//
//     let mut resp = DiscoveryResponse::new();
//     resp.set_version_info(version.to_string());
//     resp.set_resources(filtered);
//     resp
// }
