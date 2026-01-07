use super::SanitizeSummary;

pub fn should_rule_a(lines: &[String]) -> bool {
    let len = lines.len();
    if !(2..=4).contains(&len) {
        return false;
    }
    let long_context = lines.iter().any(|line| line.chars().count() >= 40);
    for idx in 1..lines.len() {
        let line = &lines[idx];
        if is_whitespace_continuation(line) {
            return true;
        }
        if long_context && looks_wrapped(line) && should_insert_space(&lines[idx - 1], line) {
            return true;
        }
    }
    false
}

fn is_whitespace_continuation(line: &str) -> bool {
    if line.is_empty() {
        return false;
    }
    line.starts_with(|c: char| c.is_whitespace())
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

pub fn infer_wrap_width(lines: &[String]) -> Option<usize> {
    let mut counts = std::collections::HashMap::new();
    let mut total = 0usize;
    for line in lines {
        let len = line.chars().count();
        if len >= 40 {
            total += 1;
            *counts.entry(len).or_insert(0usize) += 1;
        }
    }
    if total == 0 {
        return None;
    }
    let (mode, count) = counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .unwrap();
    if total >= 2 {
        if count >= 2 && count * 2 >= total {
            Some(mode)
        } else {
            None
        }
    } else if lines.len() <= 3 {
        Some(mode)
    } else {
        None
    }
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

fn is_identifier_split(prev: &str, next: &str) -> bool {
    let prev_last = prev.chars().last();
    let next_first = next.chars().next();
    let Some(prev_last) = prev_last else { return false };
    let Some(next_first) = next_first else { return false };
    if !is_ident_char(prev_last) || !is_ident_char(next_first) {
        return false;
    }
    let mut chars = prev.chars().rev();
    while let Some(ch) = chars.next() {
        if is_ident_char(ch) {
            continue;
        }
        return matches!(ch, '.' | ':' | '>' | '/' | '\\' | '_' | '$');
    }
    false
}

fn is_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}
