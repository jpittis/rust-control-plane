use crate::cache::{Cache, FetchError};
use crate::service::delta_stream::handle_delta_stream;
use crate::service::stream::handle_stream;
use crate::snapshot::type_url;
use data_plane_api::envoy::service::discovery::v3::{
    DeltaDiscoveryRequest, DeltaDiscoveryResponse, DiscoveryRequest, DiscoveryResponse,
};
use futures::Stream;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response};
use tonic::{Status, Streaming};
use tracing::{info_span, Instrument};

#[derive(Debug)]
pub struct Service<C: Cache> {
    cache: Arc<C>,
    next_stream_id: AtomicUsize,
}

pub type StreamResponse<T> = Pin<Box<dyn Stream<Item = Result<T, Status>> + Send + 'static>>;

impl<C: Cache> Service<C> {
    pub fn new(cache: Arc<C>) -> Self {
        Self {
            cache,
            next_stream_id: AtomicUsize::new(0),
        }
    }

    pub fn stream(
        &self,
        req: Request<Streaming<DiscoveryRequest>>,
        type_url: &'static str,
    ) -> Result<Response<StreamResponse<DiscoveryResponse>>, Status> {
        let input = req.into_inner();
        let (tx, rx) = mpsc::channel(1);
        let output = ReceiverStream::new(rx);
        let cache_clone = self.cache.clone();
        let stream_id = self.next_stream_id.fetch_add(1, Ordering::SeqCst);

        tokio::spawn(
            async move { handle_stream(input, tx, type_url, cache_clone).await }.instrument(
                info_span!(
                    "handle_stream",
                    stream_id,
                    type_url = type_url::shorten(type_url),
                ),
            ),
        );

        Ok(Response::new(
            Box::pin(output) as StreamResponse<DiscoveryResponse>
        ))
    }

    pub fn delta_stream(
        &self,
        req: Request<Streaming<DeltaDiscoveryRequest>>,
        type_url: &'static str,
    ) -> Result<Response<StreamResponse<DeltaDiscoveryResponse>>, Status> {
        let input = req.into_inner();
        let (tx, rx) = mpsc::channel(1);
        let output = ReceiverStream::new(rx);
        let cache_clone = self.cache.clone();
        let stream_id = self.next_stream_id.fetch_add(1, Ordering::SeqCst);

        tokio::spawn(
            async move { handle_delta_stream(input, tx, type_url, cache_clone).await }.instrument(
                info_span!(
                    "handle_delta_stream",
                    stream_id,
                    type_url = type_url::shorten(type_url),
                ),
            ),
        );

        Ok(Response::new(
            Box::pin(output) as StreamResponse<DeltaDiscoveryResponse>
        ))
    }

    pub async fn fetch(
        &self,
        req: &DiscoveryRequest,
        type_url: &'static str,
    ) -> Result<Response<DiscoveryResponse>, Status> {
        match self.cache.fetch(req, type_url).await {
            Ok(resp) => Ok(Response::new(resp)),
            Err(FetchError::NotFound) => Err(Status::not_found("Resource not found for node")),
            Err(FetchError::VersionUpToDate) => {
                Err(Status::already_exists("Version already up to date"))
            }
        }
    }
}
