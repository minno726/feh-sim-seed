use seed::prelude::*;

use std::convert::TryFrom;
use std::fmt;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{banner::Banner, Color, Msg};

#[derive(Copy, Clone, Debug, EnumIter, PartialEq, Eq)]
pub enum GoalPreset {
    Custom,
    AnyFocus,
    AllFocus,
    RedFocus,
    AnyRed,
    BlueFocus,
    AnyBlue,
    GreenFocus,
    AnyGreen,
    ColorlessFocus,
    AnyColorless,
}

impl fmt::Display for GoalPreset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use crate::goal::GoalPreset::*;
        let s = match *self {
            Custom => "Custom goal",
            AnyFocus => "Any focus unit",
            AllFocus => "All focus units",
            RedFocus => "Specific red focus unit",
            AnyRed => "Any red focus unit",
            BlueFocus => "Specific blue focus unit",
            AnyBlue => "Any blue focus unit",
            GreenFocus => "Specific green focus unit",
            AnyGreen => "Any green focus unit",
            ColorlessFocus => "Specific colorless focus unit",
            AnyColorless => "Any colorless focus unit",
        };
        f.write_str(s)
    }
}

impl TryFrom<u8> for GoalPreset {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        for variant in GoalPreset::iter() {
            if variant as usize == value as usize {
                return Ok(variant);
            }
        }
        Err(())
    }
}

impl GoalPreset {
    pub fn is_available(self, banner: &Banner) -> bool {
        use crate::goal::GoalPreset::*;
        match self {
            Custom => false /* Until custom goals are implemented */,
            AnyFocus | AllFocus => banner.focus_sizes.iter().any(|&x| x > 0),
            RedFocus | AnyRed => banner.focus_sizes[0] > 0,
            BlueFocus | AnyBlue => banner.focus_sizes[1] > 0,
            GreenFocus | AnyGreen => banner.focus_sizes[2] > 0,
            ColorlessFocus | AnyColorless => banner.focus_sizes[3] > 0,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GoalKind {
    Any,
    All,
}

#[derive(Copy, Clone, Debug)]
pub struct GoalPart {
    pub unit_color: Color,
    pub num_copies: u8,
}

#[derive(Clone, Debug)]
pub struct Goal {
    pub kind: GoalKind,
    pub goals: Vec<GoalPart>,
    pub preset: GoalPreset,
}

impl Default for Goal {
    fn default() -> Self {
        Goal {
            kind: GoalKind::All,
            goals: vec![GoalPart {
                unit_color: Color::Red,
                num_copies: 1,
            }],
            preset: GoalPreset::AnyFocus,
        }
    }
}

impl Goal {
    pub fn set_preset(&mut self, banner: &Banner, preset: GoalPreset) {
        use crate::goal::GoalKind::*;
        use crate::goal::GoalPreset::*;
        use crate::Color::*;

        self.preset = preset;
        self.kind = match preset {
            Custom => return,
            AllFocus => All,
            // Every other preset is either Any* or has only one target
            _ => Any,
        };
        self.goals.clear();
        let mut add_color_goal = |color: Color| {
            self.goals.push(GoalPart {
                unit_color: color,
                num_copies: 1,
            });
        };
        // Add an individual GoalPart for each focus unit that matches the
        // conditions of the overall goal.
        match preset {
            Custom => unreachable!(),
            AllFocus | AnyFocus => {
                for idx in 0..banner.focus_sizes.len() {
                    for _ in 0..banner.focus_sizes[idx] {
                        add_color_goal(Color::try_from(idx as u8).unwrap());
                    }
                }
            }
            RedFocus => add_color_goal(Red),
            BlueFocus => add_color_goal(Blue),
            GreenFocus => add_color_goal(Green),
            ColorlessFocus => add_color_goal(Colorless),
            AnyRed => {
                for _ in 0..banner.focus_sizes[0] {
                    add_color_goal(Red)
                }
            }
            AnyBlue => {
                for _ in 0..banner.focus_sizes[1] {
                    add_color_goal(Blue)
                }
            }
            AnyGreen => {
                for _ in 0..banner.focus_sizes[2] {
                    add_color_goal(Green)
                }
            }
            AnyColorless => {
                for _ in 0..banner.focus_sizes[3] {
                    add_color_goal(Colorless)
                }
            }
        }
    }

    // Checks whether or not the goal is possible on the given banner.
    pub fn is_available(&self, banner: &Banner) -> bool {
        match self.preset {
            GoalPreset::Custom => self
                .goals
                .iter()
                .any(|&GoalPart { unit_color, .. }| banner.focus_sizes[unit_color as usize] > 0),
            _ => self.preset.is_available(banner),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum GoalChangeKind {
    Preset,
    Quantity,
}

pub fn goal_selector(goal: &Goal, banner: &Banner) -> El<Msg> {
    let mut select = select![
        id!["goal"],
        input_ev("input", |text| Msg::GoalChange {
            text,
            kind: GoalChangeKind::Preset
        }),
    ];
    for preset in GoalPreset::iter() {
        let mut attrs = attrs! [
            At::Value => preset as usize;
        ];
        if !preset.is_available(banner) {
            attrs.add(At::Disabled, "");
        } else if preset == goal.preset {
            attrs.add(At::Selected, "");
        }
        select.add_child(option![attrs, preset.to_string(),])
    }
    div![
        id!["goal_selector"],
        select,
        label![
            attrs![
                At::For => "goal_count";
            ],
            "Count: ",
        ],
        input![
            id!["goal_count"],
            input_ev("input", |text| Msg::GoalChange {
                text,
                kind: GoalChangeKind::Quantity
            }),
            attrs! [
                At::Type => "number";
                At::Value => goal.goals[0].num_copies; // FIXME: only affects first goalpart
                At::Class => "small_number";
                At::Min => 1;
                At::Required => true;
            ],
        ],
    ]
}
