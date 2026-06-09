// Complex fixture: 6 entities — Document + impls, Parser trait, MarkdownParser + impls, Error enum + impl, Cache + impl, format_html fn

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

pub trait Parser {
    type Output;
    fn parse(&self, input: &str) -> Result<Self::Output, Error>;
    fn name(&self) -> &str;
}

pub enum Error {
    Syntax(String),
    Io(String),
    NotFound(String),
}

impl Error {
    pub fn message(&self) -> &str {
        match self {
            Error::Syntax(msg) => msg,
            Error::Io(msg) => msg,
            Error::NotFound(msg) => msg,
        }
    }

    pub fn kind(&self) -> &str {
        match self {
            Error::Syntax(_) => "syntax",
            Error::Io(_) => "io",
            Error::NotFound(_) => "not_found",
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind(), self.message())
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error::{}({})", self.kind(), self.message())
    }
}

pub struct MarkdownParser;

impl Parser for MarkdownParser {
    type Output = Document;

    fn parse(&self, input: &str) -> Result<Document, Error> {
        let title = input
            .lines()
            .find(|l| l.starts_with("# "))
            .map(|l| l["# ".len()..].to_string())
            .unwrap_or_default();
        Ok(Document::new(&title, input))
    }

    fn name(&self) -> &str {
        "markdown"
    }
}

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

pub fn format_html(doc: &Document) -> String {
    format!(
        "<h1>{}</h1>\n<div>{}</div>",
        doc.title, doc.content
    )
}
