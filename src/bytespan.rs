use crate::merge_spans::merge_spans;
use std::fmt;
/// A byte-range within a source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteSpan {
    pub start: usize,
    pub end: usize,
}
impl ByteSpan {
    pub fn new(start: usize, end: usize) -> Self {
        assert!(start <= end, "Invalid span: start ({}) > end ({})", start, end);
        Self { start, end }
    }
    pub fn len(&self) -> usize {
        self.end - self.start
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn overlaps_with(&self, other: &ByteSpan) -> bool {
        self.start < other.end && other.start < self.end
    }
    pub fn contains(&self, other: &ByteSpan) -> bool {
        self.start <= other.start && other.end <= self.end
    }
    pub fn merge_with(&self, other: &ByteSpan) -> ByteSpan {
        ByteSpan {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}
impl fmt::Display for ByteSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}..{}]", self.start, self.end)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_valid() {
        let s = ByteSpan::new(5, 15);
        assert_eq!(s.start, 5);
        assert_eq!(s.end, 15);
    }
    #[test]
    #[should_panic(expected = "start")]
    fn new_invalid() {
        ByteSpan::new(10, 5);
    }
    #[test]
    fn new_zero_len() {
        let s = ByteSpan::new(5, 5);
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
    }
    #[test]
    fn len_normal() {
        assert_eq!(ByteSpan::new(5, 15).len(), 10);
    }
    #[test]
    fn is_empty() {
        assert!(ByteSpan::new(3, 3).is_empty());
        assert!(! ByteSpan::new(3, 4).is_empty());
    }
    #[test]
    fn overlaps_true() {
        assert!(ByteSpan::new(0, 10).overlaps_with(& ByteSpan::new(5, 15)));
        assert!(ByteSpan::new(0, 20).overlaps_with(& ByteSpan::new(5, 10)));
    }
    #[test]
    fn overlaps_false() {
        assert!(! ByteSpan::new(0, 10).overlaps_with(& ByteSpan::new(10, 20)));
        assert!(! ByteSpan::new(0, 5).overlaps_with(& ByteSpan::new(10, 15)));
    }
    #[test]
    fn contains() {
        assert!(ByteSpan::new(0, 20).contains(& ByteSpan::new(5, 10)));
        assert!(ByteSpan::new(5, 10).contains(& ByteSpan::new(5, 10)));
        assert!(! ByteSpan::new(0, 10).contains(& ByteSpan::new(5, 20)));
    }
    #[test]
    fn merge_with() {
        let m = ByteSpan::new(0, 10).merge_with(&ByteSpan::new(5, 15));
        assert_eq!(m, ByteSpan::new(0, 15));
    }
    #[test]
    fn display() {
        assert_eq!(format!("{}", ByteSpan::new(5, 15)), "[5..15]");
    }
    #[test]
    fn remove_empty_spans() {
        assert_eq!(remove_spans("hello", & []), "hello");
    }
    #[test]
    fn remove_from_start() {
        let r = remove_spans("abcdef", &[ByteSpan::new(0, 3)]);
        assert!(r.contains("def"));
        assert!(! r.contains("abc"));
    }
    #[test]
    fn remove_from_end() {
        let r = remove_spans("abcdef", &[ByteSpan::new(3, 6)]);
        assert!(r.contains("abc"));
        assert!(! r.contains("def"));
    }
    #[test]
    fn remove_middle() {
        let r = remove_spans("hello world", &[ByteSpan::new(6, 11)]);
        assert!(! r.contains("world"));
        assert!(r.contains("hello"));
    }
    #[test]
    fn remove_multiple() {
        let r = remove_spans(
            "a\nb\nc\nd\ne\n",
            &[ByteSpan::new(2, 3), ByteSpan::new(6, 7)],
        );
        assert!(! r.contains("b"));
        assert!(! r.contains("d"));
        assert!(r.contains("a"));
        assert!(r.contains("c"));
        assert!(r.contains("e"));
    }
    #[test]
    fn remove_adjacent() {
        let r = remove_spans("abcdef", &[ByteSpan::new(0, 2), ByteSpan::new(2, 4)]);
        assert_eq!(r.trim(), "ef");
    }
    #[test]
    #[should_panic]
    fn remove_overlapping_panics() {
        remove_spans("abcdef", &[ByteSpan::new(0, 4), ByteSpan::new(2, 6)]);
    }
    #[test]
    #[should_panic]
    fn remove_unsorted_panics() {
        remove_spans("abcdef", &[ByteSpan::new(4, 6), ByteSpan::new(0, 2)]);
    }
    #[test]
    fn collapse_no_change() {
        let input = "a\nb\nc";
        let output = collapse_blank_lines(&input);
        assert!(output.contains("a"));
        assert!(output.contains("b"));
        assert!(output.contains("c"));
    }
    #[test]
    fn collapse_many_blanks() {
        let input = "a\n\n\n\n\n\nb";
        let output = collapse_blank_lines(&input);
        assert!(! output.contains("\n\n\n\n"));
    }
    #[test]
    fn merge_empty() {
        assert!(merge_spans(vec![]).is_empty());
    }
    #[test]
    fn merge_single() {
        let r = merge_spans(vec![ByteSpan::new(5, 10)]);
        assert_eq!(r, vec![ByteSpan::new(5, 10)]);
    }
    #[test]
    fn merge_no_overlap() {
        let r = merge_spans(vec![ByteSpan::new(0, 5), ByteSpan::new(10, 15)]);
        assert_eq!(r.len(), 2);
    }
    #[test]
    fn merge_overlapping() {
        let r = merge_spans(
            vec![ByteSpan::new(0, 10), ByteSpan::new(5, 15), ByteSpan::new(20, 30),],
        );
        assert_eq!(r.len(), 2);
        assert_eq!(r[0], ByteSpan::new(0, 15));
        assert_eq!(r[1], ByteSpan::new(20, 30));
    }
    #[test]
    fn merge_contained() {
        let r = merge_spans(vec![ByteSpan::new(0, 20), ByteSpan::new(5, 10)]);
        assert_eq!(r, vec![ByteSpan::new(0, 20)]);
    }
    #[test]
    fn merge_adjacent() {
        let r = merge_spans(vec![ByteSpan::new(0, 10), ByteSpan::new(10, 20)]);
        assert_eq!(r, vec![ByteSpan::new(0, 20)]);
    }
    #[test]
    fn merge_unsorted_input() {
        let r = merge_spans(vec![ByteSpan::new(20, 30), ByteSpan::new(0, 10)]);
        assert_eq!(r, vec![ByteSpan::new(0, 10), ByteSpan::new(20, 30)]);
    }
}
use crate::spans::remove_spans;
use crate::spans::collapse_blank_lines;
