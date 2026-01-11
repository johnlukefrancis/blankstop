use super::heredoc;

#[derive(Debug, Default, Clone, Copy)]
pub struct ShellReflowSummary {
    pub repaired_string_wraps: usize,
    pub removed_cmd_carets: usize,
    pub joined_explicit_continuations: usize,
}
#[derive(Debug, Clone)]
pub struct ShellReflowResult {
    pub output: String,
    pub summary: ShellReflowSummary,
}
enum ContinuationKind {
    Bash,
    PowerShell,
    Cmd,
}
pub fn reflow_shell(text: &str) -> ShellReflowResult {
    let mut summary = ShellReflowSummary::default();
    let repaired = heredoc::repair_indented_terminators(text);
    let mut output = String::with_capacity(repaired.len());
    let mut in_single = false;
    let mut in_double = false;
    let mut here_delim: Option<char> = None;
    let mut skip_leading = 0usize;
    let mut lines = repaired.split('\n').peekable();
    while let Some(raw_line) = lines.next() {
        let mut line = raw_line;
        if skip_leading > 0 {
            line = trim_leading_by(line, skip_leading);
            skip_leading = 0;
        }
        let is_last = lines.peek().is_none();
        if let Some(delim) = here_delim {
            output.push_str(line);
            if !is_last {
                output.push('\n');
            }
            if ends_here_string(line, delim) {
                here_delim = None;
            }
            continue;
        }
        let here_start = starts_here_string(line);
        let line_for_quotes = if let Some((idx, _)) = here_start {
            &line[..idx]
        } else {
            line
        };
        update_quote_state(line_for_quotes, &mut in_single, &mut in_double);
        if let Some((_, delim)) = here_start {
            here_delim = Some(delim);
            output.push_str(line);
            if !is_last {
                output.push('\n');
            }
            continue;
        }
        if is_last {
            output.push_str(line);
            break;
        }
        let mut join_next = false;
        let mut continuation: Option<ContinuationKind> = None;
        if !in_single {
            if let Some(kind) = continuation_kind(line) {
                continuation = Some(kind);
                join_next = true;
            }
        }
        if !join_next && (in_single || in_double) {
            if let Some(next_line) = lines.peek().copied() {
                if should_join_string_wrap(line, next_line) {
                    join_next = true;
                    summary.repaired_string_wraps += 1;
                }
            }
        }
        if join_next {
            if let Some(kind) = continuation {
                let stripped = strip_continuation_marker(line);
                output.push_str(stripped);
                match kind {
                    ContinuationKind::Bash | ContinuationKind::PowerShell => {
                        summary.joined_explicit_continuations += 1;
                    }
                    ContinuationKind::Cmd => {
                        summary.removed_cmd_carets += 1;
                    }
                }
            } else {
                output.push_str(line);
            }
            if let Some(next_line) = lines.peek().copied() {
                skip_leading = count_leading_ws(next_line);
            }
            continue;
        }
        output.push_str(line);
        output.push('\n');
    }
    ShellReflowResult { output, summary }
}
fn update_quote_state(line: &str, in_single: &mut bool, in_double: &mut bool) {
    let mut escaped = false;
    for ch in line.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        if !*in_single && matches!(ch, '\\' | '`') {
            escaped = true;
            continue;
        }
        if ch == '\'' && !*in_double {
            *in_single = !*in_single;
            continue;
        }
        if ch == '"' && !*in_single {
            *in_double = !*in_double;
        }
    }
}
fn starts_here_string(line: &str) -> Option<(usize, char)> {
    let trimmed = line.trim_end();
    if trimmed.ends_with("@\"") {
        let idx = trimmed.len().saturating_sub(2);
        return Some((idx, '"'));
    }
    if trimmed.ends_with("@'") {
        let idx = trimmed.len().saturating_sub(2);
        return Some((idx, '\''));
    }
    None
}
fn ends_here_string(line: &str, delim: char) -> bool {
    let trimmed = line.trim();
    trimmed.len() == 2 && trimmed.starts_with(delim) && trimmed.ends_with('@')
}
fn continuation_kind(line: &str) -> Option<ContinuationKind> {
    let trimmed = line.trim_end();
    let last = trimmed.chars().last()?;
    match last {
        '\\' => Some(ContinuationKind::Bash),
        '`' => Some(ContinuationKind::PowerShell),
        '^' => Some(ContinuationKind::Cmd),
        _ => None,
    }
}
fn strip_continuation_marker(line: &str) -> &str {
    let trimmed = line.trim_end();
    let cut = trimmed.len().saturating_sub(1);
    &line[..cut]
}
fn should_join_string_wrap(line: &str, next_line: &str) -> bool {
    let line_trim = line.trim_end();
    if line_trim.is_empty() {
        return false;
    }
    if line_trim.chars().last().map(|ch| ch.is_whitespace()).unwrap_or(false) {
        return false;
    }
    let next_trim = next_line.trim_start();
    if next_trim.is_empty() {
        return false;
    }
    let leading_ws = count_leading_ws(next_line);
    if leading_ws == 0 {
        return false;
    }
    let prev_last = line_trim.chars().last();
    let next_first = next_trim.chars().next();
    let (Some(prev_last), Some(next_first)) = (prev_last, next_first) else {
        return false;
    };
    is_join_char(prev_last) && is_join_char(next_first)
}
fn is_join_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric()
        || matches!(ch, '_' | '-' | '.' | '/' | '\\' | ':' | '$' | '@')
}
fn count_leading_ws(line: &str) -> usize {
    for (idx, ch) in line.char_indices() {
        if ch == ' ' || ch == '\t' {
            continue;
        }
        return idx;
    }
    line.len()
}
fn trim_leading_by<'a>(line: &'a str, count: usize) -> &'a str {
    if count == 0 || count >= line.len() {
        return line;
    }
    &line[count..]
}
