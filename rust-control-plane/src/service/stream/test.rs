use super::*;
use crate::cache::{Cache, FetchError, KnownResourceNames, WatchId, WatchResponder};
use crate::snapshot::type_url::{ANY_TYPE, CLUSTER, ENDPOINT};
use async_trait::async_trait;
use tokio::sync::Mutex;
use tonic::Code;

struct MockCache {
    pub inner: Mutex<InnerMockCache>,
}

type CreateWatchCall = (DiscoveryRequest, WatchResponder, KnownResourceNames);

struct InnerMockCache {
    pub create_watch_calls: Vec<CreateWatchCall>,
    pub create_watch_rep: Option<WatchId>,
    pub cancel_watch_calls: Vec<WatchId>,
}

#[async_trait]
impl Cache for MockCache {
    async fn create_watch(
        &self,
        req: &DiscoveryRequest,
        tx: WatchResponder,
        known_resource_names: &KnownResourceNames,
    ) -> Option<WatchId> {
        let mut inner = self.inner.lock().await;
        inner
            .create_watch_calls
            .push((req.clone(), tx, known_resource_names.clone()));
        inner.create_watch_rep.clone()
    }

    async fn cancel_watch(&self, watch_id: &WatchId) {
        let mut inner = self.inner.lock().await;
        inner.cancel_watch_calls.push(watch_id.clone());
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
            create_watch_rep: None,
            cancel_watch_calls: Vec::new(),
        }
    }
}

struct TestHandle {
    pub rx: mpsc::Receiver<Result<DiscoveryResponse, Status>>,
    pub stream: Stream<MockCache>,
    pub cache: Arc<MockCache>,
    type_url: &'static str,
}

impl TestHandle {
    fn new(type_url: &'static str) -> Self {
        let (tx, rx) = mpsc::channel(1);
        let cache = Arc::new(MockCache::new());
        let stream = Stream::new(tx, type_url, cache.clone());
        Self {
            rx,
            stream,
            cache,
            type_url,
        }
    }

    fn reconnect(&mut self) {
        let (tx, rx) = mpsc::channel(1);
        let mut stream = Stream::new(tx, self.type_url, self.cache.clone());
        self.rx = rx;
        std::mem::swap(&mut self.stream, &mut stream);
        drop(stream);
    }

    async fn create_watch_calls(&self) -> Vec<CreateWatchCall> {
        self.cache.inner.lock().await.create_watch_calls.clone()
    }

    async fn set_create_watch_rep(&self, rep: Option<WatchId>) {
        self.cache.inner.lock().await.create_watch_rep = rep;
    }

    async fn cancel_watch_calls(&self) -> Vec<WatchId> {
        self.cache.inner.lock().await.cancel_watch_calls.clone()
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

#[tokio::test]
async fn test_stream_cancels_watches_on_drop() {
    let mut h = TestHandle::new(ANY_TYPE);
    let req1 = DiscoveryRequest {
        node: Some(Node {
            id: "foobar".to_string(),
            ..Node::default()
        }),
        type_url: CLUSTER.to_string(),
        ..DiscoveryRequest::default()
    };
    let req2 = DiscoveryRequest {
        node: Some(Node {
            id: "foobar".to_string(),
            ..Node::default()
        }),
        type_url: ENDPOINT.to_string(),
        ..DiscoveryRequest::default()
    };
    let watch1 = WatchId {
        node_id: "foobar".to_string(),
        index: 0,
    };
    h.set_create_watch_rep(Some(watch1.clone())).await;
    h.stream.handle_client_request(req1).await;
    let watch2 = WatchId {
        node_id: "foobar".to_string(),
        index: 1,
    };
    h.set_create_watch_rep(Some(watch2.clone())).await;
    h.stream.handle_client_request(req2).await;
    h.reconnect();
    // NB: I don't know how else we can wait for the task spawned by drop to complete.
    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
    let create_calls = h.create_watch_calls().await;
    assert_eq!(create_calls.len(), 2);
    let mut cancel_calls = h.cancel_watch_calls().await;
    cancel_calls.sort();
    assert_eq!(cancel_calls.len(), 2);
    assert_eq!(cancel_calls, vec![watch1, watch2]);
}
