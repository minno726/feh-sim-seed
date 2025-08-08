use crate::*;

use rand::distributions::Distribution;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use weighted_choice::{WeightedIndex4, WeightedIndex5};

use goal::{CustomGoal, GoalKind};

/// The results of a pull session.
struct SessionResult {
    chosen_count: u32,
    got_focus: bool,
    nonfocus_count: u32,
}

struct PullOrbResult {
    got_non_focus: bool,
    got_focus: bool,
}

/// A structure holding the information for a sequence of summoning
/// sessions done until a certain goal is reached. Keeps some cached information
/// in order to make the simulation as fast as possible.
#[derive(Debug)]
pub struct Sim {
    banner: Banner,
    goal: CustomGoal,
    tables: RandTables,
    rng: SmallRng,
    goal_data: GoalData,
}

/// Precalculated tables for the probabilities of units being randomly chosen.
#[derive(Debug, Copy, Clone, Default)]
struct RandTables {
    pool_sizes: [[u8; 4]; 5],
    pool_dists: [WeightedIndex5; 26],
    color_dists: [WeightedIndex4; 5],
}

/// Scratch space for representing the goal in a way that is faster to work with.
#[derive(Debug, Clone)]
struct GoalData {
    pub is_fourstar_focus: bool,
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
            goal: goal.as_custom(&banner),
            tables: RandTables::default(),
            rng: SmallRng::from_entropy(),
            goal_data: GoalData {
                is_fourstar_focus: banner.fourstar_focus.is_some(),
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
            [16, 12, 12, 15],
            [0, 0, 0, 0],
            [50, 53, 40, 54],
            [50, 53, 40, 54],
        ];
        for i in 0..4 {
            self.tables.pool_sizes[0][i] = self.banner.focus_sizes[i].max(0) as u8;
        }
        if let Some(color) = self.banner.fourstar_focus {
            self.tables.pool_sizes[2][color as usize] = 1;
        }

        for color in 0..5 {
            self.tables.color_dists[color] = WeightedIndex4::new(self.tables.pool_sizes[color]);
        }

        for pity_incr in 0..26 {
            self.tables.pool_dists[pity_incr] =
                WeightedIndex5::new(self.probabilities(pity_incr as u32));
        }
    }

    // Initializes the internal representation of a goal.
    fn init_goal_data(&mut self) {
        self.goal_data.color_needed = [false, false, false, false];
        self.goal_data.is_fourstar_focus = false;
        for i in 0..4 {
            self.goal_data.copies_needed[i].clear();
        }
        for &goal in &self.goal.goals {
            self.goal_data.copies_needed[goal.unit_color as usize].push(goal.num_copies);
            self.goal_data.color_needed[goal.unit_color as usize] = true;
            if goal.four_star {
                self.goal_data.is_fourstar_focus = true;
            }
        }
    }

    /// Simulates until reaching the current goal, then returns # of orbs used.
    pub fn roll_until_goal(&mut self) -> u32 {
        let mut pity_count = 0;
        let mut orb_count = 0;
        let mut focus_charges = 0;
        self.init_goal_data();
        loop {
            let pity_incr = pity_count / 5;
            let samples = [
                self.sample(pity_incr, focus_charges == 3),
                self.sample(pity_incr, focus_charges == 3),
                self.sample(pity_incr, focus_charges == 3),
                self.sample(pity_incr, focus_charges == 3),
                self.sample(pity_incr, focus_charges == 3),
            ];
            let SessionResult {
                chosen_count,
                got_focus,
                nonfocus_count,
            } = self.session_select(&samples);
            pity_count += chosen_count;
            if got_focus {
                pity_count = 0;
            } else {
                pity_count = pity_count.saturating_sub(20 * nonfocus_count);
            }
            if got_focus && focus_charges == 3 {
                focus_charges = 0;
            }
            if self.banner.focus_charges {
                focus_charges = (focus_charges + nonfocus_count).min(3);
                if got_focus {
                    focus_charges = 0;
                }
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
            got_focus: false,
            nonfocus_count: 0,
        };
        for i in 0..5 {
            let sample = samples[i];
            if self.may_match_goal(sample.1) || (i == 4 && result.chosen_count == 0) {
                result.chosen_count += 1;
                let pull_result = self.pull_orb(sample);
                result.got_focus |= pull_result.got_focus;
                result.nonfocus_count += if pull_result.got_non_focus { 1 } else { 0 };
                if self.goal_data.is_met() {
                    return result;
                }
            }
        }
        result
    }

    /// Specifies whether the color has the possibility of contributing towards
    /// completing the current goal.
    fn may_match_goal(&self, color: Color) -> bool {
        self.goal_data.color_needed[color as usize]
    }

    /// Evaluates the result of selecting the given sample.
    fn pull_orb(&mut self, sample: (Pool, Color)) -> PullOrbResult {
        let color = sample.1;
        if sample.0 == Pool::Threestar
            || sample.0 == Pool::Fourstar
            || sample.0 == Pool::Fivestar
            || (sample.0 == Pool::FourstarFocus && !self.goal_data.is_fourstar_focus)
            || !self.goal_data.color_needed[color as usize]
        {
            return PullOrbResult {
                got_focus: sample.0 == Pool::Focus,
                got_non_focus: sample.0 == Pool::Fivestar,
            };
        }
        let focus_count = self.banner.focus_sizes[color as usize];
        let which_unit = if sample.0 == Pool::FourstarFocus {
            0
        } else {
            self.rng.gen::<usize>() % focus_count as usize
        };
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
        PullOrbResult {
            got_focus: sample.0 == Pool::Focus,
            got_non_focus: sample.0 == Pool::Fivestar,
        }
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
    fn sample(&mut self, pity_incr: u32, focus_charge_active: bool) -> (Pool, Color) {
        let pool = self.tables.pool_dists[pity_incr as usize].sample(&mut self.rng) as u8;
        let mut pool = Pool::try_from(pool).unwrap();
        if focus_charge_active && pool == Pool::Fivestar {
            pool = Pool::Focus;
        }

        let color = self.tables.color_dists[pool as usize].sample(&mut self.rng) as u8;
        let color = Color::try_from(color).unwrap();
        (pool, color)
    }

    /// Calculates the actual probabilities of selecting a unit from each of the four
    /// possible pools after a certain number of rate increases.
    fn probabilities(&self, pity_incr: u32) -> [f32; 5] {
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
    fn bases(&self) -> [f32; 5] {
        let (focus, fivestar) = self.banner.starting_rates;
        if self.banner.fourstar_focus.is_some() {
            [3.0, 3.0, 3.0, 55.0, 36.0]
        } else if (focus, fivestar) == (6, 0) {
            // The lower-rarity breakdown on this new banner is different
            // for no apparent reason
            [6.0, 0.0, 0.0, 60.0, 34.0]
        } else {
            let focus = focus as f32;
            let fivestar = fivestar as f32;
            let fivestar_total = focus + fivestar;
            let fourstar = (100.0 - fivestar_total) * 58.0 / 94.0;
            let threestar = (100.0 - fivestar_total) * 36.0 / 94.0;
            [focus, fivestar, 0.0, fourstar, threestar]
        }
    }
}
