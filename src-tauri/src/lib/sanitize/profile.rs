use super::lines::LineMeta;

#[derive(Clone, Copy, Debug)]
pub struct WrapProfile {
    pub wrap_width: Option<usize>,
    pub slack: usize,
}

pub fn estimate_wrap_profile(lines: &[LineMeta]) -> WrapProfile {
    // Invariant: estimate from raw line lengths (including padding) using a tolerant ±2 cluster.
    let mut long_lengths = Vec::new();
    for line in lines {
        if line.trimmed_end.trim().is_empty() {
            continue;
        }
        if line.raw_len_chars >= 40 {
            long_lengths.push(line.raw_len_chars);
        }
    }

    let wrap_width = if long_lengths.len() < 2 {
        None
    } else {
        let mut best_cluster: Vec<usize> = Vec::new();
        for center in &long_lengths {
            let cluster: Vec<usize> = long_lengths
                .iter()
                .cloned()
                .filter(|len| (*len as isize - *center as isize).abs() <= 2)
                .collect();
            if cluster.len() > best_cluster.len() {
                best_cluster = cluster;
            }
        }
        if best_cluster.len() < 2 {
            None
        } else {
            let sum: usize = best_cluster.iter().sum();
            Some((sum + best_cluster.len() / 2) / best_cluster.len())
        }
    };

    WrapProfile {
        wrap_width,
        slack: 4,
    }
}
