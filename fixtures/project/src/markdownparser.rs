use crate::error::Error;
use crate::document::Document;

use crate::parser::Parser;
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

