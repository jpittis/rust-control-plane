use crate::cache::Cache;
use crate::snapshot::type_url::{self, ANY_TYPE};
use data_plane_api::envoy::config::core::v3::Node;
use data_plane_api::envoy::service::discovery::v3::{
    DeltaDiscoveryRequest, DeltaDiscoveryResponse,
};
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::mpsc;
use tonic::{Status, Streaming};
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
        }
    }
}

struct DeltaStream<C: Cache> {
    responses: mpsc::Sender<Result<DeltaDiscoveryResponse, Status>>,
    type_url: &'static str,
    cache: Arc<C>,
    nonce: i64,
    node: Option<Node>,
}

impl<C: Cache> DeltaStream<C> {
    pub fn new(
        responses: mpsc::Sender<Result<DeltaDiscoveryResponse, Status>>,
        type_url: &'static str,
        cache: Arc<C>,
    ) -> Self {
        Self {
            responses,
            type_url,
            cache,
            nonce: 0,
            node: None,
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
    }

    fn build_client_request_span(&self, req: &DeltaDiscoveryRequest) -> tracing::Span {
        info_span!(
            "handle_client_request",
            type_url = type_url::shorten(&req.type_url),
            response_nonce = req.response_nonce,
        )
    }
}
