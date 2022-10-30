use crate::cache::Cache;
use data_plane_api::envoy::service::discovery::v3::{DiscoveryRequest, DiscoveryResponse};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use log::info;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tonic::{Status, Streaming};

pub async fn handle_stream(
    mut stream: Streaming<DiscoveryRequest>,
    tx: mpsc::Sender<Result<DiscoveryResponse, Status>>,
    type_url: &str,
    cache: Arc<Cache>,
) {
    info!("new stream for node");

    let mut nonce: i64 = 0;
    let mut known_resource_names = HashMap::new();
    let mut watches = FuturesUnordered::new();

    loop {
        tokio::select! {
            request = stream.next() => {
                info!("receiving request");
                let req = request.unwrap().unwrap();
                info!("version_info: {:?}", req.version_info);
                info!("resource_names: {:?}", req.resource_names);
                info!("type_url: {:?}", req.type_url);
                info!("response_nonce: {:?}", req.response_nonce);
                let (tx, rx) = oneshot::channel();
                cache.create_watch(&req, tx, &known_resource_names);
                watches.push(rx);
            }
            Some(response) = watches.next() => {
                info!("sending response");
                let mut rep = response.unwrap();
                nonce += 1;
                rep.nonce = nonce.to_string();
                tx.send(Ok(rep)).await.unwrap();
            }
        }
    }
}
