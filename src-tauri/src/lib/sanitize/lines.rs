#[derive(Clone, Debug)]
pub struct LineMeta {
    pub raw: String,
    pub raw_len_chars: usize,
    pub leading_ws_count: usize,
    pub trailing_ws_count: usize,
    pub trimmed_end: String,
}

pub fn build_line_meta(input: &str) -> (Vec<LineMeta>, usize) {
    let mut trimmed_trailing_ws = 0usize;
    let lines = input
        .split('\n')
        .map(|line| {
            let raw_len_chars = line.chars().count();
            let leading_ws_count = count_leading_ws(line);
            let trailing_ws_count = count_trailing_ws(line);
            let trimmed_end = line.trim_end().to_string();
            trimmed_trailing_ws += trailing_ws_count;
            LineMeta {
                raw: line.to_string(),
                raw_len_chars,
                leading_ws_count,
                trailing_ws_count,
                trimmed_end,
            }
        })
        .collect();
    (lines, trimmed_trailing_ws)
}

pub fn trim_blank_edges(lines: &mut Vec<LineMeta>) -> usize {
    let mut start = 0;
    let mut end = lines.len();
    while start < end && lines[start].trimmed_end.trim().is_empty() {
        start += 1;
    }
    while end > start && lines[end - 1].trimmed_end.trim().is_empty() {
        end -= 1;
    }
    let trimmed = start + (lines.len() - end);
    if start > 0 || end < lines.len() {
        *lines = lines[start..end].to_vec();
    }
    trimmed
}

fn count_leading_ws(line: &str) -> usize {
    line.chars().take_while(|ch| ch.is_whitespace()).count()
}

fn count_trailing_ws(line: &str) -> usize {
    line.chars().rev().take_while(|ch| ch.is_whitespace()).count()
}
