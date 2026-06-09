use crate::extractresult::ByteSpan;
use crate::line_col_to_byte::line_col_to_byte;
pub fn span_to_byte(span: &proc_macro2::Span, source: &str) -> ByteSpan {
    let start = span.start();
    let end = span.end();
    ByteSpan::new(
        line_col_to_byte(source, start.line, start.column),
        line_col_to_byte(source, end.line, end.column),
    )
}
