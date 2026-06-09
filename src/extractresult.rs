use crate::bytespan::ByteSpan;
#[derive(Debug)]
pub struct ExtractResult {
    pub new_file_path: String,
    pub test_file_path: Option<String>,
    pub items_extracted: Vec<String>,
    pub usage_files_updated: Vec<String>,
    pub extracted_spans: Vec<ByteSpan>,
}
