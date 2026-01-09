use super::clean::CleanResult;
use super::{SanitizeResult, SanitizeSummary};

pub const MAX_CLIPBOARD_CHARS: usize = 250_000;

pub fn exceeds_max_clipboard_chars(text: &str) -> bool {
    text.chars().count() > MAX_CLIPBOARD_CHARS
}

pub fn sanitize_large_input(clean: CleanResult) -> SanitizeResult {
    let (output, trimmed_trailing_ws) = trim_trailing_whitespace(&clean.text);
    SanitizeResult {
        output,
        summary: SanitizeSummary {
            unwrapped_lines: 0,
            trimmed_trailing_ws,
            trimmed_blank_lines: 0,
            removed_invisibles: clean.removed_invisibles,
            stripped_prefixes: clean.stripped_prefixes,
            js_validated: false,
        },
    }
}

fn trim_trailing_whitespace(text: &str) -> (String, usize) {
    let mut trimmed_trailing_ws = 0usize;
    let mut output = String::with_capacity(text.len());
    let mut lines = text.split('\n').peekable();
    while let Some(line) = lines.next() {
        let trailing = line.chars().rev().take_while(|ch| ch.is_whitespace()).count();
        trimmed_trailing_ws += trailing;
        if trailing == 0 {
            output.push_str(line);
        } else {
            output.push_str(line.trim_end());
        }
        if lines.peek().is_some() {
            output.push('\n');
        }
    }
    (output, trimmed_trailing_ws)
}
