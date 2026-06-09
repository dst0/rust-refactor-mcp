use crate::document::Document;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Cache {
    entries: HashMap<String, Document>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: &str, doc: Document) {
        self.entries.insert(key.to_string(), doc);
    }

    pub fn get(&self, key: &str) -> Option<&Document> {
        self.entries.get(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<Document> {
        self.entries.remove(key)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

