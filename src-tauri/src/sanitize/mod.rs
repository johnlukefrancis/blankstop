use serde::Serialize;

#[derive(Debug, Default, Clone, Serialize)]
pub struct SanitizeSummary {
    pub unwrapped_lines: usize,
    pub trimmed_trailing_ws: usize,
    pub trimmed_blank_lines: usize,
}

#[derive(Debug, Clone)]
pub struct SanitizeResult {
    pub output: String,
    pub summary: SanitizeSummary,
}

pub fn sanitize_text(input: &str) -> SanitizeResult {
    let normalized = normalize_line_endings(input);
    let (mut lines, trimmed_trailing_ws) = trim_trailing_whitespace(&normalized);
    let trimmed_blank_lines = trim_blank_edges(&mut lines);

    if lines.is_empty() {
        return SanitizeResult {
            output: String::new(),
            summary: SanitizeSummary {
                unwrapped_lines: 0,
                trimmed_trailing_ws,
                trimmed_blank_lines,
            },
        };
    }

    let mut summary = SanitizeSummary {
        unwrapped_lines: 0,
        trimmed_trailing_ws,
        trimmed_blank_lines,
    };

    let output = if should_rule_a(&lines) {
        summary.unwrapped_lines = lines.len().saturating_sub(1);
        join_wrapped_single_line(&lines)
    } else if let Some(wrap_width) = infer_wrap_width(&lines) {
        join_wrapped_multiline(&lines, wrap_width, &mut summary)
    } else {
        lines.join("\n")
    };

    SanitizeResult { output, summary }
}

impl SanitizeSummary {
    pub fn toast_message(&self) -> String {
        let mut parts = Vec::new();
        if self.unwrapped_lines > 0 {
            parts.push(format!(
                "unwrapped {} line{}",
                self.unwrapped_lines,
                if self.unwrapped_lines == 1 { "" } else { "s" }
            ));
        }
        if self.trimmed_trailing_ws > 0 {
            parts.push(format!(
                "trimmed {} trailing space{}",
                self.trimmed_trailing_ws,
                if self.trimmed_trailing_ws == 1 { "" } else { "s" }
            ));
        }
        if self.trimmed_blank_lines > 0 {
            parts.push(format!(
                "trimmed {} blank line{}",
                self.trimmed_blank_lines,
                if self.trimmed_blank_lines == 1 { "" } else { "s" }
            ));
        }
        if parts.is_empty() {
            "Sanitized clipboard".to_string()
        } else {
            format!("Sanitized clipboard: {}", parts.join(", "))
        }
    }
}

fn normalize_line_endings(input: &str) -> String {
    input.replace("\r\n", "\n").replace('\r', "\n")
}

fn trim_trailing_whitespace(input: &str) -> (Vec<String>, usize) {
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

fn trim_blank_edges(lines: &mut Vec<String>) -> usize {
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

fn should_rule_a(lines: &[String]) -> bool {
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
    if line.starts_with(|c: char| c.is_whitespace()) {
        return true;
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

fn join_wrapped_single_line(lines: &[String]) -> String {
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

fn infer_wrap_width(lines: &[String]) -> Option<usize> {
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

fn join_wrapped_multiline(
    lines: &[String],
    wrap_width: usize,
    summary: &mut SanitizeSummary,
) -> String {
    let mut out_lines = Vec::new();
    let mut i = 0usize;
    while i < lines.len() {
        let mut line = lines[i].clone();
        let mut current_len = lines[i].chars().count();
        while i + 1 < lines.len()
            && should_join(current_len, &lines[i + 1], wrap_width)
        {
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

#[cfg(test)]
mod tests {
    use super::sanitize_text;

    #[test]
    fn wrapped_commit_subject_example() {
        let input = "🟦 improve(docs): document HUD GPU clear state restore\n  requirement\n";
        let result = sanitize_text(input);
        assert_eq!(
            result.output,
            "🟦 improve(docs): document HUD GPU clear state restore requirement"
        );
    }

    #[test]
    fn multiline_code_preserved() {
        let input = "const a = 1;\nconst b = 2;\n";
        let result = sanitize_text(input);
        assert_eq!(result.output, "const a = 1;\nconst b = 2;");
    }

    #[test]
    fn soft_wrap_join_simulation() {
        let input = "checksum: Array.from({ length: 24 }, (_, i) => ((i * 7 + 13) % 97)).jo\nin(',')\n";
        let result = sanitize_text(input);
        assert_eq!(
            result.output,
            "checksum: Array.from({ length: 24 }, (_, i) => ((i * 7 + 13) % 97)).join(',')"
        );
    }
}
