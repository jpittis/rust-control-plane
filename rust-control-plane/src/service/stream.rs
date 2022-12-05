use crate::cache::Cache;
use crate::snapshot::type_url::ANY_TYPE;
use data_plane_api::envoy::service::discovery::v3::{DiscoveryRequest, DiscoveryResponse};
use futures::StreamExt;
use log::info;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::mpsc;
use tonic::{Status, Streaming};

struct LastResponse {
    nonce: i64,
    resource_names: Vec<String>,
}

pub async fn handle_stream(
    mut stream: Streaming<DiscoveryRequest>,
    tx: mpsc::Sender<Result<DiscoveryResponse, Status>>,
    type_url: &str,
    cache: Arc<Cache>,
) {
    let mut nonce: i64 = 0;
    let mut known_resource_names: HashMap<String, HashSet<String>> = HashMap::new();
    let (watches_tx, mut watches_rx) = mpsc::channel(16);
    let mut node = None;
    let mut last_responses: HashMap<String, LastResponse> = HashMap::new();

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
                    error(tx, Status::invalid_argument("type URL is required for ADS")).await;
                    return;
                } else if req.type_url == "" {
                    // Type URL is otherwise optional, but let's set it for consistency.
                    req.type_url = type_url.to_string();
                }

                // If this is an ack of a previous response, record that the client has received
                // the resource names for that response.
                if let Some(last_response) = last_responses.get(&req.type_url) {
                    if last_response.nonce == 0 || last_response.nonce == nonce {
                        let entry = known_resource_names.entry(req.type_url.clone());
                        entry
                            .and_modify(|entry| {
                                last_response.resource_names.iter().for_each(|name| {
                                    entry.insert(name.clone());
                                })
                            })
                            .or_insert_with(|| HashSet::new());
                    }
                }

                cache.create_watch(&req, watches_tx.clone(), &known_resource_names).await;
            }
            Some(mut rep) = watches_rx.recv() => {
                nonce += 1;
                rep.1.nonce = nonce.to_string();
                let last_response = LastResponse{
                    nonce,
                    resource_names: rep.0.resource_names,
                };
                last_responses.insert(rep.0.type_url, last_response);
                tx.send(Ok(rep.1)).await.unwrap();
            }
        }
    }
}

async fn error(tx: mpsc::Sender<Result<DiscoveryResponse, Status>>, status: Status) {
    tx.send(Err(status)).await.unwrap();
}
