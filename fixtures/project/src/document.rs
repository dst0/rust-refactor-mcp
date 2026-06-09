use std::fmt;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Document {
    pub title: String,
    pub content: String,
    pub metadata: HashMap<String, String>,
}

impl Document {
    pub fn new(title: &str, content: &str) -> Self {
        Self {
            title: title.to_string(),
            content: content.to_string(),
            metadata: HashMap::new(),
        }
    }

    pub fn word_count(&self) -> usize {
        self.content.split_whitespace().count()
    }

    pub fn set_meta(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }
}

impl fmt::Display for Document {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Document: {} ({} words)", self.title, self.word_count())
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new("Untitled", "")
    }
}

