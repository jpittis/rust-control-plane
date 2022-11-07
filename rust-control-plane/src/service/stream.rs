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
    let mut nonce: i64 = 0;
    let mut known_resource_names = HashMap::new();
    let mut watches = FuturesUnordered::new();

    loop {
        tokio::select! {
            request = stream.next() => {
                let req = request.unwrap().unwrap();
                info!("received request version={:?} type={:?} resources={:?} nonce={:?}",
                      req.version_info, &req.type_url[20..], req.resource_names, req.response_nonce);
                let (tx, rx) = oneshot::channel();
                cache.create_watch(&req, tx, &known_resource_names);
                watches.push(rx);
            }
            Some(response) = watches.next() => {
                let mut rep = response.unwrap();
                nonce += 1;
                rep.nonce = nonce.to_string();
                tx.send(Ok(rep)).await.unwrap();
            }
        }
    }
}
