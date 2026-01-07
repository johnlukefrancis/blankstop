use super::heuristics::{is_identifier_split, is_probably_code_block};

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
