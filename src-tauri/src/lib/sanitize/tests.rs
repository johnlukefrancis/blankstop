use super::sanitize_text;

fn pad_line(content: &str, total_len: usize) -> String {
    let len = content.chars().count();
    if len >= total_len {
        content.to_string()
    } else {
        format!("{content}{}", " ".repeat(total_len - len))
    }
}

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

#[test]
fn does_not_flatten_short_code_block() {
    let input = "if (foo) {\n  bar();\n}\n";
    let result = sanitize_text(input);
    assert_eq!(result.output, "if (foo) {\n  bar();\n}");
}

#[test]
fn unwraps_single_wrapped_line_inside_large_snippet() {
    let input = concat!(
        "const values = [1, 2, 3, 4, 5];\n",
        "const label = \"alpha\";\n",
        "const joined = items.map((item) => item.trim()).jo\n",
        "in(',');\n",
        "return { label, joined, values };\n"
    );
    let result = sanitize_text(input);
    assert_eq!(
        result.output,
        concat!(
            "const values = [1, 2, 3, 4, 5];\n",
            "const label = \"alpha\";\n",
            "const joined = items.map((item) => item.trim()).join(',');\n",
            "return { label, joined, values };"
        )
    );
}

#[test]
fn unwraps_multiple_wraps_without_poisoning_width() {
    let width = 64;
    let report_line = pad_line("const report = {", width);
    let screen_line = pad_line("  screen: safe(() => pick(screen,", width);
    let fallback_line = pad_line("  fallback: safe(() => value ? pick(data,", width);
    let input = format!(
        "{report_line}\n{screen_line}\n  [\"width\",\"height\",\"availWidth\",\"availHeight\"], null),\n{fallback_line}\n  [\"used\",\"total\",\"limit\"]) : null, null),\n}};\n"
    );
    let result = sanitize_text(&input);
    assert_eq!(
        result.output,
        concat!(
            "const report = {\n",
            "  screen: safe(() => pick(screen, [\"width\",\"height\",\"availWidth\",\"availHeight\"], null),\n",
            "  fallback: safe(() => value ? pick(data, [\"used\",\"total\",\"limit\"]) : null, null),\n",
            "};"
        )
    );
}

#[test]
fn does_not_join_return_fallback_identifier() {
    let input = "return\nfallback\n";
    let result = sanitize_text(input);
    assert_eq!(result.output, "return\nfallback");
}

#[test]
fn unwraps_return_with_indented_fallback() {
    let width = 64;
    let header = pad_line("const header = \"this is a fairly long line to anchor wrap width\";", width);
    let note = pad_line("const note = \"another long line to keep widths consistent\";", width);
    let result_line = pad_line("const result = computeSomethingVerbose(return", width);
    let input = format!("{header}\n{note}\n{result_line}\n  fallback);\n");
    let result = sanitize_text(&input);
    assert_eq!(
        result.output,
        concat!(
            "const header = \"this is a fairly long line to anchor wrap width\";\n",
            "const note = \"another long line to keep widths consistent\";\n",
            "const result = computeSomethingVerbose(return fallback);"
        )
    );
}

#[test]
fn unwraps_pick_screen_with_bracket() {
    let width = 64;
    let header = pad_line("const header = \"this is a fairly long line to anchor wrap width\";", width);
    let note = pad_line("const note = \"another long line to keep widths consistent\";", width);
    let metrics_line = pad_line("const metrics = pick(screen,", width);
    let input = format!("{header}\n{note}\n{metrics_line}\n[\"width\"]);");
    let result = sanitize_text(&input);
    assert_eq!(
        result.output,
        concat!(
            "const header = \"this is a fairly long line to anchor wrap width\";\n",
            "const note = \"another long line to keep widths consistent\";\n",
            "const metrics = pick(screen, [\"width\"]);"
        )
    );
}

#[test]
fn unwraps_emoji_punctuation_wraps() {
    let width = 64;
    let header = pad_line(
        "const header = \"🚀 release: punctuation, commas, semicolons; brackets [a, b, c]\";",
        width,
    );
    let note = pad_line("const note = \"status: ok, version: 1.2.3, build: 4567\";", width);
    let detail = pad_line("const detail = \"emoji 🚀 and punctuation: [one,", width);
    let input = format!("{header}\n{note}\n{detail}\n two, three]\";\n");
    let result = sanitize_text(&input);
    assert_eq!(
        result.output,
        concat!(
            "const header = \"🚀 release: punctuation, commas, semicolons; brackets [a, b, c]\";\n",
            "const note = \"status: ok, version: 1.2.3, build: 4567\";\n",
            "const detail = \"emoji 🚀 and punctuation: [one, two, three]\";"
        )
    );
}

#[test]
fn padded_code_block_preserves_newlines() {
    let width = 64;
    let line1 = pad_line("if (foo) {", width);
    let line2 = pad_line("  bar();", width);
    let line3 = pad_line("}", width);
    let input = format!("{line1}\n{line2}\n{line3}\n");
    let result = sanitize_text(&input);
    assert_eq!(result.output, "if (foo) {\n  bar();\n}");
}
