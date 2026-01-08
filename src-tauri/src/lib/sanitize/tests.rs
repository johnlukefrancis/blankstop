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
    let input = concat!(
        "const report = {\n",
        "  screen: safe(() => pick(screen,\n",
        "  [\"width\",\"height\",\"availWidth\",\"availHeight\"], null),\n",
        "  fallback: safe(() => value ? pick(data,\n",
        "  [\"used\",\"total\",\"limit\"]) : null, null),\n",
        "};\n"
    );
    let result = sanitize_text(input);
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
    let input = "const result = computeSomethingVerbose(return\n  fallback);\n";
    let result = sanitize_text(input);
    assert_eq!(
        result.output,
        "const result = computeSomethingVerbose(return fallback);"
    );
}

#[test]
fn unwraps_pick_screen_with_bracket() {
    let input = "const metrics = pick(screen,\n  [\"width\"]);\n";
    let result = sanitize_text(input);
    assert_eq!(
        result.output,
        "const metrics = pick(screen, [\"width\"]);"
    );
}

#[test]
fn unwraps_emoji_punctuation_wraps() {
    let input = concat!(
        "const header = \"🚀 release: punctuation, commas, semicolons; brackets [a, b, c]\";\n",
        "const note = \"status: ok, version: 1.2.3, build: 4567\";\n",
        "const detail = \"emoji 🚀 and punctuation: [one,\n",
        "  two, three]\";\n"
    );
    let result = sanitize_text(input);
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
fn does_not_join_object_properties_at_depth0() {
    let input = concat!(
        "const report = {\n",
        "  href: \"example\",\n",
        "  userAgent: \"ua\",\n",
        "};\n"
    );
    let result = sanitize_text(input);
    assert_eq!(
        result.output,
        concat!(
            "const report = {\n",
            "  href: \"example\",\n",
            "  userAgent: \"ua\",\n",
            "};"
        )
    );
}

#[test]
fn does_not_join_dedented_property_start() {
    let input = concat!(
        "const report = {\n",
        "  screen: safe(() => pick(screen,\n",
        "  [\"width\"])),\n",
        "trianglerain_globals: {\n",
        "  enabled: true,\n",
        "}\n"
    );
    let result = sanitize_text(input);
    assert_eq!(
        result.output,
        concat!(
            "const report = {\n",
            "  screen: safe(() => pick(screen, [\"width\"])),\n",
            "trianglerain_globals: {\n",
            "  enabled: true,\n",
            "}"
        )
    );
}
