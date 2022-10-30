mod check;
mod model;

use data_plane_api::envoy::service::cluster::v3::cluster_discovery_service_server::ClusterDiscoveryServiceServer;
use data_plane_api::envoy::service::endpoint::v3::endpoint_discovery_service_server::EndpointDiscoveryServiceServer;
use log::info;
use rust_control_plane::cache::Cache;
use rust_control_plane::service::common::Service;
use rust_control_plane::snapshot::type_url;
use rust_control_plane::snapshot::{Resource, Resources, Snapshot};
use std::sync::Arc;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let addr = "127.0.0.1:5678".parse().unwrap();
    let cache = Arc::new(Cache::new());

    let test1 = vec![
        model::Cluster {
            name: "xds".to_string(),
            endpoints: vec![model::Endpoint {
                addr: "127.0.0.1".to_string(),
                port: 5678,
            }],
        },
        model::Cluster {
            name: "my-cluster".to_string(),
            endpoints: vec![model::Endpoint {
                addr: "127.0.0.1".to_string(),
                port: 1234,
            }],
        },
    ];
    cache.set_snapshot("lol", model::to_snapshot(&test1, "test1"));

    let cds_service = Service::new(cache.clone());
    let eds_service = Service::new(cache.clone());
    let cds = ClusterDiscoveryServiceServer::new(cds_service);
    let eds = EndpointDiscoveryServiceServer::new(eds_service);

    info!("listening on {}", addr);
    let server = Server::builder().add_service(cds).add_service(eds);
    tokio::spawn(server.serve(addr));

    info!("waiting for envoy");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    info!("test 1");
    check::poll_until_eq(test1).await?;

    info!("test 2");
    let test2 = vec![
        model::Cluster {
            name: "xds".to_string(),
            endpoints: vec![model::Endpoint {
                addr: "127.0.0.1".to_string(),
                port: 5678,
            }],
        },
        model::Cluster {
            name: "my-cluster".to_string(),
            endpoints: vec![
                model::Endpoint {
                    addr: "127.0.0.1".to_string(),
                    port: 1234,
                },
                model::Endpoint {
                    addr: "127.0.0.1".to_string(),
                    port: 4321,
                },
            ],
        },
        model::Cluster {
            name: "my-second-cluster".to_string(),
            endpoints: vec![model::Endpoint {
                addr: "127.0.0.1".to_string(),
                port: 1234,
            }],
        },
    ];
    cache.set_snapshot("lol", model::to_snapshot(&test2, "test2"));
    check::poll_until_eq(test2).await?;

    Ok(())
}
