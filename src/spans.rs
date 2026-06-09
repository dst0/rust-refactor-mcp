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
