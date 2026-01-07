use super::sanitize_text;

#[test]
fn wrapped_commit_subject_example() {
    let input = "🟦 improve(docs): document HUD GPU clear state restore\n  requirement\n";
    let result = sanitize_text(input);
    assert_eq!(
        result.output,
        "🟦 improve(docs): document HUD GPU clear state restore requirement"
    );
}

#[test]
fn multiline_code_preserved() {
    let input = "const a = 1;\nconst b = 2;\n";
    let result = sanitize_text(input);
    assert_eq!(result.output, "const a = 1;\nconst b = 2;");
}

#[test]
fn soft_wrap_join_simulation() {
    let input = "checksum: Array.from({ length: 24 }, (_, i) => ((i * 7 + 13) % 97)).jo\nin(',')\n";
    let result = sanitize_text(input);
    assert_eq!(
        result.output,
        "checksum: Array.from({ length: 24 }, (_, i) => ((i * 7 + 13) % 97)).join(',')"
    );
}
