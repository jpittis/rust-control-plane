use data_plane_api::envoy::config::cluster::v3::cluster::{
    ClusterDiscoveryType, DiscoveryType, EdsClusterConfig,
};
use data_plane_api::envoy::config::cluster::v3::Cluster as ClusterPb;
use data_plane_api::envoy::config::core::v3::api_config_source::ApiType;
use data_plane_api::envoy::config::core::v3::config_source::ConfigSourceSpecifier;
use data_plane_api::envoy::config::core::v3::grpc_service::{EnvoyGrpc, TargetSpecifier};
use data_plane_api::envoy::config::core::v3::socket_address::PortSpecifier;
use data_plane_api::envoy::config::core::v3::{
    address, Address, ApiConfigSource, ApiVersion, ConfigSource, GrpcService, SocketAddress,
};
use data_plane_api::envoy::config::endpoint::v3::lb_endpoint::HostIdentifier;
use data_plane_api::envoy::config::endpoint::v3::Endpoint as EndpointPb;
use data_plane_api::envoy::config::endpoint::v3::{
    ClusterLoadAssignment, LbEndpoint, LocalityLbEndpoints,
};
use rust_control_plane::snapshot::type_url;
use rust_control_plane::snapshot::{Resource, Resources, Snapshot};
use std::collections::HashMap;

pub fn to_snapshot(clusters: &[Cluster], version: &str) -> Snapshot {
    let mut snapshot = Snapshot::new();
    let mut cluster_set = Resources::new(version.to_string());
    let mut endpoint_set = Resources::new(version.to_string());

    clusters.iter().for_each(|cluster| {
        if !cluster.hidden {
            cluster_set
                .items
                .insert(cluster.name.clone(), Resource::Cluster(cluster.to_proto()));
        }
        endpoint_set.items.insert(
            cluster.name.clone(),
            Resource::Endpoint(cluster.endpoints_to_proto()),
        );
    });

    snapshot.insert(type_url::CLUSTER.to_string(), cluster_set);
    snapshot.insert(type_url::ENDPOINT.to_string(), endpoint_set);
    snapshot
}

const XDS_CLUSTER_NAME: &str = "xds";

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cluster {
    pub name: String,
    pub endpoints: Vec<Endpoint>,
    pub hidden: bool,
}

impl Cluster {
    pub fn to_proto(&self) -> ClusterPb {
        ClusterPb {
            name: self.name.clone(),
            cluster_discovery_type: Some(ClusterDiscoveryType::Type(DiscoveryType::Eds as i32)),
            eds_cluster_config: Some(EdsClusterConfig {
                eds_config: Some(ConfigSource {
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
                }),
                service_name: String::new(),
            }),
            ..ClusterPb::default()
        }
    }

    pub fn endpoints_to_proto(&self) -> ClusterLoadAssignment {
        let lb_endpoints: Vec<LbEndpoint> = self
            .endpoints
            .iter()
            .map(|endpoint| LbEndpoint {
                host_identifier: Some(HostIdentifier::Endpoint(endpoint.to_proto())),
                ..LbEndpoint::default()
            })
            .collect();
        ClusterLoadAssignment {
            cluster_name: self.name.clone(),
            endpoints: vec![LocalityLbEndpoints {
                lb_endpoints,
                ..LocalityLbEndpoints::default()
            }],
            ..ClusterLoadAssignment::default()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Endpoint {
    pub addr: String,
    pub port: u32,
}

impl Endpoint {
    pub fn to_proto(&self) -> EndpointPb {
        EndpointPb {
            address: Some(Address {
                address: Some(address::Address::SocketAddress(SocketAddress {
                    address: self.addr.clone(),
                    port_specifier: Some(PortSpecifier::PortValue(self.port)),
                    ..SocketAddress::default()
                })),
            }),
            ..EndpointPb::default()
        }
    }
}

pub fn parse_clusters(body: &str) -> Vec<Cluster> {
    let mut clusters = HashMap::new();
    body.split('\n')
        .filter(|line| line.contains("cx_active"))
        .for_each(|line| {
            let mut parts = line.split("::");
            let name = parts.next().unwrap().to_string();
            let addr_port = parts.next().unwrap();
            let mut parts = addr_port.split(':');
            let addr = parts.next().unwrap().to_string();
            let port = parts.next().unwrap().parse::<u32>().unwrap();
            let endpoint = Endpoint { addr, port };
            clusters
                .entry(name.clone())
                .and_modify(|cluster: &mut Cluster| cluster.endpoints.push(endpoint.clone()))
                .or_insert_with(|| Cluster {
                    name,
                    hidden: false,
                    endpoints: vec![endpoint],
                });
        });
    let mut vec = Vec::from_iter(clusters.values().cloned());
    sort_clusters(&mut vec);
    vec
}

pub fn sort_clusters(vec: &mut [Cluster]) {
    vec.iter_mut().for_each(|cluster| cluster.endpoints.sort());
    vec.sort();
}
