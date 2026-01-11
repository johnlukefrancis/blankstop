#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ContinuationKind {
    Bash,
    PowerShell,
    Cmd,
}

pub(super) fn continuation_kind(
    line: &str,
    in_single: bool,
    in_double: bool,
) -> Option<ContinuationKind> {
    let trimmed = line.trim_end();
    let (marker_index, marker) = last_char_with_index(trimmed)?;
    let kind = match marker {
        '\\' => ContinuationKind::Bash,
        '`' => ContinuationKind::PowerShell,
        '^' => ContinuationKind::Cmd,
        _ => return None,
    };
    if marker_index == 0 {
        return None;
    }
    let prev = trimmed[..marker_index].chars().last()?;
    if !prev.is_whitespace() {
        return None;
    }
    if is_in_quotes_at(line, marker_index, in_single, in_double) {
        return None;
    }
    Some(kind)
}

pub(super) fn strip_continuation_marker(line: &str) -> &str {
    let trimmed = line.trim_end();
    let cut = trimmed.len().saturating_sub(1);
    &line[..cut]
}

fn last_char_with_index(text: &str) -> Option<(usize, char)> {
    text.char_indices().last()
}

fn is_in_quotes_at(line: &str, stop_at: usize, mut in_single: bool, mut in_double: bool) -> bool {
    let mut escaped = false;
    for (idx, ch) in line.char_indices() {
        if idx >= stop_at {
            break;
        }
        if escaped {
            escaped = false;
            continue;
        }
        if !in_single && matches!(ch, '\\' | '`') {
            escaped = true;
            continue;
        }
        if ch == '\'' && !in_double {
            in_single = !in_single;
            continue;
        }
        if ch == '"' && !in_single {
            in_double = !in_double;
        }
    }
    in_single || in_double
}
