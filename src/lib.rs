#[macro_use]
extern crate seed;
use seed::prelude::*;

use std::convert::TryFrom;
use std::fmt;

use strum_macros::EnumIter;

use serde::{Deserialize, Serialize};

mod banner;
use banner::Banner;

mod goal;
use goal::{Goal, GoalKind, GoalPart, GoalPreset};

mod results;

mod sim;
use sim::Sim;

mod weighted_choice;

mod stats;

mod counter;
use counter::Counter;

mod subpages;

mod query_string;

// Model

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug, EnumIter, Serialize, Deserialize)]
pub enum Color {
    Red,
    Blue,
    Green,
    Colorless,
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
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
    pub graph_highlight: Option<f32>,
}

// Update

#[derive(Clone, Debug)]
pub enum Msg {
    Null,
    Multiple(Vec<Msg>),
    Run,
    BannerFocusSizeChange { color: Color, quantity: u8 },
    BannerRateChange { rates: (u8, u8) },
    BannerFocusTypeToggle,
    BannerSet { banner: Banner },
    GoalPresetChange { preset: GoalPreset },
    GoalPartColorChange { index: usize, color: Color },
    GoalPartQuantityChange { index: usize, quantity: u8 },
    GoalPartAdd { color: Color, quantity: u8 },
    GoalKindChange { kind: GoalKind },
    GoalSet { goal: Goal },
    PageChange(Page),
    Permalink,
    GraphHighlight { frac: f32 },
}

fn update(msg: Msg, model: &mut Model, orders: &mut Orders<Msg>) {
    log!(msg);
    match msg {
        Msg::Null => {
            orders.skip();
        }
        Msg::Multiple(messages) => {
            orders.skip();
            for msg in messages {
                orders.send_msg(msg);
            }
        }
        Msg::BannerFocusSizeChange { color, quantity } => {
            model.banner.focus_sizes[color as usize] = quantity;
            // The preset's internal representation is dependent on the current
            // banner's focus sizes, so this will keep them in sync properly.

            if model.goal.preset != GoalPreset::Custom {
                model.goal.set_preset(&model.banner, model.goal.preset);
            }
            model.data.clear();
        }
        Msg::BannerRateChange { rates } => {
            model.banner.starting_rates = rates;
            model.data.clear();
            if rates == (8, 0) {
                // Convenient handling for legendary banners, since they
                // always have the same focus pool sizes.
                model.banner.focus_sizes = [3, 3, 3, 3];
            }
        }
        Msg::BannerFocusTypeToggle => {
            model.banner.new_units = !model.banner.new_units;
            model.data.clear();
        }
        Msg::BannerSet { banner } => {
            model.banner = banner;
            model.data.clear();
        }
        Msg::Run => {
            if !model.goal.is_available(&model.banner) {
                return;
            }
            let mut sim = Sim::new(model.banner, model.goal.clone());
            let mut limit = 100;
            let perf = seed::window().performance().unwrap();
            let start = perf.now();

            // Exponential increase with a loose target of 500 ms of calculation.
            // Time per simulation varies wildly depending on device performance
            // and sim parameters, so it starts with a very low number and goes
            // from there.
            while perf.now() - start < 250.0 {
                for _ in 0..limit {
                    let result = sim.roll_until_goal();
                    model.data[result] += 1;
                }
                limit *= 2;
            }

            model.graph_highlight = None;
        }
        Msg::GoalPresetChange { preset } => {
            if preset.is_available(&model.banner) {
                model.goal.set_preset(&model.banner, preset);
            }
            model.data.clear();
        }
        Msg::GoalPartColorChange { index, color } => {
            model.goal.goals[index].unit_color = color;
            model.data.clear();
        }
        Msg::GoalPartQuantityChange { index, quantity } => {
            if quantity == 0 {
                model.goal.goals.remove(index);
            } else {
                model.goal.goals[index].num_copies = quantity;
            }
            model.data.clear();
        }
        Msg::GoalPartAdd { color, quantity } => {
            model.goal.goals.push(GoalPart {
                unit_color: color,
                num_copies: quantity,
            });
            model.data.clear();
        }
        Msg::GoalKindChange { kind } => {
            model.goal.kind = kind;
            model.data.clear();
        }
        Msg::GoalSet { goal } => {
            model.goal = goal;
            model.data.clear();
        }
        Msg::PageChange(page) => {
            model.curr_page = page;
        }
        Msg::Permalink => {
            let url = seed::Url::new(vec!["fehstatsim"]).search(&format!(
                "banner={}&goal={}&run=1",
                base64::encode(&bincode::serialize(&model.banner).unwrap()),
                base64::encode(&bincode::serialize(&model.goal).unwrap())
            ));
            seed::push_route(url);
        }
        Msg::GraphHighlight { frac } => {
            model.graph_highlight = Some(frac);
        }
    }
}

// View

fn view(model: &Model) -> Vec<El<Msg>> {
    match model.curr_page {
        Page::Main => main_page(model),
        Page::Help => subpages::help(),
        Page::Changelog => subpages::changelog(),
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
            " | v0.1.0 ",
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
            button!["Permalink", simple_ev(Ev::Click, Msg::Permalink),],
            button![
                simple_ev(Ev::Click, Msg::Run),
                if !model.goal.is_available(&model.banner) {
                    attrs![At::Disabled => true]
                } else {
                    attrs![]
                },
                if model.data.is_empty() { "Run" } else { "More" }
            ],
            results::results(&model.data, model.graph_highlight),
        ],
    ]
}

fn routes(url: &seed::Url) -> Msg {
    let mut messages = vec![];

    messages.push(match url.path.get(1).map(String::as_str) {
        Some("help") => Msg::PageChange(Page::Help),
        Some("changelog") => Msg::PageChange(Page::Changelog),
        _ => Msg::PageChange(Page::Main),
    });

    if let Some(banner) = query_string::get(url, "banner").and_then(Banner::from_query_string) {
        messages.push(Msg::BannerSet { banner });
    }

    if let Some(goal) = query_string::get(url, "goal").and_then(Goal::from_query_string) {
        messages.push(Msg::GoalSet { goal });
    }

    if let Some("1") = query_string::get(url, "run") {
        messages.push(Msg::Run);
    }

    Msg::Multiple(messages)
}

#[wasm_bindgen]
pub fn render() {
    seed::App::build(Model::default(), update, view)
        .routes(routes)
        .finish()
        .run();
}
