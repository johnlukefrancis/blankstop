pub(super) fn has_heredoc_operator(line: &str) -> bool {
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
            if i > 0 && !chars[i - 1].is_whitespace() {
                i += 2;
                continue;
            }

            let mut j = i + 2;
            if j < chars.len() && chars[j] == '-' {
                j += 1;
            }
            if j >= chars.len() {
                return false;
            }
            if chars[j].is_whitespace() {
                i += 2;
                continue;
            }

            let next = chars[j];
            if next == '\'' || next == '"' {
                let quote = next;
                j += 1;
                let start = j;
                while j < chars.len() && chars[j] != quote {
                    j += 1;
                }
                if j > start + 1 {
                    return true;
                }
            } else if is_heredoc_delimiter_char(next) {
                let start = j;
                while j < chars.len() && is_heredoc_delimiter_char(chars[j]) {
                    j += 1;
                }
                if j - start >= 2 {
                    return true;
                }
            }
        }

        i += 1;
    }

    false
}

pub(super) fn powershell_signal_score(line: &str) -> usize {
    let tokens = tokenize_command_like(line);
    let mut score = 0usize;
    let mut saw_exe = false;
    let mut saw_pwsh = false;

    for token in tokens {
        let normalized = token.to_ascii_lowercase();
        if normalized.contains("powershell.exe") {
            saw_exe = true;
        }
        if normalized == "pwsh" || normalized.contains("pwsh.exe") || normalized.ends_with("\\pwsh") {
            saw_pwsh = true;
        }
        if normalized == "-noprofile" {
            score += 2;
        }
        if normalized == "-command" {
            score += 2;
        }
    }

    if saw_exe || saw_pwsh {
        score += 3;
    }

    score
}

fn is_heredoc_delimiter_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn tokenize_command_like(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if in_single {
            if ch == '\'' {
                in_single = false;
            } else {
                current.push(ch);
            }
            continue;
        }
        if in_double {
            if ch == '"' {
                in_double = false;
            } else if ch == '\\' {
                if let Some('"') = chars.peek().copied() {
                    chars.next();
                    current.push('"');
                } else {
                    current.push(ch);
                }
            } else {
                current.push(ch);
            }
            continue;
        }

        if ch.is_whitespace() {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
            continue;
        }
        if ch == '\'' {
            in_single = true;
            continue;
        }
        if ch == '"' {
            in_double = true;
            continue;
        }

        current.push(ch);
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}
