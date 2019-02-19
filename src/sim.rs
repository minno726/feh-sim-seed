//use rand::rngs::SmallRng;
use rand::distributions::{Uniform, WeightedIndex};
use rand::prelude::*;

use crate::*;

mod parser {
    use nom::IResult;
    use nom::{tag, named, ws, map, do_parse, map_res};
    use crate::sim::Banner;

    fn number<T: std::str::FromStr>(i: &str) -> IResult<&str, T> {
        return map_res! {
            i,
            nom::digit,
            |s: &str| str::parse::<T>(s)
        }
    }

    named!{focus<&str, (u8, u8, u8, u8)>,
        do_parse!(
            r: number >>
            tag!("/") >>
            b: number >>
            tag!("/") >>
            g: number >>
            tag!("/") >>
            c: number >>
            ((r, b, g, c))
        )
    }

    named!{rates<&str, (u8, u8)>,
        ws!(
            delimited!(
                tag!("("),
                separated_pair!(
                    number,
                    tag!(","),
                    number
                ),
                tag!(")")
            )
        )
    }

    named!{banner(&str) -> Banner,
        map!(
            ws!(
                tuple! (
                    focus,
                    rates
                )
            ),
            |(focus, rates)| Banner {
                starting_rates: rates,
                focus_counts: [focus.0, focus.1, focus.2, focus.3]
            }
        )
    }

    impl Banner {
        pub fn parse(input: &str) -> Option<Banner> {
            banner(input).ok().map(|(_, result)| result).and_then(|result| {
                // Banner guarantees a 5* after 120 rolls, which adds 12 percentage
                // points total. If rates start out at >88% total, that stops
                // making sense.
                if result.starting_rates.0 + result.starting_rates.1 > 88 {
                    None
                } else {
                    Some(result)
                }
            })
        }
    }
}

// Outer layer: focus/5*/4*/3*
// Inner layer: r/b/g/c
const POOL_SIZES: [[u8; 4]; 4] = [
    [0, 0, 0, 0],
    [41, 28, 21, 17],
    [32, 29, 20, 28],
    [28, 25, 18, 25],
];

#[derive(Copy, Clone, Debug)]
pub struct Banner {
    pub starting_rates: (u8, u8),
    pub focus_counts: [u8; 4],
}

impl std::fmt::Display for Banner {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let [r, b, g, c] = self.focus_counts;
        let (focus, regular) = self.starting_rates;
        write!(f, "{}/{}/{}/{} ({}, {})", r, b, g, c, focus, regular)
    }
}

// Simulates until reaching the specified goal, then returns # of orbs used
pub fn roll_until(goal: Goal, banner: Banner, rng: &mut SmallRng) -> u16 {
    let mut pool_sizes = POOL_SIZES;
    pool_sizes[0] = banner.focus_counts;
    let mut pity_count = 0;
    let mut orb_count = 0;
    let color_dists = [
        WeightedIndex::new(&pool_sizes[0]).unwrap(),
        WeightedIndex::new(&pool_sizes[1]).unwrap(),
        WeightedIndex::new(&pool_sizes[2]).unwrap(),
        WeightedIndex::new(&pool_sizes[3]).unwrap(),
    ];
    loop {
        let weights = probabilities(banner, pity_count);
        let pool_dist = WeightedIndex::new(&weights).unwrap();

        let mut sampler = || sample(&pool_dist, &color_dists, rng);
        let samples = [sampler(), sampler(), sampler(), sampler(), sampler()];
        let (num_rolls, num_matches, do_reset) = session_select(goal, banner, &samples);
        if do_reset {
            pity_count = 0;
        } else {
            pity_count += num_rolls;
        }
        orb_count += orb_cost(num_rolls);
        if num_matches > 0 {
            return orb_count;
        }
    }
}

// Returns: number of items picked, number of items meeting the goal,
// and whether the rate reset
fn session_select(goal: Goal, banner: Banner, samples: &[(Pool, Color); 5]) -> (u16, u16, bool) {
    let mut num_picked = 0;
    let mut num_goal = 0;
    let mut reset = false;
    for i in 0..5 {
        let sample = samples[i];
        if may_match_goal(goal, banner, sample) {
            num_picked += 1;
            if does_match_goal(goal, banner, sample) {
                num_goal += 1;
                return (num_picked, num_goal, true);
            }
            if sample.0 as usize == Pool::Focus as usize || sample.0 as usize == Pool::Fivestar as usize {
                reset = true;
            }
        }
    }
    if num_picked == 0 {
        // None with the color we want, so pick randomly
        let i = Uniform::new(0, 5).sample(&mut rand::thread_rng());
        let sample = samples[i];
        if sample.0 as usize == Pool::Focus as usize || sample.0 as usize == Pool::Fivestar as usize {
            reset = true;
        }
        num_picked = 1;
    }
    (num_picked, num_goal, reset)
}

fn may_match_goal(goal: Goal, banner: Banner, sample: (Pool, Color)) -> bool {
    use Goal::*;
    match goal {
        AnyFivestar | AnyFocus => banner.focus_counts[sample.1 as usize] > 0,
        RedFocus => sample.1 as usize == Color::Red as usize,
        BlueFocus => sample.1 as usize == Color::Blue as usize,
        GreenFocus => sample.1 as usize == Color::Green as usize,
        ColorlessFocus => sample.1 as usize == Color::Colorless as usize,
    }
}

fn does_match_goal(goal: Goal, banner: Banner, sample: (Pool, Color)) -> bool {
    use Goal::*;
    if sample.0 as usize == Pool::Fourstar as usize || sample.0 as usize == Pool::Threestar as usize {
        return false;
    }
    let color = match goal {
        // Can quit early, no need to check banner.
        AnyFivestar => return true,
        AnyFocus => return sample.0 as usize == Pool::Focus as usize,
        // Need to extract the color, to check against banner focus counts
        RedFocus => Color::Red as usize,
        BlueFocus => Color::Blue as usize,
        GreenFocus => Color::Green as usize,
        ColorlessFocus => Color::Colorless as usize,
    };
    if sample.0 as usize != Pool::Focus as usize {
        return false;
    }
    if banner.focus_counts[color] == 1 {
        true
    } else {
        // Equal chance among all same-color focus units
        return Uniform::new(0, banner.focus_counts[color]).sample(&mut rand::thread_rng()) == 0;
    }
}

fn orb_cost(count: u16) -> u16 {
    match count {
        1 => 5,
        2 => 9,
        3 => 13,
        4 => 17,
        5 => 20,
        _ => panic!("Invalid orb cost: {}", count),
    }
}

fn sample(
    pool_dist: &WeightedIndex<f64>,
    color_dists: &[WeightedIndex<u8>; 4],
    rng: &mut SmallRng,
) -> (Pool, Color) {
    let pool = pool_dist.sample(rng) as u8;
    let color = color_dists[pool as usize].sample(rng) as u8;
    (pool.try_into_Pool().unwrap(), color.try_into_Color().unwrap())
}

// [focus, fivestar, fourstar, threestar]
fn probabilities(banner: Banner, roll_count: u16) -> [f64; 4] {
    let bases = bases(banner);
    let pity_pct = if roll_count >= 120 {
        100.0 - bases[Pool::Focus as usize] - bases[1]
    } else {
        (roll_count / 5) as f64 * 0.5
    };
    let mut probabilities = bases;
    probabilities[Pool::Focus as usize] += pity_pct * bases[Pool::Focus as usize] / (bases[Pool::Focus as usize] + bases[Pool::Fivestar as usize]);
    probabilities[Pool::Fivestar as usize] += pity_pct * bases[Pool::Fivestar as usize] / (bases[Pool::Focus as usize] + bases[Pool::Fivestar as usize]);
    let reduction =
        (bases[Pool::Fourstar as usize] + bases[Pool::Threestar as usize] - pity_pct) / (bases[Pool::Fourstar as usize] + bases[Pool::Threestar as usize]);
    probabilities[Pool::Fourstar as usize] *= reduction;
    probabilities[Pool::Threestar as usize] *= reduction;
    probabilities
}

fn bases(banner: Banner) -> [f64; 4] {
    let (focus, fivestar) = banner.starting_rates;
    let focus = focus as f64;
    let fivestar = fivestar as f64;
    let fivestar_total = focus + fivestar;
    let fourstar = (100.0 - fivestar_total) * 58.0 / 94.0;
    let threestar = (100.0 - fivestar_total) * 36.0 / 94.0;
    [focus, fivestar, fourstar, threestar]
}
