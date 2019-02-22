use wasm_bindgen::prelude::*;

use lazy_static::lazy_static;

use std::collections::BTreeMap;
use std::sync::Mutex;

use rand::rngs::SmallRng;
use rand::FromEntropy;

mod weighted_choice;
use weighted_choice::WeightedIndex4;

mod sim;
mod stats;

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug, num_enum::IntoPrimitive, num_enum::CustomTryInto)]
enum Pool {
    Focus,
    Fivestar,
    Fourstar,
    Threestar,
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug, num_enum::IntoPrimitive, num_enum::CustomTryInto)]
enum Color {
    Red,
    Blue,
    Green,
    Colorless,
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug, num_enum::IntoPrimitive, num_enum::CustomTryInto)]
pub enum GoalKind {
    AnyFivestar,
    AnyFocus,
    RedFocus,
    BlueFocus,
    GreenFocus,
    ColorlessFocus,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Goal {
    kind: GoalKind,
    count: u32,
}

lazy_static! {
    static ref DATA: Mutex<BTreeMap<u32, u32>> = Mutex::new(BTreeMap::new());
    static ref RNG: Mutex<SmallRng> = Mutex::new(SmallRng::from_entropy());
    static ref POOL_SIZES: [[u8; 4]; 4] = [
        [0, 0, 0, 0],
        [41, 28, 21, 17],
        [32, 29, 20, 28],
        [28, 25, 18, 25],
    ];
    static ref POOL_DISTS: Mutex<[WeightedIndex4; 26]> =
        Mutex::new([WeightedIndex4::default(); 26]);
    static ref COLOR_DISTS: Mutex<[WeightedIndex4; 4]> = Mutex::new([WeightedIndex4::default(); 4]);
    static ref FOCUS_SIZES: Mutex<[u8; 4]> = Mutex::new([1; 4]);
    static ref STARTING_RATES: Mutex<(u8, u8)> = Mutex::new((3, 3));
}

#[wasm_bindgen]
pub fn init_banner(r: u8, b: u8, g: u8, c: u8, rate_f: u8, rate_5: u8) -> bool {
    DATA.lock().unwrap().clear();
    *FOCUS_SIZES.lock().unwrap() = [r, b, g, c];
    *STARTING_RATES.lock().unwrap() = (rate_f, rate_5);

    sim::init_probability_tables();

    true
}

#[wasm_bindgen]
pub fn run(num_samples: u32, goal_kind: u8, count: u32) {
    let kind = goal_kind
        .try_into_GoalKind()
        .unwrap_or(GoalKind::AnyFivestar);
    let mut results = DATA.lock().unwrap();
    for _ in 0..num_samples {
        let result = sim::roll_until(Goal { kind, count });
        *results.entry(result).or_insert(0) += 1;
    }
}

#[wasm_bindgen]
pub fn results() -> String {
    let percentiles = [0.25, 0.5, 0.75, 0.9, 0.99];
    let mut retval = String::new();
    for &pct in &percentiles {
        retval.push_str(&format!("\n{}: {}", pct, stats::percentile(pct)));
    }
    retval
}

#[wasm_bindgen]
pub fn clear_data() {
    DATA.lock().unwrap().clear();
}
