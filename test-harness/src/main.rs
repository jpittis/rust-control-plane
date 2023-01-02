mod model;
mod process;

use crate::model::generate::Generator;
use crate::model::{Fleet, Node};
use clap::Parser;
use data_plane_api::envoy::service::cluster::v3::cluster_discovery_service_server::ClusterDiscoveryServiceServer;
use data_plane_api::envoy::service::discovery::v3::aggregated_discovery_service_server::AggregatedDiscoveryServiceServer;
use data_plane_api::envoy::service::endpoint::v3::endpoint_discovery_service_server::EndpointDiscoveryServiceServer;
use model::to_proto::{cluster_to_proto, endpoints_to_proto};
use rand_pcg::Pcg64;
use rand_seeder::Seeder;
use rust_control_plane::cache::snapshot::SnapshotCache;
use rust_control_plane::cache::Cache;
use rust_control_plane::service::common::Service;
use rust_control_plane::snapshot::type_url;
use rust_control_plane::snapshot::{Resource, Resources, Snapshot};
use std::sync::Arc;
use tokio::time::Duration;
use tonic::transport::Server;
use tracing::info;

#[derive(Parser, Debug)]
#[command()]
struct Args {
    /// Whether to configure client Envoys (and the cache) to use ADS.
    #[arg(short, long, default_value_t = false)]
    ads: bool,

    /// Whether to configure client Envoys to use the state of the world (default), or incremental
    /// delta gRPC protocol.
    #[arg(short, long, default_value_t = false)]
    delta: bool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    info!("Running with args={:?}", args);

    let mut envoy = process::Process::new(process::Config {
        ads: args.ads,
        delta: args.delta,
        service_node: "lol".to_string(),
        service_cluster: "wat".to_string(),
    });
    envoy.spawn().expect("Failed to spawn");

    process::poll::until(
        process::poll::Config {
            timeout: Duration::from_secs(5),
            backoff: Duration::from_secs(1),
        },
        || async { process::get::config_dump(9901).await },
    )
    .await
    .expect("Envoy failed to boot");

    let mut rng: Pcg64 = Seeder::from("main").make_rng();
    let mut gen = Generator::default();

    let cache = Arc::new(SnapshotCache::new(args.ads));
    set_model(&cache, gen.current(), "0", args.ads).await;

    let cds_service = Service::new(cache.clone());
    let eds_service = Service::new(cache.clone());
    let ads_service = Service::new(cache.clone());
    let cds = ClusterDiscoveryServiceServer::new(cds_service);
    let eds = EndpointDiscoveryServiceServer::new(eds_service);
    let ads = AggregatedDiscoveryServiceServer::new(ads_service);
    let server = Server::builder()
        .add_service(cds)
        .add_service(eds)
        .add_service(ads);
    let addr = "127.0.0.1:5678".parse().expect("Failed to parse xDS addr");
    tokio::spawn(server.serve(addr));

    let mut version = 1;
    loop {
        for _ in 0..10 {
            gen.next(&mut rng).unwrap();
        }
        set_model(&cache, gen.current(), &version.to_string(), args.ads).await;
        let expected_config_dump = fleet_to_config_dump(gen.current());
        let result: Result<(), process::get::EqualityError> = process::poll::until(
            process::poll::Config {
                timeout: Duration::from_secs(5),
                backoff: Duration::from_secs(1),
            },
            || async {
                let config_dump = process::get::config_dump(9901)
                    .await
                    .map_err(process::get::EqualityError::Request)?;
                if config_dump == expected_config_dump {
                    Ok(())
                } else {
                    Err(process::get::EqualityError::ClustersNotEqual(
                        config_dump,
                        expected_config_dump.clone(),
                    ))
                }
            },
        )
        .await;
        result.expect("Not equal");
        version += 1;
    }
}

async fn set_model(cache: &SnapshotCache, fleet: &Fleet, version: &str, ads: bool) {
    for (node_id, node) in &fleet.nodes {
        let snapshot = node_to_snapshot(node, version, ads);
        cache.set_snapshot(node_id, snapshot).await;
    }
}

fn node_to_snapshot(node: &Node, version: &str, ads: bool) -> Snapshot {
    let mut snapshot = Snapshot::new();
    let mut clusters = Resources::new(version.to_string());
    let mut endpoints = Resources::new(version.to_string());

    for cluster in node.clusters.values() {
        clusters.items.insert(
            cluster.name.clone(),
            Resource::Cluster(cluster_to_proto(cluster, ads)),
        );
        endpoints.items.insert(
            cluster.name.clone(),
            Resource::Endpoint(endpoints_to_proto(cluster)),
        );
    }

    snapshot.insert(type_url::CLUSTER.to_string(), clusters);
    snapshot.insert(type_url::ENDPOINT.to_string(), endpoints);
    snapshot
}

fn fleet_to_config_dump(fleet: &Fleet) -> process::get::ConfigDump {
    let mut configs = Vec::new();
    for (_, cluster) in &fleet.nodes.values().next().unwrap().clusters {
        configs.push(process::get::Resource {
            cluster: process::get::Cluster {
                name: cluster.name.clone(),
                lb_config: None,
            },
        })
    }
    process::get::ConfigDump {
        configs: Some(configs),
    }
}
