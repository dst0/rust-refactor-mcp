use crate::extract::ByteSpan;
/// Merge overlapping spans into a minimal set of non-overlapping spans.
pub fn merge_spans(mut spans: Vec<ByteSpan>) -> Vec<ByteSpan> {
    if spans.len() <= 1 {
        return spans;
    }
    spans.sort_by_key(|s| s.start);
    let mut merged = vec![spans[0]];
    for span in &spans[1..] {
        let last = merged.last_mut().unwrap();
        if span.start <= last.end {
            last.end = last.end.max(span.end);
        } else {
            merged.push(*span);
        }
    }
    merged
}
