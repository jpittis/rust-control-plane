use super::*;
use crate::cache::{Cache, FetchError, KnownResourceNames, WatchId, WatchResponder};
use crate::snapshot::type_url::CLUSTER;
use async_trait::async_trait;
use tokio::sync::Mutex;

struct MockCache {
    pub inner: Mutex<InnerMockCache>,
}

struct InnerMockCache {
    pub create_watch_calls: Vec<(DiscoveryRequest, WatchResponder, KnownResourceNames)>,
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

#[tokio::test]
async fn test_stream_stores_node_for_future_requests() {
    let (tx, _) = mpsc::channel(1);
    let cache = Arc::new(MockCache::new());
    let req_with_node = DiscoveryRequest {
        node: Some(Node {
            id: "foobar".to_string(),
            ..Node::default()
        }),
        ..DiscoveryRequest::default()
    };
    let req_without_node = DiscoveryRequest {
        ..DiscoveryRequest::default()
    };
    let mut stream = Stream::new(tx, CLUSTER, cache.clone());
    stream.handle_client_request(req_with_node).await;
    stream.handle_client_request(req_without_node).await;
    assert_eq!(
        cache
            .inner
            .lock()
            .await
            .create_watch_calls
            .iter()
            .map(|(req, _, _)| req.node.clone())
            .collect::<Vec<Option<Node>>>(),
        vec![
            Some(Node {
                id: "foobar".to_string(),
                ..Node::default()
            }),
            Some(Node {
                id: "foobar".to_string(),
                ..Node::default()
            }),
        ]
    );
}
