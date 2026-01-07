use super::boundaries::{decide_join, BoundaryContext};
use super::lines::LineMeta;
use super::profile::WrapProfile;
use super::SanitizeSummary;

pub fn unwarp_lines(lines: &[LineMeta], profile: &WrapProfile, summary: &mut SanitizeSummary) -> String {
    let mut out_lines = Vec::new();
    let mut i = 0usize;
    while i < lines.len() {
        let mut current = lines[i].trimmed_end.clone();
        let mut prev_index = i;
        while prev_index + 1 < lines.len() {
            let next_index = prev_index + 1;
            let decision = decide_join(
                &lines[prev_index],
                &lines[next_index],
                profile,
                BoundaryContext {
                    prev_index,
                    next_index,
                    total: lines.len(),
                },
            );
            if !decision.join {
                break;
            }
            let next_trim = lines[next_index].trimmed_end.trim_start();
            if decision.insert_space {
                current.push(' ');
            }
            current.push_str(next_trim);
            summary.unwrapped_lines += 1;
            prev_index += 1;
        }
        out_lines.push(current);
        i = prev_index + 1;
    }
    out_lines.join("\n")
}
