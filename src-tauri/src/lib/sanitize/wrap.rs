pub use super::heuristics::estimate_wrap_width;
use super::heuristics::{is_identifier_split, is_probably_code_block, should_join_identifier};
use super::SanitizeSummary;

pub fn should_rule_a(lines: &[String]) -> bool {
    let len = lines.len();
    if !(2..=4).contains(&len) {
        return false;
    }
    if is_probably_code_block(lines) {
        return false;
    }
    let long_context = lines.iter().any(|line| line.chars().count() >= 40);
    for idx in 1..lines.len() {
        let line = &lines[idx];
        if long_context && looks_wrapped(line) && should_insert_space(&lines[idx - 1], line) {
            return true;
        }
    }
    false
}

fn looks_wrapped(line: &str) -> bool {
    let trimmed = line.trim_start();
    let first = trimmed.chars().next();
    match first {
        Some(ch) if ch.is_lowercase() => true,
        Some(',' | '.' | ';' | ':' | ')' | ']' | '}') => true,
        _ => false,
    }
}

pub fn join_identifier_splits(
    lines: &[String],
    summary: &mut SanitizeSummary,
) -> Option<Vec<String>> {
    let mut out_lines = Vec::with_capacity(lines.len());
    let mut i = 0usize;
    let mut changed = false;
    while i < lines.len() {
        let mut line = lines[i].clone();
        while i + 1 < lines.len() && should_join_identifier(&line, &lines[i + 1]) {
            let next_line = &lines[i + 1];
            let trimmed_next = next_line.trim_start();
            line.push_str(trimmed_next);
            summary.unwrapped_lines += 1;
            changed = true;
            i += 1;
        }
        out_lines.push(line);
        i += 1;
    }
    if changed { Some(out_lines) } else { None }
}

pub fn join_wrapped_single_line(lines: &[String]) -> String {
    let mut out = String::new();
    for (idx, line) in lines.iter().enumerate() {
        let part = if idx == 0 { line.trim_end() } else { line.trim_start() };
        if part.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out = out.trim_end().to_string();
            out.push(' ');
        }
        out.push_str(part);
    }
    out
}

pub fn join_wrapped_multiline(
    lines: &[String],
    wrap_width: usize,
    summary: &mut SanitizeSummary,
) -> String {
    let mut out_lines = Vec::new();
    let mut i = 0usize;
    while i < lines.len() {
        let mut line = lines[i].clone();
        let mut current_len = lines[i].chars().count();
        while i + 1 < lines.len() && should_join(current_len, &lines[i + 1], wrap_width) {
            let next_line = &lines[i + 1];
            let trimmed_next = if is_wrap_indent(next_line) {
                next_line.trim_start().to_string()
            } else {
                next_line.clone()
            };
            if should_insert_space(&line, &trimmed_next) {
                line.push(' ');
            }
            line.push_str(&trimmed_next);
            summary.unwrapped_lines += 1;
            i += 1;
            current_len = lines[i].chars().count();
        }
        out_lines.push(line);
        i += 1;
    }
    out_lines.join("\n")
}

fn should_join(prev_len: usize, next_line: &str, wrap_width: usize) -> bool {
    if prev_len + 2 < wrap_width {
        return false;
    }
    if prev_len > wrap_width + 2 {
        return false;
    }
    !next_line.trim().is_empty()
}

fn is_wrap_indent(line: &str) -> bool {
    line.starts_with("  ") || line.starts_with('\t')
}

fn should_insert_space(prev: &str, next: &str) -> bool {
    let prev_trim = prev.trim_end();
    let next_trim = next.trim_start();
    if prev_trim.is_empty() || next_trim.is_empty() {
        return false;
    }
    if prev_trim.ends_with(|c: char| c.is_whitespace())
        || next_trim.starts_with(|c: char| c.is_whitespace())
    {
        return false;
    }

    if is_identifier_split(prev_trim, next_trim) {
        return false;
    }

    true
}
