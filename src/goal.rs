use seed::prelude::*;

use std::convert::TryFrom;
use std::fmt;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use serde::{Deserialize, Serialize};

use crate::{banner::Banner, Color, Msg};

/// Pre-set options for common goals.
#[derive(Copy, Clone, Debug, EnumIter, PartialEq, Eq, Serialize, Deserialize)]
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
    /// Determines whether or not the selected preset is a goal that it is
    /// possible to achieve on the banner.
    pub fn is_available(self, banner: &Banner) -> bool {
        use crate::goal::GoalPreset::*;
        match self {
            Custom => true,
            AnyFocus | AllFocus => banner.focus_sizes.iter().any(|&x| x > 0),
            RedFocus | AnyRed => banner.focus_sizes[0] > 0,
            BlueFocus | AnyBlue => banner.focus_sizes[1] > 0,
            GreenFocus | AnyGreen => banner.focus_sizes[2] > 0,
            ColorlessFocus | AnyColorless => banner.focus_sizes[3] > 0,
        }
    }

    /// Says whether or not the preset has only a single unit that counts for
    /// completing the goal.
    fn is_single_target(&self) -> bool {
        match self {
            GoalPreset::RedFocus
            | GoalPreset::BlueFocus
            | GoalPreset::GreenFocus
            | GoalPreset::ColorlessFocus => true,
            _ => false,
        }
    }
}

/// Whether the given goal is to achieve all of the goal parts or just a single one.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoalKind {
    Any,
    All,
}

/// A single unit that the goal is trying to obtain.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct GoalPart {
    pub unit_color: Color,
    pub num_copies: u8,
}

/// The goal of a summoning session.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Goal {
    pub kind: GoalKind,
    pub goals: Vec<GoalPart>,
    pub preset: GoalPreset,
}

impl Default for Goal {
    fn default() -> Self {
        let mut goal = Goal {
            kind: GoalKind::Any,
            goals: vec![],
            preset: GoalPreset::AnyFocus,
        };
        goal.set_preset(&Banner::default(), goal.preset);
        goal
    }
}

impl Goal {
    /// Sets the goal to the provided preset, updating the internal structure
    /// as needed.
    pub fn set_preset(&mut self, banner: &Banner, preset: GoalPreset) {
        use crate::goal::GoalKind::*;
        use crate::goal::GoalPreset::*;
        use crate::Color::*;

        let count = if self.preset.is_single_target() && preset.is_single_target() {
            // If we're switching from one preset to another and both have
            // a single target, then preserve the number of copies that the
            // old setting specified, if possible.
            self.goals.get(0).map(|part| part.num_copies).unwrap_or(1)
        } else {
            1
        };

        if self.preset != GoalPreset::Custom && preset == GoalPreset::Custom {
            // If we're switching from a preset to a custom goal, make
            // sure that the subgoals are in sync with that preset, since
            // they're about to be displayed to the user for editing.
            self.set_preset(banner, self.preset);
        }
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
                num_copies: count,
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

    /// Checks whether or not the goal is possible on the given banner.
    pub fn is_available(&self, banner: &Banner) -> bool {
        match self.preset {
            GoalPreset::Custom => self
                .goals
                .iter()
                .any(|&GoalPart { unit_color, .. }| banner.focus_sizes[unit_color as usize] > 0),
            _ => self.preset.is_available(banner),
        }
    }

    /// Parses data from the representation used in query strings to share settings.
    pub fn from_query_string(s: &str) -> Option<Self> {
        let data = base64::decode(s).ok()?;
        bincode::deserialize(&data).ok()
    }

}

/// Section for selecting the goal.
pub fn goal_selector(goal: &Goal, banner: &Banner) -> El<Msg> {
    let mut select = select![
        id!["goal"],
        input_ev("input", |text| {
            if let Some(preset) = text
                .parse::<u8>()
                .ok()
                .and_then(|id| GoalPreset::try_from(id).ok())
            {
                Msg::GoalPresetChange { preset }
            } else {
                Msg::Null
            }
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
        if goal.preset.is_single_target() {
            span![
                label![
                    attrs![
                        At::For => "goal_count";
                    ],
                    "Count: ",
                ],
                input![
                    id!["goal_count"],
                    input_ev("input", |text| {
                        if let Ok(quantity) = text.parse::<u8>() {
                            Msg::GoalPartQuantityChange { index: 0, quantity }
                        } else {
                            Msg::Null
                        }
                    }),
                    attrs! [
                        At::Type => "number";
                        At::Value => goal.goals[0].num_copies; // FIXME: only affects first goalpart
                        At::Class => "small_number";
                        At::Min => 1;
                        At::Required => true;
                    ],
                ]
            ]
        } else {
            seed::empty()
        },
        if goal.preset == GoalPreset::Custom {
            advanced_goal_selector(goal)
        } else {
            seed::empty()
        },
    ]
}

/// Subsection for selecting the goal using the detailed representation instead of
/// a preset.
fn advanced_goal_selector(goal: &Goal) -> El<Msg> {
    let mut base = div![style!["margin-left" => "2em";]];
    if goal.goals.len() > 1 {
        base.add_child(select![
            input_ev(Ev::Input, |text| match &*text {
                "Any" => Msg::GoalKindChange {
                    kind: GoalKind::Any
                },
                "All" => Msg::GoalKindChange {
                    kind: GoalKind::All
                },
                _ => Msg::Null,
            }),
            option![
                attrs![
                    At::Value => "Any";
                ],
                if goal.kind == GoalKind::Any {
                    attrs![At::Selected => ""]
                } else {
                    attrs![]
                },
                "Any of these",
            ],
            option![
                attrs![
                    At::Value => "All";
                ],
                if goal.kind == GoalKind::All {
                    attrs![At::Selected => ""]
                } else {
                    attrs![]
                },
                "All of these",
            ],
        ]);
    }

    for (index, goal_part) in goal.goals.iter().enumerate() {
        let mut color_select = select![input_ev(Ev::Input, move |value| {
            if let Some(color) = value
                .parse::<u8>()
                .ok()
                .and_then(|num| Color::try_from(num).ok())
            {
                Msg::GoalPartColorChange { index, color }
            } else {
                Msg::Null
            }
        }),];
        for color in Color::iter() {
            let mut attrs = attrs![At::Value => color as usize];
            if goal_part.unit_color == color {
                attrs.add(At::Selected, "");
            }
            color_select.add_child(option![attrs, color.to_string()])
        }
        base.add_child(div![
            button![
                simple_ev(
                    Ev::Click,
                    Msg::GoalPartQuantityChange { index, quantity: 0 }
                ),
                "X",
            ],
            input![
                input_ev(Ev::Input, move |value| {
                    if let Ok(quantity) = value.parse::<u8>() {
                        Msg::GoalPartQuantityChange { index, quantity }
                    } else {
                        Msg::Null
                    }
                }),
                attrs![
                    At::Type => "number";
                    At::Class => "small_number";
                    At::Min => 0;
                    At::Required => true;
                    At::Value => goal_part.num_copies;
                ]
            ],
            " copies of a specific ",
            color_select,
            " unit",
        ]);
    }

    base.add_child(button![
        simple_ev(
            Ev::Click,
            Msg::GoalPartAdd {
                color: Color::Red,
                quantity: 1
            }
        ),
        "+",
    ]);

    base
}
