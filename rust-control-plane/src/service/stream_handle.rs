use std::collections::{HashMap, HashSet};

pub struct StreamHandle {
    known_resource_names: HashMap<String, HashSet<String>>,
}

impl StreamHandle {
    pub fn new() -> Self {
        Self {
            known_resource_names: HashMap::new(),
        }
    }

    pub fn known_resource_names(&self, type_url: &str) -> Option<&HashSet<String>> {
        self.known_resource_names.get(type_url)
    }

    pub fn add_known_resource_names(&mut self, type_url: &str, names: &[String]) {
        self.known_resource_names
            .entry(type_url.to_string())
            .and_modify(|entry| {
                names.iter().for_each(|name| {
                    entry.insert(name.clone());
                })
            })
            .or_insert_with(|| {
                let mut entry = HashSet::new();
                names.iter().for_each(|name| {
                    entry.insert(name.clone());
                });
                entry
            });
    }
}

impl Default for StreamHandle {
    fn default() -> Self {
        Self::new()
    }
}
