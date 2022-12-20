use crate::cache::{Cache, WatchId};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct Watch {
    pub nonce: Option<i64>,
    pub id: WatchId,
}

pub struct Watches<C: Cache> {
    cache: Arc<C>,
    active: HashMap<String, Watch>,
}

impl<C: Cache> Watches<C> {
    pub fn new(cache: Arc<C>) -> Self {
        Self {
            cache,
            active: HashMap::new(),
        }
    }

    pub fn get(&self, type_url: &str) -> Option<&Watch> {
        self.active.get(type_url)
    }

    pub fn get_mut(&mut self, type_url: &str) -> Option<&mut Watch> {
        self.active.get_mut(type_url)
    }

    pub fn add(&mut self, type_url: &str, watch_id: WatchId) {
        self.active.insert(
            type_url.to_string(),
            Watch {
                nonce: None,
                id: watch_id,
            },
        );
    }
}

pub async fn cancel_all<C: Cache>(active: HashMap<String, Watch>, cache: Arc<C>) {
    for (_, watch) in active.iter() {
        cache.cancel_watch(&watch.id).await;
    }
}

impl<C: Cache> Drop for Watches<C> {
    fn drop(&mut self) {
        tokio::spawn(cancel_all(self.active.clone(), self.cache.clone()));
    }
}
