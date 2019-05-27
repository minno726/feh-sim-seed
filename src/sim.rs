use crate::*;

use rand::distributions::Distribution;
use rand::FromEntropy;
use rand::Rng;

use weighted_choice::WeightedIndex4;

use goal::GoalKind;

struct SessionResult {
    chosen_count: u32,
    reset: bool,
}

#[derive(Debug)]
pub struct Sim {
    banner: Banner,
    goal: Goal,
    tables: RandTables,
    rng: rand::rngs::SmallRng,
    goal_data: GoalData,
}

#[derive(Debug, Copy, Clone, Default)]
struct RandTables {
    pool_sizes: [[u8; 4]; 4],
    pool_dists: [WeightedIndex4; 26],
    color_dists: [WeightedIndex4; 4],
}

// Scratch space for representing the goal in a way that is faster to work with
#[derive(Debug, Clone)]
struct GoalData {
    pub color_needed: [bool; 4],
    pub copies_needed: [Vec<u8>; 4],
}

impl GoalData {
    fn is_met(&self) -> bool {
        self.color_needed == [false, false, false, false]
    }
}

impl Sim {
    pub fn new(banner: Banner, goal: Goal) -> Self {
        let mut sim = Sim {
            banner,
            goal,
            tables: RandTables::default(),
            rng: rand::rngs::SmallRng::from_entropy(),
            goal_data: GoalData {
                color_needed: [false; 4],
                copies_needed: [vec![], vec![], vec![], vec![]],
            },
        };
        sim.init_probability_tables();
        sim
    }

    fn init_probability_tables(&mut self) {
        self.tables.pool_sizes = [
            [0, 0, 0, 0],
            [41, 28, 21, 17],
            [32, 29, 20, 28],
            [28, 25, 18, 25],
        ];
        self.tables.pool_sizes[0] = self.banner.focus_sizes;

        for color in 0..4 {
            self.tables.color_dists[color] = WeightedIndex4::new(self.tables.pool_sizes[color]);
        }

        for pity_incr in 0..26 {
            self.tables.pool_dists[pity_incr] =
                WeightedIndex4::new(self.probabilities(pity_incr as u32));
        }
    }

    fn init_goal_data(&mut self) {
        self.goal_data.color_needed = [false, false, false, false];
        for i in 0..4 {
            self.goal_data.copies_needed[i].clear();
        }
        for &goal in &self.goal.goals {
            self.goal_data.copies_needed[goal.unit_color as usize].push(goal.num_copies);
            self.goal_data.color_needed[goal.unit_color as usize] = true;
        }
    }

    // Simulates until reaching the specified goal, then returns # of orbs used
    pub fn roll_until_goal(&mut self) -> u32 {
        let mut pity_count = 0;
        let mut orb_count = 0;
        self.init_goal_data();
        loop {
            let pity_incr = pity_count / 5;
            let samples = [
                self.sample(pity_incr),
                self.sample(pity_incr),
                self.sample(pity_incr),
                self.sample(pity_incr),
                self.sample(pity_incr),
            ];
            let SessionResult {
                chosen_count,
                reset,
            } = self.session_select(&samples);
            if reset {
                pity_count = 0;
            } else {
                pity_count += chosen_count;
            }
            orb_count += Sim::orb_cost(chosen_count);
            if self.goal_data.is_met() {
                return orb_count;
            }
        }
    }

    // Returns: number of items picked, number of items meeting the goal,
    // and whether the rate reset
    fn session_select(&mut self, samples: &[(Pool, Color); 5]) -> SessionResult {
        let mut result = SessionResult {
            chosen_count: 0,
            reset: false,
        };
        for i in 0..5 {
            let sample = samples[i];
            if self.may_match_goal(sample) {
                result.chosen_count += 1;
                result.reset |= self.pull_orb(sample);
                if self.goal_data.is_met() {
                    return result;
                }
            }
        }
        if result.chosen_count == 0 {
            // None with the color we want, so pick randomly
            let sample = samples[self.rng.gen::<usize>() % samples.len()];
            result.reset |= self.pull_orb(sample);
            result.chosen_count = 1;
        }
        result
    }

    fn may_match_goal(&self, sample: (Pool, Color)) -> bool {
        self.goal_data.color_needed[sample.1 as usize]
    }

    // Returns true if the rate should reset
    fn pull_orb(&mut self, sample: (Pool, Color)) -> bool {
        let color = sample.1;
        let do_reset = sample.0 == Pool::Focus || sample.0 == Pool::Fivestar;
        if sample.0 != Pool::Focus || !self.goal_data.color_needed[color as usize] {
            return do_reset;
        }
        let focus_count = self.banner.focus_sizes[color as usize];
        let which_unit = self.rng.gen::<usize>() % focus_count as usize;
        if which_unit < self.goal_data.copies_needed[color as usize].len() {
            if self.goal.kind == GoalKind::Any {
                self.goal_data.color_needed = [false, false, false, false];
            } else {
                if self.goal_data.copies_needed[color as usize][which_unit] > 1 {
                    self.goal_data.copies_needed[color as usize][which_unit] -= 1;
                } else {
                    self.goal_data.copies_needed[color as usize].remove(which_unit);
                    if self.goal_data.copies_needed[color as usize].len() == 0 {
                        self.goal_data.color_needed[color as usize] = false;
                    }
                }
            }
        }
        return do_reset;
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

    fn sample(&mut self, pity_incr: u32) -> (Pool, Color) {
        let pool = self.tables.pool_dists[pity_incr as usize].sample(&mut self.rng) as u8;
        let color = self.tables.color_dists[pool as usize].sample(&mut self.rng) as u8;
        (
            Pool::try_from(pool).unwrap(),
            Color::try_from(color).unwrap(),
        )
    }

    // [focus, fivestar, fourstar, threestar]
    fn probabilities(&self, pity_incr: u32) -> [f32; 4] {
        let bases = self.bases();
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

    fn bases(&self) -> [f32; 4] {
        let (focus, fivestar) = self.banner.starting_rates;
        let focus = focus as f32;
        let fivestar = fivestar as f32;
        let fivestar_total = focus + fivestar;
        let fourstar = (100.0 - fivestar_total) * 58.0 / 94.0;
        let threestar = (100.0 - fivestar_total) * 36.0 / 94.0;
        [focus, fivestar, fourstar, threestar]
    }
}
