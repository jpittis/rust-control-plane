use crate::cache::{Cache, FetchError};
use crate::service::stream::handle_stream;
use data_plane_api::envoy::service::discovery::v3::{DiscoveryRequest, DiscoveryResponse};
use futures::Stream;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response};
use tonic::{Status, Streaming};

#[derive(Debug)]
pub struct Service {
    pub cache: Arc<Cache>,
}

pub type StreamResponse<T> = Pin<Box<dyn Stream<Item = Result<T, Status>> + Send + 'static>>;

impl Service {
    pub fn new(cache: Arc<Cache>) -> Self {
        Self { cache }
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
        tokio::spawn(async move { handle_stream(input, tx, type_url, cache_clone).await });
        Ok(Response::new(
            Box::pin(output) as StreamResponse<DiscoveryResponse>
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
