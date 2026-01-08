use super::normalize::normalize_line_endings;

#[derive(Debug, Clone)]
pub struct CleanResult {
    pub text: String,
    pub normalized: String,
    pub removed_invisibles: usize,
    pub stripped_prefixes: usize,
}

pub fn clean_text(input: &str) -> CleanResult {
    let normalized = normalize_line_endings(input);
    let mut removed_invisibles = 0usize;
    let mut cleaned = String::with_capacity(normalized.len());

    for ch in normalized.chars() {
        if ch == '\u{2028}' || ch == '\u{2029}' {
            cleaned.push('\n');
            continue;
        }
        if is_invisible_format_char(ch) {
            removed_invisibles += 1;
            continue;
        }
        cleaned.push(ch);
    }

    let (text, stripped_prefixes) = strip_leading_bullet_prefix(&cleaned);

    CleanResult {
        text,
        normalized,
        removed_invisibles,
        stripped_prefixes,
    }
}

fn is_invisible_format_char(ch: char) -> bool {
    if ch.is_control() && !matches!(ch, '\n' | '\r' | '\t') {
        return true;
    }
    matches!(
        ch,
        '\u{00AD}'
            | '\u{061C}'
            | '\u{180E}'
            | '\u{200B}'..='\u{200F}'
            | '\u{202A}'..='\u{202E}'
            | '\u{2060}'..='\u{2064}'
            | '\u{2066}'..='\u{2069}'
            | '\u{FEFF}'
    )
}

fn strip_leading_bullet_prefix(input: &str) -> (String, usize) {
    let lines: Vec<&str> = input.split('\n').collect();
    let mut output = String::with_capacity(input.len());
    let mut stripped_prefixes = 0usize;
    let mut handled_first = false;

    for (idx, line) in lines.iter().enumerate() {
        if !handled_first && !line.trim().is_empty() {
            if let Some(replaced) = strip_bullet_prefix_if_codeish(line) {
                output.push_str(&replaced);
                stripped_prefixes = 1;
            } else {
                output.push_str(line);
            }
            handled_first = true;
        } else {
            output.push_str(line);
        }
        if idx + 1 < lines.len() {
            output.push('\n');
        }
    }

    if stripped_prefixes == 0 {
        return (input.to_string(), 0);
    }

    (output, stripped_prefixes)
}

fn strip_bullet_prefix_if_codeish(line: &str) -> Option<String> {
    let trimmed_start = line.trim_start();
    if !trimmed_start.starts_with("• ") {
        return None;
    }
    let after = &trimmed_start["• ".len()..];
    if !looks_codeish(after) {
        return None;
    }
    let leading_len = line.len() - trimmed_start.len();
    let start = leading_len + "• ".len();
    Some(format!("{}{}", &line[..leading_len], &line[start..]))
}

fn looks_codeish(text: &str) -> bool {
    let trimmed = text.trim_start();
    if trimmed.is_empty() {
        return false;
    }
    let first = trimmed.chars().next().unwrap();
    if matches!(first, '(' | '{' | '[') {
        return true;
    }
    let keywords = ["const", "let", "function", "return"];
    for keyword in keywords {
        if trimmed.starts_with(keyword) && is_keyword_boundary(trimmed, keyword.len()) {
            return true;
        }
    }
    false
}

fn is_keyword_boundary(text: &str, len: usize) -> bool {
    if text.len() == len {
        return true;
    }
    text[len..]
        .chars()
        .next()
        .map(|ch| !ch.is_ascii_alphanumeric() && ch != '_')
        .unwrap_or(true)
}
