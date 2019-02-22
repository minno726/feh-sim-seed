use crate::*;

use rand::distributions::Distribution;
use rand::Rng;

struct SessionResult {
    chosen_count: u32,
    goal_count: u32,
    reset: bool,
}

pub(crate) fn init_probability_tables() {
    let mut pool_sizes = POOL_SIZES.clone();
    pool_sizes[0] = *FOCUS_SIZES.lock().unwrap();

    for color in 0..4 {
        COLOR_DISTS.lock().unwrap()[color] = WeightedIndex4::new(pool_sizes[color]);
    }

    for pity_incr in 0..26 {
        POOL_DISTS.lock().unwrap()[pity_incr] =
            WeightedIndex4::new(probabilities(pity_incr as u32));
    }
}

// Simulates until reaching the specified goal, then returns # of orbs used
pub(crate) fn roll_until(goal: Goal) -> u32 {
    let mut pity_count = 0;
    let mut orb_count = 0;
    let mut curr_goal_count = 0;
    loop {
        let pity_incr = pity_count / 5;
        let samples = [
            sample(pity_incr),
            sample(pity_incr),
            sample(pity_incr),
            sample(pity_incr),
            sample(pity_incr),
        ];
        let SessionResult {
            chosen_count,
            goal_count,
            reset,
        } = session_select(goal, &samples);
        curr_goal_count += goal_count;
        if reset {
            pity_count = 0;
        } else {
            pity_count += chosen_count;
        }
        orb_count += orb_cost(chosen_count);
        if curr_goal_count >= goal.count {
            return orb_count;
        }
    }
}

// Returns: number of items picked, number of items meeting the goal,
// and whether the rate reset
fn session_select(goal: Goal, samples: &[(Pool, Color); 5]) -> SessionResult {
    let mut chosen_count = 0;
    let mut goal_count = 0;
    let mut reset = false;
    for i in 0..5 {
        let sample = samples[i];
        if may_match_goal(goal, sample) {
            chosen_count += 1;
            if does_match_goal(goal, sample) {
                goal_count += 1;
                if goal_count >= goal.count {
                    return SessionResult {
                        chosen_count,
                        goal_count,
                        reset: true,
                    };
                }
            }
            if sample.0 == Pool::Focus || sample.0 == Pool::Fivestar {
                reset = true;
            }
        }
    }
    if chosen_count == 0 {
        // None with the color we want, so pick randomly
        let sample = samples[RNG.lock().unwrap().gen::<usize>() % samples.len()];
        if sample.0 == Pool::Focus || sample.0 == Pool::Fivestar {
            reset = true;
        }
        chosen_count = 1;
    }
    SessionResult {
        chosen_count,
        goal_count,
        reset,
    }
}

fn may_match_goal(goal: Goal, sample: (Pool, Color)) -> bool {
    use GoalKind::*;
    match goal.kind {
        AnyFivestar | AnyFocus => FOCUS_SIZES.lock().unwrap()[sample.1 as usize] > 0,
        RedFocus => sample.1 == Color::Red,
        BlueFocus => sample.1 == Color::Blue,
        GreenFocus => sample.1 == Color::Green,
        ColorlessFocus => sample.1 == Color::Colorless,
    }
}

fn does_match_goal(goal: Goal, sample: (Pool, Color)) -> bool {
    use GoalKind::*;
    if sample.0 == Pool::Fourstar || sample.0 == Pool::Threestar {
        return false;
    }
    let color = match goal.kind {
        // Can quit early, no need to check banner.
        AnyFivestar => return true,
        AnyFocus => return sample.0 == Pool::Focus,
        // Need to extract the color, to check against banner focus counts
        RedFocus => Color::Red,
        BlueFocus => Color::Blue,
        GreenFocus => Color::Green,
        ColorlessFocus => Color::Colorless,
    };
    if sample.0 != Pool::Focus {
        return false;
    }
    let focus_count = FOCUS_SIZES.lock().unwrap()[color as usize];
    if focus_count == 1 {
        true
    } else {
        // Equal chance among all same-color focus units
        return RNG.lock().unwrap().gen::<f32>() < 1.0 / focus_count as f32;
    }
}

fn orb_cost(count: u32) -> u32 {
    match count {
        1 => 5,
        2 => 9,
        3 => 13,
        4 => 17,
        5 => 20,
        _ => panic!("Invalid orb cost: {}", count),
    }
}

fn sample(pity_incr: u32) -> (Pool, Color) {
    let pool =
        POOL_DISTS.lock().unwrap()[pity_incr as usize].sample(&mut *RNG.lock().unwrap()) as u8;
    let color = COLOR_DISTS.lock().unwrap()[pool as usize].sample(&mut *RNG.lock().unwrap()) as u8;
    (
        pool.try_into_Pool().unwrap(),
        color.try_into_Color().unwrap(),
    )
}

// [focus, fivestar, fourstar, threestar]
fn probabilities(pity_incr: u32) -> [f32; 4] {
    let bases = bases();
    let pity_pct = if pity_incr >= 25 {
        100.0 - bases[Pool::Focus as usize] - bases[1]
    } else {
        pity_incr as f32 * 0.5
    };

    let mut probabilities = bases;
    let focus_ratio = bases[Pool::Focus as usize]
        / (bases[Pool::Focus as usize] + bases[Pool::Fivestar as usize]);
    probabilities[Pool::Focus as usize] += pity_pct * focus_ratio;
    probabilities[Pool::Fivestar as usize] += pity_pct * (1.0 - focus_ratio);

    let lower_ratio = bases[Pool::Fourstar as usize]
        / (bases[Pool::Fourstar as usize] + bases[Pool::Threestar as usize]);
    probabilities[Pool::Fourstar as usize] -= pity_pct * lower_ratio;
    probabilities[Pool::Threestar as usize] -= pity_pct * (1.0 - lower_ratio);
    probabilities
}

fn bases() -> [f32; 4] {
    let (focus, fivestar) = *STARTING_RATES.lock().unwrap();
    let focus = focus as f32;
    let fivestar = fivestar as f32;
    let fivestar_total = focus + fivestar;
    let fourstar = (100.0 - fivestar_total) * 58.0 / 94.0;
    let threestar = (100.0 - fivestar_total) * 36.0 / 94.0;
    [focus, fivestar, fourstar, threestar]
}
