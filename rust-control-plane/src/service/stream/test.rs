use super::*;
use crate::cache::{Cache, FetchError, KnownResourceNames, WatchId, WatchResponder};
use crate::snapshot::type_url::CLUSTER;
use async_trait::async_trait;
use tokio::sync::Mutex;
use tonic::Code;

struct MockCache {
    pub inner: Mutex<InnerMockCache>,
}

type CreateWatchCall = (DiscoveryRequest, WatchResponder, KnownResourceNames);

struct InnerMockCache {
    pub create_watch_calls: Vec<CreateWatchCall>,
}

#[async_trait]
impl Cache for MockCache {
    async fn create_watch(
        &self,
        req: &DiscoveryRequest,
        tx: WatchResponder,
        known_resource_names: &KnownResourceNames,
    ) -> Option<WatchId> {
        self.inner.lock().await.create_watch_calls.push((
            req.clone(),
            tx,
            known_resource_names.clone(),
        ));
        None
    }

    async fn cancel_watch(&self, _watch_id: &WatchId) {
        unimplemented!()
    }

    async fn fetch<'a>(
        &'a self,
        _req: &'a DiscoveryRequest,
        _type_url: &'static str,
    ) -> Result<DiscoveryResponse, FetchError> {
        unimplemented!()
    }
}

impl MockCache {
    fn new() -> Self {
        Self {
            inner: Mutex::new(InnerMockCache::new()),
        }
    }
}

impl InnerMockCache {
    fn new() -> Self {
        Self {
            create_watch_calls: Vec::new(),
        }
    }
}

struct TestHandle {
    pub rx: mpsc::Receiver<Result<DiscoveryResponse, Status>>,
    pub stream: Stream<MockCache>,
    pub cache: Arc<MockCache>,
}

impl TestHandle {
    async fn create_watch_calls(&self) -> Vec<CreateWatchCall> {
        self.cache.inner.lock().await.create_watch_calls.clone()
    }
}

impl TestHandle {
    fn new(type_url: &'static str) -> Self {
        let (tx, rx) = mpsc::channel(1);
        let cache = Arc::new(MockCache::new());
        let stream = Stream::new(tx, type_url, cache.clone());
        Self { rx, stream, cache }
    }
}

#[tokio::test]
async fn test_stream_stores_node_for_future_requests() {
    let mut h = TestHandle::new(CLUSTER);
    let req_with_node = DiscoveryRequest {
        node: Some(Node {
            id: "foobar".to_string(),
            ..Node::default()
        }),
        type_url: CLUSTER.to_string(),
        ..DiscoveryRequest::default()
    };
    let req_without_node = DiscoveryRequest {
        type_url: CLUSTER.to_string(),
        ..DiscoveryRequest::default()
    };
    h.stream.handle_client_request(req_with_node.clone()).await;
    h.stream.handle_client_request(req_without_node).await;
    let calls = h.create_watch_calls().await;
    assert_eq!(calls.len(), 2);
    for (req, _, _) in calls {
        assert_eq!(req, req_with_node);
    }
}

#[tokio::test]
async fn test_stream_forwards_type_url_if_not_present() {
    let mut h = TestHandle::new(CLUSTER);
    let req_without_type_url = DiscoveryRequest {
        node: Some(Node {
            id: "foobar".to_string(),
            ..Node::default()
        }),
        ..DiscoveryRequest::default()
    };
    h.stream.handle_client_request(req_without_type_url).await;
    let calls = h.create_watch_calls().await;
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0.type_url, CLUSTER);
}

#[tokio::test]
async fn test_stream_aborts_if_type_url_not_present_for_ads() {
    let mut h = TestHandle::new(ANY_TYPE);
    let req_without_type_url = DiscoveryRequest {
        node: Some(Node {
            id: "foobar".to_string(),
            ..Node::default()
        }),
        ..DiscoveryRequest::default()
    };
    h.stream.handle_client_request(req_without_type_url).await;
    let calls = h.create_watch_calls().await;
    assert_eq!(calls.len(), 0);
    let status = h.rx.try_recv().unwrap().unwrap_err();
    assert_eq!(status.code(), Code::InvalidArgument);
    assert_eq!(status.message(), "type URL is required for ADS");
}
