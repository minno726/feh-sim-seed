use crate::*;

pub(crate) fn percentile(pct: f32) -> u32 {
    let data = DATA.lock().unwrap();
    let total: u32 = data.values().sum();
    let mut accum_total = 0;
    for (&key, &value) in &*data {
        accum_total += value;
        if accum_total as f32 / total as f32 > pct {
            return key;
        }
    }
    *data.values().next_back().unwrap()
}
