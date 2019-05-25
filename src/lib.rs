#[macro_use]
extern crate seed;
use seed::prelude::*;

use std::convert::TryFrom;

mod banner;
use banner::{Banner, BannerChangeKind};

mod goal;
use goal::{Goal, GoalChangeKind};

mod results;

mod sim;
use sim::Sim;

mod weighted_choice;

mod stats;

mod counter;
use counter::Counter;

// Model

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Color {
    Red,
    Blue,
    Green,
    Colorless,
}

impl TryFrom<u8> for Color {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use Color::*;
        Ok(match value {
            0 => Red,
            1 => Blue,
            2 => Green,
            3 => Colorless,
            _ => return Err(()),
        })
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Pool {
    Focus,
    Fivestar,
    Fourstar,
    Threestar,
}

impl TryFrom<u8> for Pool {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use Pool::*;
        Ok(match value {
            0 => Focus,
            1 => Fivestar,
            2 => Fourstar,
            3 => Threestar,
            _ => return Err(()),
        })
    }
}

#[derive(Default, Debug)]
struct Model {
    pub data: Counter,
    pub banner: Banner,
    pub goal: Goal,
}

// Update

#[derive(Clone, Debug)]
pub enum Msg {
    Run,
    BannerChange {
        text: String,
        kind: BannerChangeKind,
    },
    GoalChange {
        text: String,
        kind: GoalChangeKind,
    },
}

fn update(msg: Msg, model: &mut Model, _: &mut Orders<Msg>) {
    log!(msg);
    match msg {
        Msg::BannerChange {
            text,
            kind: BannerChangeKind::FocusSize(idx),
        } => {
            if let Ok(num) = text.parse::<u8>() {
                model.banner.focus_sizes[idx] = num;
                model.data.clear();
            }
        }
        Msg::BannerChange {
            text,
            kind: BannerChangeKind::StartingRates,
        } => {
            let mut numbers = text.split_whitespace();
            let mut nextnum = || -> Option<u8> { Some(numbers.next()?.parse::<u8>().ok()?) };
            if let (Some(first), Some(second)) = (nextnum(), nextnum()) {
                model.banner.starting_rates.0 = first;
                model.banner.starting_rates.1 = second;
                model.data.clear();
                if (first, second) == (8, 0) {
                    // Special handling for legendary banners, since they
                    // always have the same focus pool sizes.
                    model.banner.focus_sizes = [3, 3, 3, 3];
                }
            }
        }
        Msg::Run => {
            let mut sim = Sim::new(model.banner, model.goal.clone());
            let mut limit = 100;
            let perf = seed::window().performance().unwrap();
            let start = perf.now();
            while perf.now() - start < 500.0 {
                for _ in 0..limit {
                    let result = sim.roll_until_goal();
                    model.data[result] += 1;
                }
                limit *= 2;
            }

            log!(format!(
                "{} ({})",
                stats::percentile(&model.data, 0.5),
                model.data.iter().sum::<u32>()
            ));
        }
        Msg::GoalChange {
            text,
            kind: GoalChangeKind::Color,
        } => {
            let value = text
                .parse::<u8>()
                .ok()
                .and_then(|id| Color::try_from(id).ok());
            if let Some(color) = value {
                if model.banner.focus_sizes[color as usize] > 0 {
                    model.goal.goals[0] = goal::GoalPart {
                        unit_color: color,
                        num_copies: 1,
                    };
                }
                model.data.clear();
            }
        }
        Msg::GoalChange {
            text,
            kind: GoalChangeKind::Quantity,
        } => {
            let value = text.parse::<u8>();
            if let Ok(quantity) = value {
                model.goal.goals[0].num_copies = quantity;
                model.data.clear();
            }
        }
    }
    //log!(model);
}

// View

fn view(model: &Model) -> Vec<El<Msg>> {
    vec![
        header![
            a![
                "Help",
                attrs! [
                    At::Href => "help.html";
                ],
            ],
            " | v0.0.3 ",
            a![
                "Changelog",
                attrs![
                    At::Href => "changelog.html";
                ],
            ],
        ],
        div![
            id!["content"],
            goal::goal_selector(&model.goal, &model.banner),
            banner::banner_selector(&model.banner),
            div![simple_ev(Ev::Click, Msg::Run), button!["Run"]],
            results::results(&model.data),
        ],
    ]
}

#[wasm_bindgen]
pub fn render() {
    seed::App::build(Model::default(), update, view)
        .finish()
        .run();
}
