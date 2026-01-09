pub fn is_probably_code_block(lines: &[String]) -> bool {
    let mut has_indent = false;
    let mut has_brace = false;
    let mut has_semicolon = false;
    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if line.starts_with(|c: char| c.is_whitespace()) {
            has_indent = true;
        }
        if trimmed.contains('{') || trimmed.contains('}') {
            has_brace = true;
        }
        if trimmed.ends_with(';') {
            has_semicolon = true;
        }
    }
    has_indent && (has_brace || has_semicolon)
}

pub fn is_identifier_split(prev: &str, next: &str) -> bool {
    let prev_last = prev.chars().last();
    let next_first = next.chars().next();
    let Some(prev_last) = prev_last else { return false };
    let Some(next_first) = next_first else { return false };
    if !is_ident_char(prev_last) || !is_ident_char(next_first) {
        return false;
    }
    let chars = prev.chars().rev();
    for ch in chars {
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
