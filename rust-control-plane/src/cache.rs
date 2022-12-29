pub mod snapshot;

use crate::service::stream_handle::{DeltaStreamHandle, StreamHandle};
use async_trait::async_trait;
use data_plane_api::envoy::service::discovery::v3::{
    DeltaDiscoveryRequest, DeltaDiscoveryResponse, DiscoveryRequest, DiscoveryResponse,
};
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct WatchId {
    pub node_id: String,
    pub index: usize,
}

pub type KnownResourceNames = HashMap<String, HashSet<String>>;

pub type WatchResponse = (DiscoveryRequest, DiscoveryResponse);

pub type WatchResponder = mpsc::Sender<WatchResponse>;

pub type NextVersionMap = HashMap<String, String>;

pub type DeltaWatchResponse = (DeltaDiscoveryResponse, NextVersionMap);

pub type DeltaWatchResponder = mpsc::Sender<DeltaWatchResponse>;

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
        stream: &StreamHandle,
    ) -> Option<WatchId>;
    async fn create_delta_watch(
        &self,
        req: &DeltaDiscoveryRequest,
        tx: DeltaWatchResponder,
        stream: &DeltaStreamHandle,
    ) -> Option<WatchId>;
    async fn cancel_watch(&self, watch_id: &WatchId);
    async fn cancel_delta_watch(&self, watch_id: &WatchId);
    async fn fetch<'a>(
        &'a self,
        req: &'a DiscoveryRequest,
        type_url: &'static str,
    ) -> Result<DiscoveryResponse, FetchError>;
}
