#[derive(Debug, Clone)]
struct HeredocSpec {
    delimiter: String,
}

pub(super) fn repair_indented_terminators(text: &str) -> String {
    if !text.contains("<<") {
        return text.to_string();
    }
    let mut output = String::with_capacity(text.len());
    let mut active: Option<HeredocSpec> = None;
    let mut lines = text.split('\n').peekable();

    while let Some(line) = lines.next() {
        let is_last = lines.peek().is_none();
        if let Some(spec) = active.as_ref() {
            if let Some(rewrite) = terminator_rewrite(line, &spec.delimiter) {
                if rewrite {
                    output.push_str(&spec.delimiter);
                } else {
                    output.push_str(line);
                }
                active = None;
            } else {
                output.push_str(line);
            }
            if !is_last {
                output.push('\n');
            }
            continue;
        }

        if let Some(spec) = parse_heredoc_opener(line) {
            active = Some(spec);
        }
        output.push_str(line);
        if !is_last {
            output.push('\n');
        }
    }

    output
}

fn terminator_rewrite(line: &str, delimiter: &str) -> Option<bool> {
    let trimmed_start = line.trim_start_matches(|ch| ch == ' ' || ch == '\t');
    let trimmed = trimmed_start.trim_end_matches(|ch| ch == ' ' || ch == '\t' || ch == '\r');
    if trimmed != delimiter {
        return None;
    }
    Some(line != delimiter)
}

fn parse_heredoc_opener(line: &str) -> Option<HeredocSpec> {
    let chars: Vec<char> = line.chars().collect();
    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;
    let mut i = 0usize;

    while i + 1 < chars.len() {
        let ch = chars[i];
        if in_double {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_double = false;
            }
            i += 1;
            continue;
        }
        if in_single {
            if ch == '\'' {
                in_single = false;
            }
            i += 1;
            continue;
        }

        if ch == '\'' {
            in_single = true;
            i += 1;
            continue;
        }
        if ch == '"' {
            in_double = true;
            i += 1;
            continue;
        }

        if ch == '<' && chars[i + 1] == '<' {
            let mut j = i + 2;
            if j < chars.len() && chars[j] == '-' {
                j += 1;
            }
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            if j >= chars.len() {
                return None;
            }
            if chars[j] == '\'' || chars[j] == '"' {
                let quote = chars[j];
                j += 1;
                let start = j;
                while j < chars.len() && chars[j] != quote {
                    j += 1;
                }
                if j > start {
                    let delimiter: String = chars[start..j].iter().collect();
                    if contains_alpha(&delimiter) {
                        return Some(HeredocSpec { delimiter });
                    }
                }
            } else if is_heredoc_delimiter_char(chars[j]) {
                let start = j;
                while j < chars.len() && is_heredoc_delimiter_char(chars[j]) {
                    j += 1;
                }
                let len = j.saturating_sub(start);
                if len >= 2 {
                    let delimiter: String = chars[start..j].iter().collect();
                    if contains_alpha(&delimiter) {
                        return Some(HeredocSpec { delimiter });
                    }
                }
            }
        }

        i += 1;
    }

    None
}

fn is_heredoc_delimiter_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn contains_alpha(delimiter: &str) -> bool {
    delimiter.chars().any(|ch| ch.is_ascii_alphabetic())
}

#[cfg(test)]
mod tests {
    use super::repair_indented_terminators;

    #[test]
    fn repair_indented_terminator_moves_delimiter_to_column_one() {
        let input = "cat <<'EOF'\n  echo hello\n  EOF\n";
        let output = repair_indented_terminators(input);
        assert_eq!(output, "cat <<'EOF'\n  echo hello\nEOF\n");
    }
}
