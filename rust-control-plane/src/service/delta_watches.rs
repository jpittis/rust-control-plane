use crate::cache::{Cache, WatchId};
use std::collections::HashMap;
use std::sync::Arc;

pub struct DeltaWatches<C: Cache> {
    cache: Arc<C>,
    active: HashMap<String, Watch>,
}

#[derive(Clone)]
pub struct Watch {
    pub id: WatchId,
}

impl<C: Cache> DeltaWatches<C> {
    pub fn new(cache: Arc<C>) -> Self {
        Self {
            cache,
            active: HashMap::new(),
        }
    }

    pub fn add(&mut self, type_url: &str, watch_id: WatchId) {
        self.active
            .insert(type_url.to_string(), Watch { id: watch_id });
    }

    pub fn remove(&mut self, type_url: &str) -> Option<Watch> {
        self.active.remove(type_url)
    }
}

pub async fn cancel_all<C: Cache>(active: HashMap<String, Watch>, cache: Arc<C>) {
    for (_, watch) in active.iter() {
        cache.cancel_watch(&watch.id).await;
    }
}

impl<C: Cache> Drop for DeltaWatches<C> {
    fn drop(&mut self) {
        tokio::spawn(cancel_all(self.active.clone(), self.cache.clone()));
    }
}
