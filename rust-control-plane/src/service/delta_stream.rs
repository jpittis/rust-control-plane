use crate::cache::{Cache, DeltaWatchResponse};
use crate::service::delta_watches::DeltaWatches;
use crate::service::stream_handle::DeltaStreamHandle;
use crate::snapshot::type_url::{self, ANY_TYPE};
use data_plane_api::envoy::config::core::v3::Node;
use data_plane_api::envoy::service::discovery::v3::{
    DeltaDiscoveryRequest, DeltaDiscoveryResponse,
};
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tonic::{Status, Streaming};
use tracing::info;
use tracing::{info_span, Instrument};

pub async fn handle_delta_stream<C: Cache>(
    mut requests: Streaming<DeltaDiscoveryRequest>,
    responses: mpsc::Sender<Result<DeltaDiscoveryResponse, Status>>,
    type_url: &'static str,
    cache: Arc<C>,
) {
    let mut stream = DeltaStream::new(responses, type_url, cache);
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

struct DeltaStream<C: Cache> {
    responses: mpsc::Sender<Result<DeltaDiscoveryResponse, Status>>,
    type_url: &'static str,
    cache: Arc<C>,
    nonce: i64,
    node: Option<Node>,
    states: HashMap<String, DeltaStreamHandle>,
    watches_tx: mpsc::Sender<DeltaWatchResponse>,
    watches_rx: mpsc::Receiver<DeltaWatchResponse>,
    watches: DeltaWatches<C>,
}

impl<C: Cache> DeltaStream<C> {
    pub fn new(
        responses: mpsc::Sender<Result<DeltaDiscoveryResponse, Status>>,
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
            node: None,
            states: HashMap::new(),
            watches_tx,
            watches_rx,
            watches: DeltaWatches::new(cache_clone),
        }
    }

    async fn handle_client_request(&mut self, mut req: DeltaDiscoveryRequest) {
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

        let state = self
            .states
            .entry(req.type_url.to_string())
            .or_insert_with(|| DeltaStreamHandle::new(&req));
        self.watches.remove(&req.type_url);
        if let Some(watch) = self.watches.remove(&req.type_url) {
            self.cache.cancel_watch(&watch.id).await;
        }
        state.apply_subscriptions(&req);
        let watch_id = self
            .cache
            .create_delta_watch(&req, self.watches_tx.clone(), state)
            .await;
        if let Some(id) = watch_id {
            self.watches.add(&req.type_url, id);
        }
    }

    async fn handle_watch_response(&mut self, mut rep: DeltaWatchResponse) {
        self.nonce += 1;
        rep.0.nonce = self.nonce.to_string();
        self.states
            .get_mut(&rep.0.type_url)
            .unwrap()
            .set_resource_versions(rep.1);
        self.responses.send(Ok(rep.0)).await.unwrap();
    }

    fn build_client_request_span(&self, req: &DeltaDiscoveryRequest) -> tracing::Span {
        info_span!(
            "handle_client_request",
            type_url = type_url::shorten(&req.type_url),
            response_nonce = req.response_nonce,
        )
    }
}
