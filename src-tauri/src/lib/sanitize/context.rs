use super::lines::LineMeta;

#[derive(Clone, Copy, Debug, Default)]
pub struct BoundaryContext {
    pub paren_depth: usize,
    pub bracket_depth: usize,
    pub in_string: bool,
    pub in_block_comment: bool,
}

pub fn build_boundary_contexts(lines: &[LineMeta]) -> Vec<BoundaryContext> {
    if lines.len() < 2 {
        return Vec::new();
    }

    let mut contexts = Vec::with_capacity(lines.len() - 1);
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut in_single = false;
    let mut in_double = false;
    let mut in_backtick = false;
    let mut in_block_comment = false;

    for line in lines.iter() {
        let mut chars = line.trimmed_end.chars().peekable();
        while let Some(ch) = chars.next() {
            if in_block_comment {
                if ch == '*' && chars.peek() == Some(&'/') {
                    chars.next();
                    in_block_comment = false;
                }
                continue;
            }

            if in_single {
                if ch == '\\' {
                    let _ = chars.next();
                    continue;
                }
                if ch == '\'' {
                    in_single = false;
                }
                continue;
            }

            if in_double {
                if ch == '\\' {
                    let _ = chars.next();
                    continue;
                }
                if ch == '"' {
                    in_double = false;
                }
                continue;
            }

            if in_backtick {
                if ch == '\\' {
                    let _ = chars.next();
                    continue;
                }
                if ch == '`' {
                    in_backtick = false;
                }
                continue;
            }

            if ch == '/' && chars.peek() == Some(&'/') {
                break;
            }
            if ch == '/' && chars.peek() == Some(&'*') {
                chars.next();
                in_block_comment = true;
                continue;
            }

            match ch {
                '\'' => in_single = true,
                '"' => in_double = true,
                '`' => in_backtick = true,
                '(' => paren_depth += 1,
                ')' => paren_depth = paren_depth.saturating_sub(1),
                '[' => bracket_depth += 1,
                ']' => bracket_depth = bracket_depth.saturating_sub(1),
                _ => {}
            }
        }

        contexts.push(BoundaryContext {
            paren_depth,
            bracket_depth,
            in_string: in_single || in_double || in_backtick,
            in_block_comment,
        });
    }

    contexts.pop();
    contexts
}
