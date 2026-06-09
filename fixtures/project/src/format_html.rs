use crate::document::Document;

pub fn format_html(doc: &Document) -> String {
    format!(
        "<h1>{}</h1>\n<div>{}</div>",
        doc.title, doc.content
    )
}

