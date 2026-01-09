#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Normal,
    SingleQuote,
    DoubleQuote,
    Template,
    LineComment,
    BlockComment,
}

pub fn reflow_js(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut mode = Mode::Normal;
    let mut prev_was_space = false;
    let mut prev_non_ws: Option<char> = None;
    let mut line_has_content = false;

    while let Some(ch) = chars.next() {
        match mode {
            Mode::Normal => {
                if ch == '/' {
                    if let Some('/') = chars.peek().copied() {
                        out.push('/');
                        out.push('/');
                        chars.next();
                        mode = Mode::LineComment;
                        prev_was_space = false;
                        continue;
                    }
                    if let Some('*') = chars.peek().copied() {
                        out.push('/');
                        out.push('*');
                        chars.next();
                        mode = Mode::BlockComment;
                        prev_was_space = false;
                        continue;
                    }
                }
                if ch == '\'' {
                    out.push(ch);
                    mode = Mode::SingleQuote;
                    prev_was_space = false;
                    continue;
                }
                if ch == '"' {
                    out.push(ch);
                    mode = Mode::DoubleQuote;
                    prev_was_space = false;
                    continue;
                }
                if ch == '`' {
                    out.push(ch);
                    mode = Mode::Template;
                    prev_was_space = false;
                    continue;
                }
                if ch == '\n' {
                    if !line_has_content {
                        out.push('\n');
                        prev_was_space = true;
                        prev_non_ws = None;
                        continue;
                    }
                    let (next_non_ws, next_is_line_comment) = peek_next_non_ws(&chars);
                    let next_line_trim = peek_next_line_trim(&chars);
                    if should_join(
                        prev_non_ws,
                        next_non_ws,
                        next_is_line_comment,
                        next_line_trim.as_deref(),
                    ) {
                        if !prev_was_space {
                            out.push(' ');
                            prev_was_space = true;
                        }
                    } else {
                        out.push('\n');
                        prev_was_space = true;
                        prev_non_ws = None;
                        line_has_content = false;
                    }
                    continue;
                }
                out.push(ch);
                prev_was_space = ch.is_whitespace();
                if !ch.is_whitespace() {
                    prev_non_ws = Some(ch);
                    line_has_content = true;
                }
            }
            Mode::LineComment => {
                out.push(ch);
                if ch == '\n' {
                    mode = Mode::Normal;
                    prev_was_space = true;
                    prev_non_ws = None;
                    line_has_content = false;
                }
            }
            Mode::BlockComment => {
                out.push(ch);
                if ch == '*' {
                    if let Some('/') = chars.peek().copied() {
                        out.push('/');
                        chars.next();
                        mode = Mode::Normal;
                        prev_was_space = false;
                    }
                }
            }
            Mode::SingleQuote => {
                if ch == '\n' {
                    out.push(' ');
                    prev_was_space = true;
                    continue;
                }
                out.push(ch);
                if ch == '\\' {
                    if let Some(next) = chars.peek().copied() {
                        out.push(next);
                        chars.next();
                    }
                    continue;
                }
                if ch == '\'' {
                    mode = Mode::Normal;
                    prev_was_space = false;
                    continue;
                }
            }
            Mode::DoubleQuote => {
                if ch == '\n' {
                    out.push(' ');
                    prev_was_space = true;
                    continue;
                }
                out.push(ch);
                if ch == '\\' {
                    if let Some(next) = chars.peek().copied() {
                        out.push(next);
                        chars.next();
                    }
                    continue;
                }
                if ch == '"' {
                    mode = Mode::Normal;
                    prev_was_space = false;
                    continue;
                }
            }
            Mode::Template => {
                out.push(ch);
                if ch == '\\' {
                    if let Some(next) = chars.peek().copied() {
                        out.push(next);
                        chars.next();
                    }
                    continue;
                }
                if ch == '`' {
                    mode = Mode::Normal;
                    prev_was_space = false;
                    continue;
                }
                if ch == '\n' {
                    prev_was_space = true;
                }
            }
        }
    }

    out.trim_end().to_string()
}

fn peek_next_non_ws(chars: &std::iter::Peekable<std::str::Chars<'_>>) -> (Option<char>, bool) {
    let mut iter = chars.clone();
    while let Some(ch) = iter.next() {
        if ch.is_whitespace() {
            continue;
        }
        if ch == '/' {
            let is_line_comment = matches!(iter.peek().copied(), Some('/'));
            return (Some(ch), is_line_comment);
        }
        return (Some(ch), false);
    }
    (None, false)
}

fn should_join(
    prev_non_ws: Option<char>,
    next_non_ws: Option<char>,
    next_is_line_comment: bool,
    next_line_trim: Option<&str>,
) -> bool {
    if next_non_ws.is_none() || next_is_line_comment {
        return false;
    }
    if matches!(prev_non_ws, Some(';' | '{' | '}' | ')' | ']')) {
        return false;
    }
    if matches!(next_non_ws, Some('}' | ']' | ')')) {
        return false;
    }
    if next_line_trim
        .map(|trim| looks_like_property_start(trim))
        .unwrap_or(false)
    {
        return false;
    }
    true
}

fn peek_next_line_trim(chars: &std::iter::Peekable<std::str::Chars<'_>>) -> Option<String> {
    let mut iter = chars.clone();
    let mut started = false;
    let mut out = String::new();
    while let Some(ch) = iter.next() {
        if ch == '\n' {
            break;
        }
        if !started {
            if ch.is_whitespace() {
                continue;
            }
            started = true;
        }
        if started {
            out.push(ch);
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn looks_like_property_start(text: &str) -> bool {
    let trimmed = text.trim_start();
    if trimmed.is_empty() {
        return false;
    }
    let mut chars = trimmed.chars().peekable();
    match chars.peek().copied() {
        Some('\"') | Some('\'') => {
            let quote = chars.next().unwrap();
            let mut prev_escape = false;
            while let Some(ch) = chars.next() {
                if prev_escape {
                    prev_escape = false;
                    continue;
                }
                if ch == '\\' {
                    prev_escape = true;
                    continue;
                }
                if ch == quote {
                    break;
                }
            }
            while let Some(ch) = chars.peek().copied() {
                if ch.is_whitespace() {
                    chars.next();
                } else {
                    break;
                }
            }
            return chars.peek() == Some(&':');
        }
        Some(ch) if is_ident_start(ch) => {
            while let Some(ch) = chars.peek().copied() {
                if is_ident_char(ch) {
                    chars.next();
                } else {
                    break;
                }
            }
            while let Some(ch) = chars.peek().copied() {
                if ch.is_whitespace() {
                    chars.next();
                } else {
                    break;
                }
            }
            return chars.peek() == Some(&':');
        }
        _ => {}
    }
    false
}

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_' || ch == '$'
}

fn is_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'
}
