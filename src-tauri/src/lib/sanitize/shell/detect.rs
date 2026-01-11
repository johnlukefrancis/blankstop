use super::detect_helpers::{has_heredoc_operator, powershell_signal_score};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellFlavor {
    Bash,
    PowerShell,
    Cmd,
}

pub fn looks_shell_like(text: &str) -> bool {
    detect_shell_flavor(text).is_some()
}

fn detect_shell_flavor(text: &str) -> Option<ShellFlavor> {
    let trimmed = text.trim();
    if trimmed.is_empty() || !trimmed.contains('\n') {
        return None;
    }

    let mut bash_score = 0usize;
    let mut ps_score = 0usize;
    let mut cmd_score = 0usize;

    for line in trimmed.lines() {
        let line_trim = line.trim_start();
        if line_trim.starts_with("PS ") || line_trim.starts_with("PS>") {
            ps_score += 3;
        }
        if is_drive_prompt(line_trim) {
            cmd_score += 3;
        }
        if line_trim.starts_with("$ ") || line_trim.starts_with("# ") {
            bash_score += 2;
        }
        if line_trim.starts_with("$env:") || line_trim.starts_with("$Env:") {
            ps_score += 3;
        }
        if is_powershell_assignment(line_trim) {
            ps_score += 2;
        }
        if has_powershell_verb(line_trim) {
            ps_score += 2;
        }
        if line_trim.starts_with("export ") || line_trim.starts_with("sudo ") || line_trim.starts_with("cd ") {
            bash_score += 1;
        }
        if line_trim.starts_with("set ") || line_trim.starts_with("dir") || line_trim.starts_with("copy ") {
            cmd_score += 1;
        }
        if line_trim.contains("$(") || line_trim.contains("${") {
            bash_score += 1;
        }
        if has_heredoc_operator(line_trim) {
            bash_score += 3;
        }
        let ps_signal = powershell_signal_score(line_trim);
        if ps_signal > 0 {
            ps_score += ps_signal;
        }
        if let Some(kind) = continuation_kind(line_trim) {
            match kind {
                ShellFlavor::Bash => bash_score += 2,
                ShellFlavor::PowerShell => ps_score += 2,
                ShellFlavor::Cmd => cmd_score += 2,
            }
        }
    }

    if trimmed.contains("\\\\wsl$") {
        ps_score += 2;
        cmd_score += 1;
    }

    let (flavor, score) = max_score(bash_score, ps_score, cmd_score);
    if score >= 2 { Some(flavor) } else { None }
}

fn max_score(bash: usize, ps: usize, cmd: usize) -> (ShellFlavor, usize) {
    if ps >= bash && ps >= cmd {
        return (ShellFlavor::PowerShell, ps);
    }
    if bash >= cmd {
        return (ShellFlavor::Bash, bash);
    }
    (ShellFlavor::Cmd, cmd)
}

fn has_powershell_verb(line: &str) -> bool {
    let verbs = [
        "Get-", "Set-", "New-", "Remove-", "Write-", "Select-", "Start-", "Stop-", "Test-",
        "Invoke-", "Import-",
    ];
    verbs.iter().any(|verb| line.starts_with(verb))
}

fn is_powershell_assignment(line: &str) -> bool {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('$') {
        return false;
    }
    if trimmed.starts_with("$ ") {
        return false;
    }
    trimmed[1..].contains('=')
}

fn is_drive_prompt(line: &str) -> bool {
    let bytes = line.as_bytes();
    if bytes.len() < 3 {
        return false;
    }
    let drive = bytes[0];
    if !drive.is_ascii_alphabetic() {
        return false;
    }
    bytes[1] == b':' && bytes[2] == b'\\'
}

fn continuation_kind(line: &str) -> Option<ShellFlavor> {
    let trimmed = line.trim_end();
    let last = trimmed.chars().last()?;
    match last {
        '\\' => Some(ShellFlavor::Bash),
        '`' => Some(ShellFlavor::PowerShell),
        '^' => Some(ShellFlavor::Cmd),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::looks_shell_like;

    #[test]
    fn looks_shell_like_detects_heredoc_without_prompts() {
        let snippet = "cat <<'EOF'\necho hello\nEOF\n";
        assert!(looks_shell_like(snippet));
    }
}
