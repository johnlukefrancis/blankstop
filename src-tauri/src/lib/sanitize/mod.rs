mod normalize;
mod clean;
mod js;
mod shell;
mod limits;
mod boundaries;
mod context;
mod heuristics;
mod lines;
mod profile;
mod wrap;
mod unwarp;

use serde::Serialize;

use lines::{build_line_meta, trim_blank_edges};
use clean::{clean_text, CleanResult};
use profile::estimate_wrap_profile;
use unwarp::unwarp_lines;
use wrap::{join_wrapped_single_line, should_rule_a};

#[derive(Debug, Default, Clone, Serialize)]
pub struct SanitizeSummary {
    pub unwrapped_lines: usize,
    pub trimmed_trailing_ws: usize,
    pub trimmed_blank_lines: usize,
    pub removed_invisibles: usize,
    pub stripped_prefixes: usize,
    pub repaired_string_wraps: usize,
    pub removed_cmd_carets: usize,
    pub joined_explicit_continuations: usize,
    pub js_validated: bool,
}

#[derive(Debug, Clone)]
pub struct SanitizeResult {
    pub output: String,
    #[cfg_attr(not(windows), allow(dead_code))]
    pub summary: SanitizeSummary,
}

pub fn sanitize_text(input: &str) -> SanitizeResult {
    let clean = clean_text(input);
    if limits::exceeds_max_clipboard_chars(&clean.text) {
        return limits::sanitize_large_input(clean);
    }
    if shell::looks_shell_like(&clean.text) {
        let shell_result = shell::reflow_shell(&clean.text);
        return SanitizeResult {
            output: shell_result.output,
            summary: SanitizeSummary {
                unwrapped_lines: shell_result.summary.unwrapped_lines,
                trimmed_trailing_ws: 0,
                trimmed_blank_lines: 0,
                removed_invisibles: clean.removed_invisibles,
                stripped_prefixes: clean.stripped_prefixes,
                repaired_string_wraps: shell_result.summary.repaired_string_wraps,
                removed_cmd_carets: shell_result.summary.removed_cmd_carets,
                joined_explicit_continuations: shell_result.summary.joined_explicit_continuations,
                js_validated: false,
            },
        };
    }
    if js::looks_js_like(&clean.text) {
        if let Some(output) = js::sanitize_js(&clean.text) {
            return SanitizeResult {
                output,
                summary: SanitizeSummary {
                    unwrapped_lines: 0,
                    trimmed_trailing_ws: 0,
                    trimmed_blank_lines: 0,
                    removed_invisibles: clean.removed_invisibles,
                    stripped_prefixes: clean.stripped_prefixes,
                    repaired_string_wraps: 0,
                    removed_cmd_carets: 0,
                    joined_explicit_continuations: 0,
                    js_validated: true,
                },
            };
        }
        return SanitizeResult {
            output: clean.normalized,
            summary: SanitizeSummary {
                unwrapped_lines: 0,
                trimmed_trailing_ws: 0,
                trimmed_blank_lines: 0,
                removed_invisibles: clean.removed_invisibles,
                stripped_prefixes: clean.stripped_prefixes,
                repaired_string_wraps: 0,
                removed_cmd_carets: 0,
                joined_explicit_continuations: 0,
                js_validated: false,
            },
        };
    }
    sanitize_text_text_mode_from_clean(clean)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn sanitize_text_text_mode(input: &str) -> SanitizeResult {
    let clean = clean_text(input);
    if limits::exceeds_max_clipboard_chars(&clean.text) {
        return limits::sanitize_large_input(clean);
    }
    sanitize_text_text_mode_from_clean(clean)
}

fn sanitize_text_text_mode_from_clean(clean: CleanResult) -> SanitizeResult {
    let (mut lines, trimmed_trailing_ws) = build_line_meta(&clean.text);
    let trimmed_blank_lines = trim_blank_edges(&mut lines);

    if lines.is_empty() {
        return SanitizeResult {
            output: String::new(),
            summary: SanitizeSummary {
                unwrapped_lines: 0,
                trimmed_trailing_ws,
                trimmed_blank_lines,
                removed_invisibles: clean.removed_invisibles,
                stripped_prefixes: clean.stripped_prefixes,
                repaired_string_wraps: 0,
                removed_cmd_carets: 0,
                joined_explicit_continuations: 0,
                js_validated: false,
            },
        };
    }

    let mut summary = SanitizeSummary {
        unwrapped_lines: 0,
        trimmed_trailing_ws,
        trimmed_blank_lines,
        removed_invisibles: clean.removed_invisibles,
        stripped_prefixes: clean.stripped_prefixes,
        repaired_string_wraps: 0,
        removed_cmd_carets: 0,
        joined_explicit_continuations: 0,
        js_validated: false,
    };

    let trimmed_lines: Vec<String> = lines.iter().map(|line| line.trimmed_end.clone()).collect();
    let profile = estimate_wrap_profile(&lines);

    let output = if should_rule_a(&trimmed_lines) {
        summary.unwrapped_lines = trimmed_lines.len().saturating_sub(1);
        join_wrapped_single_line(&trimmed_lines)
    } else {
        unwarp_lines(&lines, &profile, &mut summary)
    };

    SanitizeResult { output, summary }
}

impl SanitizeSummary {
    #[cfg_attr(not(windows), allow(dead_code))]
    pub fn toast_message(&self) -> String {
        let mut parts = Vec::new();
        if self.unwrapped_lines > 0 {
            parts.push(format!(
                "unwrapped {} line{}",
                self.unwrapped_lines,
                if self.unwrapped_lines == 1 { "" } else { "s" }
            ));
        }
        if self.trimmed_trailing_ws > 0 {
            parts.push(format!(
                "trimmed {} trailing space{}",
                self.trimmed_trailing_ws,
                if self.trimmed_trailing_ws == 1 { "" } else { "s" }
            ));
        }
        if self.trimmed_blank_lines > 0 {
            parts.push(format!(
                "trimmed {} blank line{}",
                self.trimmed_blank_lines,
                if self.trimmed_blank_lines == 1 { "" } else { "s" }
            ));
        }
        if self.removed_invisibles > 0 {
            parts.push(format!(
                "removed {} invisible char{}",
                self.removed_invisibles,
                if self.removed_invisibles == 1 { "" } else { "s" }
            ));
        }
        if self.stripped_prefixes > 0 {
            parts.push(format!(
                "stripped {} bullet prefix{}",
                self.stripped_prefixes,
                if self.stripped_prefixes == 1 { "" } else { "es" }
            ));
        }
        if self.repaired_string_wraps > 0 {
            parts.push(format!(
                "repaired {} string wrap{}",
                self.repaired_string_wraps,
                if self.repaired_string_wraps == 1 { "" } else { "s" }
            ));
        }
        if self.removed_cmd_carets > 0 {
            parts.push(format!(
                "removed {} cmd caret{}",
                self.removed_cmd_carets,
                if self.removed_cmd_carets == 1 { "" } else { "s" }
            ));
        }
        if self.joined_explicit_continuations > 0 {
            parts.push(format!(
                "joined {} explicit continuation{}",
                self.joined_explicit_continuations,
                if self.joined_explicit_continuations == 1 { "" } else { "s" }
            ));
        }
        if parts.is_empty() {
            "Clipboard sanitized".to_string()
        } else {
            format!("Clipboard sanitized: {}", parts.join(", "))
        }
    }
}

#[cfg(test)]
mod tests;
