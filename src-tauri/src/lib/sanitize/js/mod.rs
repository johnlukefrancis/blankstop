mod parse;
mod reflow;

use super::heuristics::is_identifier_split;

pub fn looks_js_like(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return false;
    }
    if !text.contains('\n') {
        return false;
    }
    if !has_js_signal(trimmed) {
        return false;
    }
    true
}

pub fn sanitize_js(text: &str) -> Option<String> {
    let candidate = reflow::reflow_js(text);
    parse::parse_and_codegen(&candidate).ok().map(|_| candidate)
}

fn has_js_signal(text: &str) -> bool {
    if text.contains("=>") {
        return true;
    }
    if text.contains(';') {
        return true;
    }
    if contains_keyword(text) {
        return true;
    }
    let has_structure = text.contains('{')
        || text.contains('}')
        || text.contains('(')
        || text.contains(')')
        || text.contains('[')
        || text.contains(']');
    if !has_structure {
        return false;
    }
    has_soft_wrap_seam(text)
}

fn has_soft_wrap_seam(text: &str) -> bool {
    let mut lines = text.split('\n').peekable();
    while let Some(current) = lines.next() {
        let Some(next) = lines.peek().copied() else { break };
        let prev_trim = current.trim_end();
        let next_trim = next.trim_start();
        if prev_trim.is_empty() || next_trim.is_empty() {
            continue;
        }
        if is_identifier_split(prev_trim, next_trim) {
            return true;
        }
        let prev_last = prev_trim.chars().last();
        let next_first = next_trim.chars().next();
        let Some(prev_last) = prev_last else { continue };
        let Some(next_first) = next_first else { continue };
        if is_continuation_char(prev_last) || is_continuation_char(next_first) {
            return true;
        }
    }
    false
}

fn is_continuation_char(ch: char) -> bool {
    matches!(
        ch,
        '.' | ',' | '+' | '-' | '*' | '/' | '%' | '=' | '?' | ':' | '&' | '|' | '(' | '['
    )
}

fn contains_keyword(text: &str) -> bool {
    let keywords = [
        "const", "let", "var", "function", "return", "class", "import", "export", "if",
        "for", "while", "switch", "try", "catch", "throw", "async", "await", "new",
    ];
    for keyword in keywords {
        if contains_word(text, keyword) {
            return true;
        }
    }
    false
}

fn contains_word(text: &str, word: &str) -> bool {
    let mut index = 0usize;
    while let Some(pos) = text[index..].find(word) {
        let start = index + pos;
        let end = start + word.len();
        let before = text[..start].chars().last();
        let after = text[end..].chars().next();
        let before_ok = before.map(|ch| !is_ident_char(ch)).unwrap_or(true);
        let after_ok = after.map(|ch| !is_ident_char(ch)).unwrap_or(true);
        if before_ok && after_ok {
            return true;
        }
        index = end;
    }
    false
}

fn is_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'
}
