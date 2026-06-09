// Usage file 2: references Document, Error, Cache, format_html from complex.rs

use crate::complex::{Document, Error, Cache, format_html};

pub fn cache_docs(docs: Vec<Document>) -> Cache {
    let mut cache = Cache::new();
    for doc in docs {
        cache.insert(&doc.title, doc);
    }
    println!("Cached {} documents", cache.len());
    cache
}

pub fn lookup(cache: &Cache, title: &str) -> Result<String, Error> {
    match cache.get(title) {
        Some(doc) => Ok(format_html(doc)),
        None => Err(Error::NotFound(format!("No document: {}", title))),
    }
}

pub fn handle_error(err: &Error) {
    match err.kind() {
        "syntax" => println!("Syntax issue: {}", err.message()),
        "io" => println!("IO issue: {}", err.message()),
        "not_found" => println!("Not found: {}", err.message()),
        _ => println!("Unknown error: {}", err),
    }
}

pub fn doc_factory(title: &str) -> Document {
    Document::new(title, "content here")
}
