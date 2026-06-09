// Usage file 1: references Document, Parser, MarkdownParser from complex.rs

use crate::complex::{Document, Parser, MarkdownParser};

pub fn render(parsed: &Document) -> String {
    println!("Rendering: {}", parsed);
    format!("# {}\n\n{}", parsed.title, parsed.content)
}

pub fn process(input: &str) -> Result<Document, crate::complex::Error> {
    let parser = MarkdownParser;
    println!("Using parser: {}", parser.name());
    parser.parse(input)
}

pub fn summarize(docs: &[Document]) -> String {
    docs.iter()
        .map(|d| format!("- {} ({} words)", d.title, d.word_count()))
        .collect::<Vec<_>>()
        .join("\n")
}
