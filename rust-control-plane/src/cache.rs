pub mod snapshot;

use async_trait::async_trait;
use data_plane_api::envoy::service::discovery::v3::{DiscoveryRequest, DiscoveryResponse};
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc;

#[derive(Clone, Debug)]
pub struct WatchId {
    node_id: String,
    index: usize,
}

pub type WatchResponder = mpsc::Sender<(DiscoveryRequest, DiscoveryResponse)>;

pub enum FetchError {
    VersionUpToDate,
    NotFound,
}

#[async_trait]
pub trait Cache: Sync + Send + 'static {
    async fn create_watch(
        &self,
        req: &DiscoveryRequest,
        tx: WatchResponder,
        known_resource_names: &HashMap<String, HashSet<String>>,
    ) -> Option<WatchId>;
    async fn cancel_watch(&self, watch_id: &WatchId);
    async fn fetch<'a>(
        &'a self,
        req: &'a DiscoveryRequest,
        type_url: &'static str,
    ) -> Result<DiscoveryResponse, FetchError>;
}
