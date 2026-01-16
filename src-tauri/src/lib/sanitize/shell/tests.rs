use crate::sanitize::sanitize_text;

#[test]
fn power_shell_wrapped_path_literal_is_unwrapped_without_spaces() {
    let input = concat!(
        "PS> wsl.exe -e bash -lc \"cd /home/johnf/code/\n",
        "  textureportal && ls\"\n",
    );
    let result = sanitize_text(input);
    assert!(result.output.contains("/home/johnf/code/textureportal"));
    assert!(!result.output.contains("/home/johnf/code/ textureportal"));
}

#[test]
fn power_shell_wrapped_unc_path_is_unwrapped() {
    let input = concat!(
        "PS> cd \"\\\\wsl$\n",
        "    \\\\Ubuntu\\\\home\\\\johnf\"\n",
    );
    let result = sanitize_text(input);
    assert_eq!(
        result.output.trim_end(),
        "PS> cd \"\\\\wsl$\\\\Ubuntu\\\\home\\\\johnf\""
    );
}

#[test]
fn cmd_caret_tokens_are_removed_at_token_start() {
    let input = concat!(
        "cmd /c powershell -Command ^\n",
        "  \"Write-Host 'hi';\" ^\n",
        "  \"[Environment]::Exit(0)\"\n",
    );
    let result = sanitize_text(input);
    assert!(!result.output.contains('^'));
    assert!(
        result
            .output
            .contains("-Command \"Write-Host 'hi';\" \"[Environment]::Exit(0)\"")
    );
}

#[test]
fn heredoc_terminator_is_repaired_and_body_preserved() {
    let input = "cat > /tmp/x <<'EOF'\n  echo hi\n  EOF\n";
    let result = sanitize_text(input);
    assert!(result.output.contains("\nEOF\n"));
    assert!(result.output.contains("\n  echo hi\n"));
}

#[test]
fn multiple_heredoc_terminators_are_repaired_independently() {
    let input = concat!(
        "cat > /tmp/a <<'ONE'\n",
        "  echo a\n",
        "  ONE\n",
        "cat > /tmp/b <<CLIP\n",
        "  echo b\n",
        "  CLIP\n",
    );
    let result = sanitize_text(input);
    assert!(result.output.contains("\nONE\n"));
    assert!(result.output.contains("\nCLIP\n"));
}

#[test]
fn quoted_path_trailing_backslash_is_preserved() {
    let input = concat!(
        "echo \"\\\\wsl$\\\\Ubuntu\\\\home\\\\johnf\\\\code\\\\\n",
        "  textureportal\"\n",
    );
    let result = sanitize_text(input);
    assert!(result
        .output
        .contains("\"\\\\wsl$\\\\Ubuntu\\\\home\\\\johnf\\\\code\\\\textureportal\""));
}

#[test]
fn cmd_caret_inside_quotes_is_preserved() {
    let input = "cmd /c echo \"caret ^ literal\"\n";
    let result = sanitize_text(input);
    assert!(result.output.contains("caret ^ literal"));
}

#[test]
fn shell_wrapped_path_tokens_are_stitched_without_spaces() {
    let input = concat!(
        "PS> ls assets/\n",
        "  organized/procedural/seamless_void.png\n",
    );
    let result = sanitize_text(input);
    assert!(result
        .output
        .contains("assets/organized/procedural/seamless_void.png"));
    assert_eq!(result.summary.unwrapped_lines, 1);
}
