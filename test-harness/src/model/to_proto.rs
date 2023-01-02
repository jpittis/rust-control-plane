use crate::model::{Cluster, Endpoint};
use data_plane_api::envoy::config::cluster::v3::cluster::{
    ClusterDiscoveryType, DiscoveryType, EdsClusterConfig,
};
use data_plane_api::envoy::config::cluster::v3::Cluster as ClusterPb;
use data_plane_api::envoy::config::core::v3::api_config_source::ApiType;
use data_plane_api::envoy::config::core::v3::config_source::ConfigSourceSpecifier;
use data_plane_api::envoy::config::core::v3::grpc_service::{EnvoyGrpc, TargetSpecifier};
use data_plane_api::envoy::config::core::v3::socket_address::PortSpecifier;
use data_plane_api::envoy::config::core::v3::{
    address, Address, AggregatedConfigSource, ApiConfigSource, ApiVersion, ConfigSource,
    GrpcService, SocketAddress,
};
use data_plane_api::envoy::config::endpoint::v3::lb_endpoint::HostIdentifier;
use data_plane_api::envoy::config::endpoint::v3::Endpoint as EndpointPb;
use data_plane_api::envoy::config::endpoint::v3::{
    ClusterLoadAssignment, LbEndpoint, LocalityLbEndpoints,
};

const XDS_CLUSTER_NAME: &str = "xds";
const ENDPOINT_ADDR: &str = "127.0.0.1";

pub fn cluster_to_proto(cluster: &Cluster, ads: bool) -> ClusterPb {
    ClusterPb {
        name: cluster.name.clone(),
        cluster_discovery_type: Some(ClusterDiscoveryType::Type(DiscoveryType::Eds as i32)),
        eds_cluster_config: Some(EdsClusterConfig {
            eds_config: cluster_eds_config(ads),
            service_name: String::new(),
        }),
        lb_policy: cluster.lb_policy.into(),
        ..ClusterPb::default()
    }
}

fn cluster_eds_config(ads: bool) -> Option<ConfigSource> {
    if ads {
        Some(ConfigSource {
            resource_api_version: ApiVersion::V3 as i32,
            config_source_specifier: Some(ConfigSourceSpecifier::Ads(AggregatedConfigSource {})),
            ..ConfigSource::default()
        })
    } else {
        Some(ConfigSource {
            resource_api_version: ApiVersion::V3 as i32,
            config_source_specifier: Some(ConfigSourceSpecifier::ApiConfigSource(
                ApiConfigSource {
                    api_type: ApiType::Grpc as i32,
                    transport_api_version: ApiVersion::V3 as i32,
                    grpc_services: vec![GrpcService {
                        target_specifier: Some(TargetSpecifier::EnvoyGrpc(EnvoyGrpc {
                            cluster_name: XDS_CLUSTER_NAME.to_string(),
                            ..EnvoyGrpc::default()
                        })),
                        ..GrpcService::default()
                    }],
                    ..ApiConfigSource::default()
                },
            )),
            ..ConfigSource::default()
        })
    }
}

pub fn endpoints_to_proto(cluster: &Cluster) -> ClusterLoadAssignment {
    let lb_endpoints: Vec<LbEndpoint> = cluster
        .endpoints
        .iter()
        .map(|endpoint| LbEndpoint {
            host_identifier: Some(HostIdentifier::Endpoint(endpoint_to_proto(endpoint))),
            ..LbEndpoint::default()
        })
        .collect();
    ClusterLoadAssignment {
        cluster_name: cluster.name.clone(),
        endpoints: vec![LocalityLbEndpoints {
            lb_endpoints,
            ..LocalityLbEndpoints::default()
        }],
        ..ClusterLoadAssignment::default()
    }
}

fn endpoint_to_proto(endpoint: &Endpoint) -> EndpointPb {
    EndpointPb {
        address: Some(Address {
            address: Some(address::Address::SocketAddress(SocketAddress {
                address: ENDPOINT_ADDR.to_string(),
                port_specifier: Some(PortSpecifier::PortValue(endpoint.port)),
                ..SocketAddress::default()
            })),
        }),
        ..EndpointPb::default()
    }
}
