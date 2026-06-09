use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct ByteSpan {
    pub start: usize,
    pub end: usize,
}

impl ByteSpan {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Serialize)]
pub struct ExtractResult {
    pub new_file_path: String,
    pub test_file_path: Option<String>,
    pub items_extracted: Vec<String>,
    pub usage_files_updated: Vec<String>,
    pub extracted_spans: Vec<ByteSpan>,
}
