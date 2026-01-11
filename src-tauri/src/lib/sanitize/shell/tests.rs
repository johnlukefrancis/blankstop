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
        "  \"Write-Host 'hi'; ^\n",
        "  [Environment]::Exit(0)\"\n",
    );
    let result = sanitize_text(input);
    assert!(!result.output.contains('^'));
    assert!(
        result
            .output
            .contains("-Command \"Write-Host 'hi'; [Environment]::Exit(0)\"")
    );
}
