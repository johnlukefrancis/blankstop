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

pub fn estimate_wrap_width(lines: &[String]) -> Option<usize> {
    // Invariant: estimate from original visual rows only, using a tolerant cluster (±2 chars).
    let mut long_lengths = Vec::new();
    for line in lines {
        let len = line.chars().count();
        if len >= 40 {
            long_lengths.push(len);
        }
    }
    if long_lengths.len() < 2 {
        return None;
    }

    let mut best_cluster: Vec<usize> = Vec::new();
    for center in &long_lengths {
        let cluster: Vec<usize> = long_lengths
            .iter()
            .cloned()
            .filter(|len| (*len as isize - *center as isize).abs() <= 2)
            .collect();
        if cluster.len() > best_cluster.len() {
            best_cluster = cluster;
        }
    }
    if best_cluster.len() < 2 {
        return None;
    }
    let sum: usize = best_cluster.iter().sum();
    Some((sum + best_cluster.len() / 2) / best_cluster.len())
}

pub fn should_join_identifier(prev_line: &str, next_line: &str) -> bool {
    if next_line.trim().is_empty() {
        return false;
    }
    if next_line.starts_with(|c: char| c.is_whitespace()) {
        return false;
    }
    let prev_trim = prev_line.trim_end();
    let next_trim = next_line.trim_start();
    if prev_trim.is_empty() || next_trim.is_empty() {
        return false;
    }
    if prev_trim.chars().count() < 40 {
        return false;
    }
    let Some(prev_last) = prev_trim.chars().last() else {
        return false;
    };
    let Some(next_first) = next_trim.chars().next() else {
        return false;
    };
    is_ident_char(prev_last) && is_ident_char(next_first)
}

pub fn is_identifier_split(prev: &str, next: &str) -> bool {
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
