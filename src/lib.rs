#[macro_use]
extern crate seed;
use seed::prelude::*;

use std::collections::BTreeMap;

use rand::rngs::SmallRng;
use rand::FromEntropy;

mod sim;

fn now() -> f64 {
    web_sys::window().unwrap().performance().unwrap().now()
}

// Model

#[derive(Clone)]
struct Model {
    pub counts: BTreeMap<u32, u32>,
    pub curr_goal: Goal,
    pub banner: sim::Banner,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            counts: BTreeMap::new(),
            curr_goal: Goal::AnyFivestar,
            banner: sim::Banner {
                starting_rates: (3, 3),
                focus_counts: [1, 1, 1, 1],
            },
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Goal {
    AnyFivestar,
    AnyFocus,
    RedFocus,
    BlueFocus,
    GreenFocus,
    ColorlessFocus,
}

impl Goal {
    fn to_str(&self) -> &'static str {
        use crate::Goal::*;
        match *self {
            AnyFivestar => "Any 5*",
            AnyFocus => "Any Focus",
            RedFocus => "Red Focus",
            BlueFocus => "Blue Focus",
            GreenFocus => "Green Focus",
            ColorlessFocus => "Colorless Focus",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        use crate::Goal::*;
        match s {
            "Any 5*" => Some(AnyFivestar),
            "Any Focus" => Some(AnyFocus),
            "Red Focus" => Some(RedFocus),
            "Blue Focus" => Some(BlueFocus),
            "Green Focus" => Some(GreenFocus),
            "Colorless Focus" => Some(ColorlessFocus),
            _ => None,
        }
    }

    fn color(&self) -> Option<Color> {
        use crate::Color::*;
        use crate::Goal::*;
        match *self {
            RedFocus => Some(Red),
            BlueFocus => Some(Blue),
            GreenFocus => Some(Green),
            ColorlessFocus => Some(Colorless),
            _ => None,
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq)]
#[derive(num_enum::IntoPrimitive, num_enum::CustomTryInto)]
enum Pool {
    Focus,
    Fivestar,
    Fourstar,
    Threestar,
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq)]
#[derive(num_enum::IntoPrimitive, num_enum::CustomTryInto)]
enum Color {
    Red,
    Blue,
    Green,
    Colorless,
}

// Update

#[derive(Clone)]
enum Msg {
    Roll,
    ChangeGoal(String),
    ChangeBanner(String),
}

fn update(msg: Msg, mut model: Model) -> Update<Model> {
    match msg {
        Msg::Roll => {
            let start = now();
            let mut rng = SmallRng::from_entropy();
            for step in 10..16 {
                if now() - start > 500.0 {
                    break;
                }
                for _ in 0..(1 << step) {
                    let result = sim::roll_until(model.curr_goal, model.banner, &mut rng);
                    *model.counts.entry(result as u32).or_insert(0) += 1;
                }
            }
        }
        Msg::ChangeGoal(input) => {
            if input != "" {
                if let Some(new_goal) = Goal::from_str(&input) {
                    model.curr_goal = new_goal;
                    model.counts.clear();
                }
            }
        },
        Msg::ChangeBanner(mut input) => {
            // Hack to get around Nom's issues with incomplete data
            // https://github.com/Geal/nom/issues/839
            input.push('/');
            if let Some(banner) = sim::Banner::parse(&input) {
                model.banner = banner;
                model.counts.clear();
            }
        }
    }
    Render(model)
}

// View

fn goal_menu(model: &Model) -> El<Msg> {
    let variants = [
        Goal::AnyFivestar,
        Goal::AnyFocus,
        Goal::RedFocus,
        Goal::BlueFocus,
        Goal::GreenFocus,
        Goal::ColorlessFocus,
    ];
    let mut options = vec![];
    for &variant in &variants {
        // Prevent color-sniping options from showing up when that color
        // isn't present on the current banner.
        if let Some(color) = variant.color() {
            if model.banner.focus_counts[color as usize] == 0 {
                continue;
            }
        }

        if model.curr_goal == variant {
            options.push(option![variant.to_str(), attrs! { At::Selected => true }]);
        } else {
            options.push(option![variant.to_str()]);
        }
    }
    select![input_ev(Ev::Input, Msg::ChangeGoal), options,]
}

fn results(model: &Model) -> El<Msg> {
    let mut entries = Vec::new();
    let percentiles = [0.25, 0.5, 0.75, 0.9, 0.99];
    let total: u32 = model.counts.values().sum();
    for &pct in &percentiles {
        let mut accum_total = 0;
        for (&key, &value) in &model.counts {
            accum_total += value;
            if accum_total as f64 / total as f64 > pct {
                entries.push(li![format!("{}: {}", pct, key)]);
                break;
            }
        }
    }
    ul![entries]
}

fn banner_selector(model: &Model) -> El<Msg> {
    div![
        label!["Banner: "],
        input![
            input_ev(Ev::Input, Msg::ChangeBanner),
            attrs! {
                At::Value => format!("{}", model.banner);
                At::Title => "Format: r/b/g/c for number of focus units, then (f, r) for starting focus and 5* rates.\nE.g. hero fest is 1/1/1/1 (5, 3), CYL is 1/1/1/1 (3, 3), and legendary is 3/3/3/3 (8, 0)."
            }
        ]
    ]
}

fn view(_state: seed::App<Msg, Model>, model: &Model) -> El<Msg> {
    div![
        div![
            button![
                simple_ev(Ev::Click, Msg::Roll),
                if model.counts.is_empty() {
                    "Roll"
                } else {
                    "More"
                },
            ],
            goal_menu(model)
        ],
        banner_selector(model),
        results(model),
    ]
}

#[wasm_bindgen]
pub fn render() {
    seed::App::build(Model::default(), update, view)
        .finish()
        .run();
}
