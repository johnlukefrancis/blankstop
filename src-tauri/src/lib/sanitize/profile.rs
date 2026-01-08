use super::lines::LineMeta;

#[derive(Clone, Copy, Debug)]
pub struct WrapProfile {
    pub wrap_width: Option<usize>,
    pub slack: usize,
}

pub fn estimate_wrap_profile(lines: &[LineMeta]) -> WrapProfile {
    // Invariant: wrap width is optional and based on a tail of raw lengths (padding helps but is not required).
    let mut lengths = Vec::new();
    for line in lines {
        if line.trimmed_end.trim().is_empty() {
            continue;
        }
        lengths.push(line.raw_len_chars);
    }

    let wrap_width = if lengths.len() < 2 {
        None
    } else {
        let max_len = *lengths.iter().max().unwrap();
        let tail: Vec<usize> = lengths
            .into_iter()
            .filter(|len| *len + 8 >= max_len)
            .collect();
        if tail.len() >= 2 {
            let mut sorted = tail;
            sorted.sort_unstable();
            Some(sorted[sorted.len() / 2])
        } else {
            None
        }
    };

    WrapProfile {
        wrap_width,
        slack: 4,
    }
}
