use crate::counter::Counter;

pub fn percentile(data: &Counter, pct: f32) -> u32 {
    debug_assert!(pct >= 0.0);
    debug_assert!(pct <= 1.0);

    let total: u32 = data.iter().sum();

    if total == 0 {
        return 0;
    }

    let mut accum_total = 0;
    for idx in 0..data.len() as u32 {
        accum_total += data[idx];
        if accum_total as f32 / total as f32 > pct {
            return idx;
        }
    }

    // Pct is 100% (or close enough for rounding errors) if it didn't already finish,
    // so grab the last non-zero value.
    for &value in data.iter().rev() {
        if value > 0 {
            return value;
        }
    }

    // It would have returned early if all entries were zero, so the above loop
    // is guaranteed to find something and exit.
    unreachable!()
}
