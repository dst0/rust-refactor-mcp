use crate::bytespan::ByteSpan;
/// Remove multiple sorted, non-overlapping byte spans from source text.
/// Preserves whitespace and comments outside the spans.
/// Spans must be sorted by start position and must not overlap.
pub fn remove_spans(source: &str, spans: &[ByteSpan]) -> String {
    if spans.is_empty() {
        return source.to_string();
    }
    for i in 1..spans.len() {
        assert!(
            spans[i - 1].end <= spans[i].start,
            "Spans must be sorted and non-overlapping: {} overlaps with {}", spans[i -
            1], spans[i]
        );
    }
    let mut result = String::with_capacity(source.len());
    let mut pos: usize = 0;
    for span in spans {
        if span.start > pos {
            result.push_str(&source[pos..span.start]);
        }
        pos = span.end;
    }
    if pos < source.len() {
        result.push_str(&source[pos..]);
    }
    collapse_blank_lines(&result)
}
/// Collapse runs of 3+ blank lines down to 2.
pub fn collapse_blank_lines(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut blank_count: usize = 0;
    for line in text.lines() {
        if line.trim().is_empty() {
            blank_count += 1;
            if blank_count <= 2 {
                result.push('\n');
            }
        } else {
            blank_count = 0;
            result.push_str(line);
            result.push('\n');
        }
    }
    result
}
