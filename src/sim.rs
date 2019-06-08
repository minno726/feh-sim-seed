use crate::*;

use rand::distributions::Distribution;
use rand::FromEntropy;
use rand::Rng;

use weighted_choice::WeightedIndex4;

use goal::GoalKind;

/// The results of a pull session.
struct SessionResult {
    chosen_count: u32,
    reset: bool,
}

/// A structure holding the information for a sequence of summoning
/// sessions done until a certain goal is reached. Keeps some cached information
/// in order to make the simulation as fast as possible.
#[derive(Debug)]
pub struct Sim {
    banner: Banner,
    goal: Goal,
    tables: RandTables,
    rng: rand::rngs::SmallRng,
    goal_data: GoalData,
}

/// Precalculated tables for the probabilities of units being randomly chosen.
#[derive(Debug, Copy, Clone, Default)]
struct RandTables {
    pool_sizes: [[u8; 4]; 4],
    pool_dists: [WeightedIndex4; 26],
    color_dists: [WeightedIndex4; 4],
}

/// Scratch space for representing the goal in a way that is faster to work with.
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
    /// Creates a new simulator for the given banner and goal, doing some
    /// moderately expensive initialization. Avoid running in a hot loop, but
    /// it's not a problem to call somewhat frequently.
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

    /// Initializes the precalculated tables used for fast random sampling.
    fn init_probability_tables(&mut self) {
        self.tables.pool_sizes = [
            [0, 0, 0, 0],
            if self.banner.new_units {
                [21, 16, 16, 9]
            } else {
                [41, 28, 24, 19]
            },
            [32, 31, 20, 28],
            [32, 29, 19, 28],
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

    // Initializes the internal representation of a goal.
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

    /// Simulates until reaching the current goal, then returns # of orbs used.
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

    /// Given a session with five randomly-selected units, decides which ones
    /// would be chosen to achieve the current goal, then evaluates the results
    /// of choosing them.
    fn session_select(&mut self, samples: &[(Pool, Color); 5]) -> SessionResult {
        let mut result = SessionResult {
            chosen_count: 0,
            reset: false,
        };
        for i in 0..5 {
            let sample = samples[i];
            if self.may_match_goal(sample.1) {
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

    /// Specifies whether the color has the possibility of contributing towards
    /// completing the current goal.
    fn may_match_goal(&self, color: Color) -> bool {
        self.goal_data.color_needed[color as usize]
    }

    /// Evaluates the result of selecting the given sample. Returns `true` if the
    /// sample made the rate increase reset.
    fn pull_orb(&mut self, sample: (Pool, Color)) -> bool {
        let color = sample.1;
        let do_reset = sample.0 == Pool::Focus || sample.0 == Pool::Fivestar;
        if sample.0 != Pool::Focus || !self.goal_data.color_needed[color as usize] {
            return do_reset;
        }
        let focus_count = self.banner.focus_sizes[color as usize];
        let which_unit = self.rng.gen::<usize>() % focus_count as usize;
        if which_unit < self.goal_data.copies_needed[color as usize].len() {
            if self.goal_data.copies_needed[color as usize][which_unit] > 1 {
                self.goal_data.copies_needed[color as usize][which_unit] -= 1;
            } else {
                self.goal_data.copies_needed[color as usize].remove(which_unit);
                if self.goal.kind == GoalKind::Any {
                    self.goal_data.color_needed = [false, false, false, false];
                } else if self.goal_data.copies_needed[color as usize].len() == 0 {
                    self.goal_data.color_needed[color as usize] = false;
                }
            }
        }
        return do_reset;
    }

    /// The total orb cost of choosing the given number of units from a session.
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

    /// Chooses a weighted random unit from the summoning pool. `pity_incr` is the
    /// number of times that the 5* rates have increased by 0.5% total.
    fn sample(&mut self, pity_incr: u32) -> (Pool, Color) {
        let pool = self.tables.pool_dists[pity_incr as usize].sample(&mut self.rng) as u8;
        let color = self.tables.color_dists[pool as usize].sample(&mut self.rng) as u8;
        (
            Pool::try_from(pool).unwrap(),
            Color::try_from(color).unwrap(),
        )
    }

    /// Calculates the actual probabilities of selecting a unit from each of the four
    /// possible pools after a certain number of rate increases.
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

    /// Gives the base probabilities of selecting a unit from each pool.
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
