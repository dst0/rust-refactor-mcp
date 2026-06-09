pub fn line_col_to_byte(source: &str, line: usize, column: usize) -> usize {
    let target_line = line.saturating_sub(1);
    let mut byte_offset = 0;
    let mut current_line: usize = 0;
    let mut current_col: usize = 0;
    for c in source.chars() {
        if current_line == target_line && current_col == column {
            return byte_offset;
        }
        byte_offset += c.len_utf8();
        if c == '\n' {
            current_line += 1;
            current_col = 0;
        } else {
            current_col += 1;
        }
    }
    byte_offset
}
