use super::context::BoundaryContext;
use super::heuristics::is_identifier_split;
use super::lines::LineMeta;
use super::profile::WrapProfile;

#[derive(Clone, Copy, Debug)]
pub struct JoinDecision {
    pub join: bool,
    pub insert_space: bool,
}

pub fn decide_join(
    prev: &LineMeta,
    next: &LineMeta,
    profile: &WrapProfile,
    ctx: BoundaryContext,
) -> JoinDecision {
    let prev_trim = prev.trimmed_end.as_str();
    let next_trim = next.trimmed_end.as_str();
    let next_trim_start = next_trim.trim_start();

    if prev_trim.trim().is_empty() || next_trim.trim().is_empty() {
        return JoinDecision {
            join: false,
            insert_space: false,
        };
    }

    if ctx.in_block_comment {
        return JoinDecision {
            join: false,
            insert_space: false,
        };
    }

    if prev_trim.ends_with('{') && next.leading_ws_count > prev.leading_ws_count {
        return JoinDecision {
            join: false,
            insert_space: false,
        };
    }

    if next_trim_start.starts_with('}') || next_trim_start.starts_with(']') || next_trim_start.starts_with(')') {
        return JoinDecision {
            join: false,
            insert_space: false,
        };
    }

    if ctx.paren_depth == 0
        && ctx.bracket_depth == 0
        && prev_trim.ends_with(';')
        && next.leading_ws_count <= prev.leading_ws_count
    {
        return JoinDecision {
            join: false,
            insert_space: false,
        };
    }

    if next.leading_ws_count == prev.leading_ws_count
        && looks_like_property_start(next_trim_start)
    {
        return JoinDecision {
            join: false,
            insert_space: false,
        };
    }

    let identifier_split = is_identifier_split(prev_trim, next_trim_start);
    if identifier_split {
        return JoinDecision {
            join: true,
            insert_space: false,
        };
    }

    if ends_with_keyword(prev_trim, "return") || ends_with_keyword(prev_trim, "throw") {
        if looks_like_expression_start(next_trim_start)
            && (next.leading_ws_count > 0
                || matches!(next_trim_start.chars().next(), Some('(' | '[' | '{' | '"' | '\'' | '`')))
        {
            return JoinDecision {
                join: true,
                insert_space: true,
            };
        }
    }

    let mut score = 0i32;
    if ctx.paren_depth > 0 || ctx.bracket_depth > 0 {
        score += 3;
    }
    if ctx.brace_depth > 0 && next.leading_ws_count > prev.leading_ws_count {
        score += 1;
    }
    if ctx.in_string {
        score += 1;
    }
    if next.leading_ws_count > 0 {
        score += 1;
    }
    if next.leading_ws_count < prev.leading_ws_count {
        score += 1;
    }
    if continuation_punct(prev_trim, next_trim_start) {
        score += 2;
    }
    if let Some(wrap_width) = profile.wrap_width {
        let slack = profile.slack as isize;
        let delta = prev.raw_len_chars as isize - wrap_width as isize;
        if delta.abs() <= slack {
            score += 1;
        }
    }
    if prev.trailing_ws_count > 0 {
        score += 1;
    }

    if score < 4 {
        return JoinDecision {
            join: false,
            insert_space: false,
        };
    }

    let insert_space = should_insert_space(prev_trim, next_trim_start);
    JoinDecision {
        join: true,
        insert_space,
    }
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

fn ends_with_keyword(line: &str, keyword: &str) -> bool {
    let trimmed = line.trim_end();
    if !trimmed.ends_with(keyword) {
        return false;
    }
    let prefix_len = trimmed.len().saturating_sub(keyword.len());
    let before = trimmed[..prefix_len].chars().last();
    match before {
        None => true,
        Some(ch) => !ch.is_ascii_alphanumeric() && ch != '_',
    }
}

fn looks_like_expression_start(text: &str) -> bool {
    let trimmed = text.trim_start();
    if trimmed.is_empty() {
        return false;
    }
    let first = trimmed.chars().next().unwrap();
    first.is_ascii_alphanumeric()
        || matches!(first, '(' | '[' | '{' | '"' | '\'' | '`' | '_' | '$')
}

fn looks_like_property_start(text: &str) -> bool {
    let trimmed = text.trim_start();
    if trimmed.is_empty() {
        return false;
    }
    let mut chars = trimmed.chars().peekable();
    match chars.peek() {
        Some('"') | Some('\'') => {
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
            while let Some(ch) = chars.peek() {
                if ch.is_whitespace() {
                    chars.next();
                } else {
                    break;
                }
            }
            return chars.peek() == Some(&':');
        }
        Some(ch) if ch.is_ascii_alphabetic() || *ch == '_' || *ch == '$' => {
            while let Some(ch) = chars.peek() {
                if ch.is_ascii_alphanumeric() || *ch == '_' || *ch == '$' {
                    chars.next();
                } else {
                    break;
                }
            }
            while let Some(ch) = chars.peek() {
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

fn continuation_punct(prev: &str, next: &str) -> bool {
    let prev_trim = prev.trim_end();
    let next_trim = next.trim_start();
    if prev_trim.is_empty() || next_trim.is_empty() {
        return false;
    }
    let prev_last = prev_trim.chars().last().unwrap();
    let next_first = next_trim.chars().next().unwrap();
    matches!(prev_last, ',' | '.' | '+' | '-' | '*' | '/' | '%' | '=' | '?' | ':' | '&' | '|' | '(' | '[')
        || matches!(next_first, '.' | ',' | ')' | ']' | ':' | '?' | '+' | '-' | '*' | '/' | '%' | '&' | '|' | '[' | '(')
}
