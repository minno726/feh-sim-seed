use crate::counter::Counter;

pub fn percentile(data: &Counter, pct: f32) -> u32 {
    percentiles(data, &[pct])[0]
}

pub fn percentiles(data: &Counter, pcts: &[f32]) -> Vec<u32> {
    debug_assert!(pcts.iter().all(|&x| x >= 0.0 && x <= 1.0));
    debug_assert!((0..pcts.len() - 1).all(|idx| pcts[idx + 1] >= pcts[idx]));

    let total: u32 = data.iter().sum();
    let mut results = vec![0; pcts.len()];

    if total == 0 {
        return results;
    }

    let mut accum_total = 0;
    let mut out_idx = 0;
    for value in 0..data.len() as u32 {
        accum_total += data[value];
        while out_idx < results.len() && accum_total as f32 / total as f32 > pcts[out_idx] {
            results[out_idx] = value;
            out_idx += 1;
        }
    }

    if out_idx == results.len() {
        return results;
    }

    // The remaining values in pcts are 100% (or close enough for rounding errors)
    // if it didn't already finish, so grab the last non-zero value and fill the
    // rest of the results.
    for &value in data.iter().rev() {
        if value > 0 {
            for i in out_idx..results.len() {
                results[i] = value;
            }
            return results;
        }
    }

    // It would have returned early if all entries were zero, so the above loop
    // is guaranteed to find something and exit.
    unreachable!()
}