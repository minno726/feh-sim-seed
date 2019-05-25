use crate::*;

use rand::distributions::Distribution;
use rand::FromEntropy;
use rand::Rng;

use weighted_choice::WeightedIndex4;

struct SessionResult {
    chosen_count: u32,
    goal_count: u8,
    reset: bool,
}

pub struct Sim {
    banner: Banner,
    goal: Goal,
    tables: RandTables,
    rng: rand::rngs::SmallRng,
}

#[derive(Debug, Copy, Clone, Default)]
struct RandTables {
    pool_sizes: [[u8; 4]; 4],
    pool_dists: [WeightedIndex4; 26],
    color_dists: [WeightedIndex4; 4],
}

impl Sim {
    pub fn new(banner: Banner, goal: Goal) -> Self {
        let mut sim = Sim {
            banner,
            goal,
            tables: RandTables::default(),
            rng: rand::rngs::SmallRng::from_entropy(),
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

    // Simulates until reaching the specified goal, then returns # of orbs used
    pub fn roll_until_goal(&mut self) -> u32 {
        let mut pity_count = 0;
        let mut orb_count = 0;
        let mut curr_goal_count = 0;
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
                goal_count,
                reset,
            } = self.session_select(&samples);
            curr_goal_count += goal_count;
            if reset {
                pity_count = 0;
            } else {
                pity_count += chosen_count;
            }
            orb_count += Sim::orb_cost(chosen_count);
            if curr_goal_count >= self.goal.goals[0].num_copies {
                return orb_count;
            }
        }
    }

    // Returns: number of items picked, number of items meeting the goal,
    // and whether the rate reset
    fn session_select(&mut self, samples: &[(Pool, Color); 5]) -> SessionResult {
        let mut chosen_count = 0;
        let mut goal_count = 0;
        let mut reset = false;
        for i in 0..5 {
            let sample = samples[i];
            if self.may_match_goal(sample) {
                chosen_count += 1;
                if self.does_match_goal(sample) {
                    goal_count += 1;
                    if goal_count >= self.goal.goals[0].num_copies {
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
            let sample = samples[self.rng.gen::<usize>() % samples.len()];
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

    fn may_match_goal(&self, sample: (Pool, Color)) -> bool {
        self.goal.has_color(sample.1)
    }

    fn does_match_goal(&mut self, sample: (Pool, Color)) -> bool {
        let color = self.goal.goals[0].unit_color;
        if sample.0 != Pool::Focus || sample.1 != color {
            return false;
        }
        let focus_count = self.banner.focus_sizes[color as usize];
        if focus_count == 1 {
            true
        } else {
            // Equal chance among all same-color focus units
            return self.rng.gen::<f32>() < 1.0 / focus_count as f32;
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
