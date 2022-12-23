#[cfg(test)]
mod test;

use super::watches::Watches;
use crate::cache::{Cache, WatchResponse};
use crate::snapshot::type_url::{self, ANY_TYPE};
use data_plane_api::envoy::config::core::v3::Node;
use data_plane_api::envoy::service::discovery::v3::{DiscoveryRequest, DiscoveryResponse};
use futures::StreamExt;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::mpsc;
use tonic::{Status, Streaming};
use tracing::{info_span, Instrument};

pub async fn handle_stream<C: Cache>(
    mut requests: Streaming<DiscoveryRequest>,
    responses: mpsc::Sender<Result<DiscoveryResponse, Status>>,
    type_url: &'static str,
    cache: Arc<C>,
) {
    let mut stream = Stream::new(responses, type_url, cache);
    loop {
        tokio::select! {
            maybe_req = requests.next() => {
                let req = maybe_req.unwrap().unwrap();
                let span = stream.build_client_request_span(&req);
                stream.handle_client_request(req).instrument(span).await;
            }
            Some(rep) = stream.watches_rx.recv() => {
                stream.handle_watch_response(rep)
                    .instrument(info_span!("handle_watch_response")).await;
            }
        }
    }
}

struct LastResponse {
    nonce: i64,
    resource_names: Vec<String>,
}

struct Stream<C: Cache> {
    responses: mpsc::Sender<Result<DiscoveryResponse, Status>>,
    type_url: &'static str,
    cache: Arc<C>,
    nonce: i64,
    known_resource_names: HashMap<String, HashSet<String>>,
    watches_tx: mpsc::Sender<WatchResponse>,
    watches_rx: mpsc::Receiver<WatchResponse>,
    node: Option<Node>,
    last_responses: HashMap<String, LastResponse>,
    watches: Watches<C>,
}

impl<C: Cache> Stream<C> {
    pub fn new(
        responses: mpsc::Sender<Result<DiscoveryResponse, Status>>,
        type_url: &'static str,
        cache: Arc<C>,
    ) -> Self {
        let (watches_tx, watches_rx) = mpsc::channel(16);
        let cache_clone = cache.clone();
        Self {
            responses,
            type_url,
            cache,
            nonce: 0,
            known_resource_names: HashMap::new(),
            watches_tx,
            watches_rx,
            node: None,
            last_responses: HashMap::new(),
            watches: Watches::new(cache_clone),
        }
    }

    async fn handle_client_request(&mut self, mut req: DiscoveryRequest) {
        // Node might only be sent on the first request to save sending the same data
        // repeatedly, so let's cache it in memory for future requests on this stream.
        // NB: If client changes the node after the first request (that's a client bug), we've
        // chosen to forward that new one to avoid complexity in this algorithm.
        if req.node.is_some() {
            self.node = req.node.clone();
        } else {
            req.node = self.node.clone();
        }

        if self.type_url == ANY_TYPE && req.type_url.is_empty() {
            // Type URL is required for ADS (ANY_TYPE) because we can't tell from just the
            // gRPC method which resource this request is for.
            let status = Status::invalid_argument("type URL is required for ADS");
            self.responses.send(Err(status)).await.unwrap();
            return;
        } else if req.type_url.is_empty() {
            // Type URL is otherwise optional, but let's set it for consistency.
            // NB: We don't currently validate the type_url, or check if it's for the right RPC.
            req.type_url = self.type_url.to_string();
        }

        // If this is an ack of a previous response, record that the client has received
        // the resource names for that response.
        if let Some(last_response) = self.last_responses.get(&req.type_url) {
            if last_response.nonce == 0 || last_response.nonce == self.nonce {
                let entry = self.known_resource_names.entry(req.type_url.clone());
                entry
                    .and_modify(|entry| {
                        last_response.resource_names.iter().for_each(|name| {
                            entry.insert(name.clone());
                        })
                    })
                    .or_insert_with(|| {
                        let mut entry = HashSet::new();
                        last_response.resource_names.iter().for_each(|name| {
                            entry.insert(name.clone());
                        });
                        entry
                    });
            }
        }

        let mut watch_id = None;
        if let Some(watch) = self.watches.get(&req.type_url) {
            // A watch already exists so we need to replace it if this is a valid ack.
            if watch.nonce.is_none() || watch.nonce == Some(self.nonce) {
                self.cache.cancel_watch(&watch.id).await;
                watch_id = self
                    .cache
                    .create_watch(&req, self.watches_tx.clone(), &self.known_resource_names)
                    .instrument(info_span!("create_watch"))
                    .await;
            }
        } else {
            // No watch exists yet so we can just create one.
            watch_id = self
                .cache
                .create_watch(&req, self.watches_tx.clone(), &self.known_resource_names)
                .instrument(info_span!("create_watch"))
                .await;
        }
        if let Some(id) = watch_id {
            self.watches.add(&req.type_url, id);
        }
    }

    async fn handle_watch_response(&mut self, mut rep: WatchResponse) {
        self.nonce += 1;
        rep.1.nonce = self.nonce.to_string();
        let last_response = LastResponse {
            nonce: self.nonce,
            resource_names: rep.0.resource_names,
        };
        self.last_responses
            .insert(rep.0.type_url.clone(), last_response);
        self.responses.send(Ok(rep.1)).await.unwrap();
        if let Some(watch) = self.watches.get_mut(&rep.0.type_url) {
            watch.nonce = Some(self.nonce)
        }
    }

    fn build_client_request_span(&self, req: &DiscoveryRequest) -> tracing::Span {
        info_span!(
            "handle_client_request",
            version_info = req.version_info,
            type_url = type_url::shorten(&req.type_url),
            response_nonce = req.response_nonce,
            resource_names = ?req.resource_names,
        )
    }
}
