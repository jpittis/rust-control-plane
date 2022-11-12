use crate::cache::Cache;
use crate::snapshot::type_url::ANY_TYPE;
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
    let mut node = None;

    loop {
        tokio::select! {
            request = stream.next() => {
                let mut req = request.unwrap().unwrap();
                info!("received request version={:?} type={:?} resources={:?} nonce={:?}",
                      req.version_info, &req.type_url[20..], req.resource_names, req.response_nonce);

                // Node might only be sent on the first request to save sending the same data
                // repeatedly, so let's cache it in memory for future requests on this stream.
                if req.node.is_some() {
                    node = req.node.clone();
                } else {
                    req.node = node.clone();
                }

                if type_url == ANY_TYPE && req.type_url == "" {
                    // Type URL is required for ADS (ANY_TYPE) because we can't tell from just the
                    // gRPC method which resource this request is for.
                    respond_error(tx, Status::invalid_argument("type URL is required for ADS")).await;
                    return;
                } else if req.type_url == "" {
                    // Type URL is otherwise optional, but let's set it for consistency.
                    req.type_url = type_url.to_string();
                }

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

async fn respond_error(tx: mpsc::Sender<Result<DiscoveryResponse, Status>>, status: Status) {
    tx.send(Err(status)).await.unwrap();
}
