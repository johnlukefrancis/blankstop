use super::heuristics::is_identifier_split;
use super::lines::LineMeta;
use super::profile::WrapProfile;

#[derive(Clone, Copy, Debug)]
pub struct BoundaryContext {
    pub prev_index: usize,
    pub next_index: usize,
    pub total: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct JoinDecision {
    pub join: bool,
    pub insert_space: bool,
}

pub fn decide_join(prev: &LineMeta, next: &LineMeta, profile: &WrapProfile, _ctx: BoundaryContext) -> JoinDecision {
    let prev_trim = prev.trimmed_end.as_str();
    let next_trim = next.trimmed_end.as_str();

    if prev_trim.trim().is_empty() || next_trim.trim().is_empty() {
        return JoinDecision {
            join: false,
            insert_space: false,
        };
    }

    if prev_trim.ends_with('{') && next.leading_ws_count > prev.leading_ws_count {
        return JoinDecision {
            join: false,
            insert_space: false,
        };
    }

    if next_trim.trim_start().starts_with('}') {
        return JoinDecision {
            join: false,
            insert_space: false,
        };
    }

    if prev_trim.ends_with(';') && next.leading_ws_count <= prev.leading_ws_count {
        return JoinDecision {
            join: false,
            insert_space: false,
        };
    }

    // Invariants: require wrap evidence (near width or padded) and a continuation signal.
    let mut wrap_evidence = prev.trailing_ws_count > 0;
    if let Some(wrap_width) = profile.wrap_width {
        let slack = profile.slack as isize;
        let delta = prev.raw_len_chars as isize - wrap_width as isize;
        if delta.abs() <= slack {
            wrap_evidence = true;
        }
    }

    let next_trim_start = next_trim.trim_start();
    let identifier_split = is_identifier_split(prev_trim, next_trim_start);
    let continuation_signal = identifier_split
        || next.leading_ws_count > 0
        || next_trim_start.starts_with('[')
        || next_trim_start.starts_with('(')
        || next_trim_start.starts_with('.')
        || prev_trim.ends_with(',');

    if !wrap_evidence && !identifier_split {
        return JoinDecision {
            join: false,
            insert_space: false,
        };
    }

    if !continuation_signal {
        return JoinDecision {
            join: false,
            insert_space: false,
        };
    }

    let insert_space = should_insert_space(prev_trim, next_trim_start);
    JoinDecision {
        join: true,
        insert_space,
    }
}

fn should_insert_space(prev: &str, next: &str) -> bool {
    let prev_trim = prev.trim_end();
    let next_trim = next.trim_start();
    if prev_trim.is_empty() || next_trim.is_empty() {
        return false;
    }
    if prev_trim.ends_with(|c: char| c.is_whitespace())
        || next_trim.starts_with(|c: char| c.is_whitespace())
    {
        return false;
    }
    if is_identifier_split(prev_trim, next_trim) {
        return false;
    }
    true
}
