mod normalize;
mod boundaries;
mod heuristics;
mod lines;
mod profile;
mod wrap;
mod unwarp;

use serde::Serialize;

use lines::{build_line_meta, trim_blank_edges};
use normalize::normalize_line_endings;
use profile::estimate_wrap_profile;
use unwarp::unwarp_lines;
use wrap::{join_wrapped_single_line, should_rule_a};

#[derive(Debug, Default, Clone, Serialize)]
pub struct SanitizeSummary {
    pub unwrapped_lines: usize,
    pub trimmed_trailing_ws: usize,
    pub trimmed_blank_lines: usize,
}

#[derive(Debug, Clone)]
pub struct SanitizeResult {
    pub output: String,
    pub summary: SanitizeSummary,
}

pub fn sanitize_text(input: &str) -> SanitizeResult {
    let normalized = normalize_line_endings(input);
    let (mut lines, trimmed_trailing_ws) = build_line_meta(&normalized);
    let trimmed_blank_lines = trim_blank_edges(&mut lines);

    if lines.is_empty() {
        return SanitizeResult {
            output: String::new(),
            summary: SanitizeSummary {
                unwrapped_lines: 0,
                trimmed_trailing_ws,
                trimmed_blank_lines,
            },
        };
    }

    let mut summary = SanitizeSummary {
        unwrapped_lines: 0,
        trimmed_trailing_ws,
        trimmed_blank_lines,
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
        if parts.is_empty() {
            "Sanitized clipboard".to_string()
        } else {
            format!("Sanitized clipboard: {}", parts.join(", "))
        }
    }
}

#[cfg(test)]
mod tests;
