#[macro_use]
extern crate seed;
use seed::prelude::*;

use std::convert::TryFrom;

mod banner;
use banner::{Banner, BannerChangeKind};

mod goal;
use goal::{Goal, GoalChangeKind, GoalPreset};

mod results;

mod sim;
use sim::Sim;

mod weighted_choice;

mod stats;

mod counter;
use counter::Counter;

mod subpages;

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

#[derive(Copy, Clone, Debug)]
pub enum Page {
    Main,
    Help,
    Changelog,
}

impl Default for Page {
    fn default() -> Self {
        Page::Main
    }
}

#[derive(Default, Debug)]
struct Model {
    pub data: Counter,
    pub banner: Banner,
    pub goal: Goal,
    pub curr_page: Page,
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
    PageChange(Page),
}

fn update(msg: Msg, model: &mut Model, _: &mut Orders<Msg>) {
    log!(msg);
    match msg {
        Msg::BannerChange {
            text,
            kind: BannerChangeKind::FocusSize(color),
        } => {
            if let Ok(num) = text.parse::<u8>() {
                model.banner.focus_sizes[color as usize] = num;
                model.data.clear();
            }
        }
        Msg::BannerChange {
            text,
            kind: BannerChangeKind::StartingRates,
        } => {
            let mut numbers = text.split_whitespace();
            let mut nextnum = || Some(numbers.next()?.parse::<u8>().ok()?);
            if let (Some(first), Some(second)) = (nextnum(), nextnum()) {
                model.banner.starting_rates.0 = first;
                model.banner.starting_rates.1 = second;
                model.data.clear();
                if (first, second) == (8, 0) {
                    // Convenient handling for legendary banners, since they
                    // always have the same focus pool sizes.
                    model.banner.focus_sizes = [3, 3, 3, 3];
                }
            }
        }
        Msg::Run => {
            // Ensure that the controls are in sync
            model.goal.set_preset(&model.banner, model.goal.preset);
            if !model.goal.is_available(&model.banner) {
                return;
            }
            let mut sim = Sim::new(model.banner, model.goal.clone());
            let mut limit = 100;
            let perf = seed::window().performance().unwrap();
            let start = perf.now();

            // Exponential increase with a loose target of 1000 ms of calculation.
            // Time per simulation varies wildly depending on device performance
            // and sim parameters, so it starts with a very low number and goes
            // from there.
            while perf.now() - start < 500.0 {
                for _ in 0..limit {
                    let result = sim.roll_until_goal();
                    model.data[result] += 1;
                }
                limit *= 2;
            }
        }
        Msg::GoalChange {
            text,
            kind: GoalChangeKind::Preset,
        } => {
            let value = text
                .parse::<u8>()
                .ok()
                .and_then(|id| GoalPreset::try_from(id).ok());
            if let Some(preset) = value {
                if preset.is_available(&model.banner) {
                    model.goal.set_preset(&model.banner, preset);
                }
                model.data.clear();
            }
            log!(model.goal);
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
        Msg::PageChange(page) => {
            model.curr_page = page;
        }
    }
}

// View

fn view(model: &Model) -> Vec<El<Msg>> {
    match model.curr_page {
        Page::Main => main_page(model),
        Page::Help => El::from_markdown(subpages::HELP_MD),
        Page::Changelog => El::from_markdown(subpages::CHANGELOG_MD),
    }
}

fn main_page(model: &Model) -> Vec<El<Msg>> {
    vec![
        header![
            a![
                "Help",
                attrs! [
                    At::Href => "/fehstatsim/help";
                ],
            ],
            " | v0.0.3 ",
            a![
                "Changelog",
                attrs![
                    At::Href => "/fehstatsim/changelog";
                ],
            ],
        ],
        div![
            id!["content"],
            goal::goal_selector(&model.goal, &model.banner),
            banner::banner_selector(&model.banner),
            div![
                simple_ev(Ev::Click, Msg::Run),
                button![
                    if !model.goal.is_available(&model.banner) {
                        attrs![At::Disabled => true]
                    } else {
                        attrs![]
                    },
                    "Run"
                ]
            ],
            results::results(&model.data),
        ],
    ]
}

fn routes(url: &seed::Url) -> Msg {
    if url.path.len() <= 1 {
        return Msg::PageChange(Page::Main);
    }

    log!(url);

    match &*url.path[1] {
        "help" => Msg::PageChange(Page::Help),
        "changelog" => Msg::PageChange(Page::Changelog),
        _ => Msg::PageChange(Page::Main),
    }
}

#[wasm_bindgen]
pub fn render() {
    seed::App::build(Model::default(), update, view)
        .routes(routes)
        .finish()
        .run();
}
