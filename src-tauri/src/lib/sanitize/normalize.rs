pub fn normalize_line_endings(input: &str) -> String {
    input.replace("\r\n", "\n").replace('\r', "\n")
}

pub fn trim_trailing_whitespace(input: &str) -> (Vec<String>, usize) {
    let mut removed = 0;
    let lines = input
        .split('\n')
        .map(|line| {
            let trimmed = line.trim_end();
            removed += line.len().saturating_sub(trimmed.len());
            trimmed.to_string()
        })
        .collect();
    (lines, removed)
}

pub fn trim_blank_edges(lines: &mut Vec<String>) -> usize {
    let mut start = 0;
    let mut end = lines.len();
    while start < end && lines[start].trim().is_empty() {
        start += 1;
    }
    while end > start && lines[end - 1].trim().is_empty() {
        end -= 1;
    }
    let trimmed = start + (lines.len() - end);
    if start > 0 || end < lines.len() {
        *lines = lines[start..end].to_vec();
    }
    trimmed
}
