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
    AnyFocus,
    AllFocus,
    RedFocus,
    AnyRed,
    RedFourstarFocus,
    BlueFocus,
    AnyBlue,
    BlueFourstarFocus,
    GreenFocus,
    AnyGreen,
    GreenFourstarFocus,
    ColorlessFocus,
    AnyColorless,
    ColorlessFourstarFocus,
}

impl fmt::Display for GoalPreset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use crate::goal::GoalPreset::*;
        let s = match *self {
            AnyFocus => "Any 5* focus unit",
            AllFocus => "All focus units",
            RedFocus => "Specific red 5* focus unit",
            RedFourstarFocus => "The red 4* focus unit",
            AnyRed => "Any red 5* focus unit",
            BlueFocus => "Specific blue 5* focus unit",
            BlueFourstarFocus => "The blue 4* focus unit",
            AnyBlue => "Any blue 5* focus unit",
            GreenFocus => "Specific green 5* focus unit",
            GreenFourstarFocus => "The green 4* focus unit",
            AnyGreen => "Any green 5* focus unit",
            ColorlessFocus => "Specific colorless 5* focus unit",
            AnyColorless => "Any colorless 5* focus unit",
            ColorlessFourstarFocus => "The colorless 4* focus unit",
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
        use GoalPreset::*;
        match self {
            AnyFocus | AllFocus => banner.focus_sizes.iter().any(|&x| x > 0),
            RedFocus | AnyRed => banner.focus_sizes[0] > 0,
            BlueFocus | AnyBlue => banner.focus_sizes[1] > 0,
            GreenFocus | AnyGreen => banner.focus_sizes[2] > 0,
            ColorlessFocus | AnyColorless => banner.focus_sizes[3] > 0,
            RedFourstarFocus => {
                banner.fourstar_focus == Some(Color::Red) && banner.focus_sizes[0] > 0
            }
            BlueFourstarFocus => {
                banner.fourstar_focus == Some(Color::Blue) && banner.focus_sizes[1] > 0
            }
            GreenFourstarFocus => {
                banner.fourstar_focus == Some(Color::Green) && banner.focus_sizes[2] > 0
            }
            ColorlessFourstarFocus => {
                banner.fourstar_focus == Some(Color::Colorless) && banner.focus_sizes[3] > 0
            }
        }
    }

    /// Says whether or not the preset has only a single unit that counts for
    /// completing the goal.
    fn is_single_target(&self) -> bool {
        use GoalPreset::*;
        match self {
            RedFocus
            | BlueFocus
            | GreenFocus
            | ColorlessFocus
            | RedFourstarFocus
            | BlueFourstarFocus
            | GreenFourstarFocus
            | ColorlessFourstarFocus => true,
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
    pub four_star: bool,
}

/// The flexible representation of a goal
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomGoal {
    pub kind: GoalKind,
    pub goals: Vec<GoalPart>,
}

/// The goal of a summoning session.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Goal {
    Custom(CustomGoal),
    Preset(GoalPreset, u8),
}

impl Default for Goal {
    fn default() -> Self {
        Goal::Preset(GoalPreset::AnyFocus, 1)
    }
}

impl Goal {
    /// Convert the current preset into a custom goal or retreive the current
    /// custom goal.
    pub fn as_custom(&self, banner: &Banner) -> CustomGoal {
        use crate::goal::GoalKind::*;
        use crate::goal::GoalPreset::*;
        use crate::Color::*;

        let (preset, count) = match self {
            Goal::Preset(preset, count) => (*preset, *count),
            Goal::Custom(custom) => return custom.clone(),
        };

        let count = if preset.is_single_target() {
            count.max(1)
        } else {
            1
        };

        let kind = match preset {
            AllFocus => All,
            // Every other preset is either Any* or has only one target
            _ => Any,
        };
        let mut custom_goal = CustomGoal {
            kind,
            goals: vec![],
        };

        let mut add_color_goal = |color: Color, four_star: bool| {
            custom_goal.goals.push(GoalPart {
                unit_color: color,
                num_copies: count,
                four_star,
            });
        };
        // Add an individual GoalPart for each focus unit that matches the
        // conditions of the overall goal.
        match preset {
            AllFocus | AnyFocus => {
                for idx in 0..banner.focus_sizes.len() {
                    for _ in 0..banner.focus_sizes[idx] {
                        add_color_goal(Color::try_from(idx as u8).unwrap(), false);
                    }
                }
            }
            RedFocus => add_color_goal(Red, false),
            BlueFocus => add_color_goal(Blue, false),
            GreenFocus => add_color_goal(Green, false),
            ColorlessFocus => add_color_goal(Colorless, false),
            AnyRed => {
                for _ in 0..banner.focus_sizes[0] {
                    add_color_goal(Red, false)
                }
            }
            AnyBlue => {
                for _ in 0..banner.focus_sizes[1] {
                    add_color_goal(Blue, false)
                }
            }
            AnyGreen => {
                for _ in 0..banner.focus_sizes[2] {
                    add_color_goal(Green, false)
                }
            }
            AnyColorless => {
                for _ in 0..banner.focus_sizes[3] {
                    add_color_goal(Colorless, false)
                }
            }
            RedFourstarFocus => add_color_goal(Red, true),
            BlueFourstarFocus => add_color_goal(Blue, true),
            GreenFourstarFocus => add_color_goal(Green, true),
            ColorlessFourstarFocus => add_color_goal(Colorless, true),
        }

        custom_goal
    }

    /// Checks whether or not the goal is possible on the given banner.
    pub fn is_available(&self, banner: &Banner) -> bool {
        match self {
            Goal::Custom(custom_goal) => custom_goal
                .goals
                .iter()
                .any(|&GoalPart { unit_color, .. }| banner.focus_sizes[unit_color as usize] > 0),
            Goal::Preset(preset, _) => preset.is_available(banner),
        }
    }

    /// Parses data from the representation used in query strings to share settings.
    pub fn from_query_string(s: &str) -> Option<Self> {
        let data = base64::decode(s).ok()?;
        bincode::deserialize(&data).ok()
    }
}

/// Section for selecting the goal.
pub fn goal_selector(goal: &Goal, banner: &Banner) -> Node<Msg> {
    let mut select = select![
        id!["goal"],
        input_ev("input", |text| {
            if let Some(preset) = text
                .parse::<u8>()
                .ok()
                .and_then(|id| GoalPreset::try_from(id).ok())
            {
                Msg::GoalPresetChange { preset }
            } else if text == "custom" {
                Msg::GoalMakeCustom
            } else {
                Msg::Null
            }
        }),
    ];
    select.add_child(option![
        attrs![
            At::Value => "custom";
        ],
        if let Goal::Custom(_) = goal {
            attrs![
                At::Selected => "";
            ]
        } else {
            attrs![]
        },
        "Custom goal",
    ]);
    for preset in GoalPreset::iter() {
        let mut attrs = attrs! [
            At::Value => preset as usize;
        ];
        if !preset.is_available(banner) {
            attrs.add(At::Disabled, "");
        } else if let Goal::Preset(goal_preset, _) = goal {
            if *goal_preset == preset {
                attrs.add(At::Selected, "");
            }
        }
        select.add_child(option![attrs, preset.to_string(),]);
    }
    div![
        id!["goal_selector"],
        select,
        if let Goal::Preset(preset, count) = goal {
            if preset.is_single_target() {
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
                                Msg::GoalPresetQuantityChange { quantity }
                            } else {
                                Msg::GoalPresetQuantityChange { quantity: 0 }
                            }
                        }),
                        attrs! [
                            At::Type => "number";
                            At::Value => if *count > 0 {
                                count.to_string()
                            } else {
                                "".to_string()
                            };
                            At::Class => "small_number";
                            At::Min => 1;
                            At::Required => true;
                        ],
                    ]
                ]
            } else {
                seed::empty()
            }
        } else {
            seed::empty()
        },
        advanced_goal_selector(goal),
    ]
}

/// Subsection for selecting the goal using the detailed representation instead of
/// a preset.
fn advanced_goal_selector(goal: &Goal) -> Node<Msg> {
    if let Goal::Custom(custom_goal) = goal {
        let mut base = div![style!["margin-left" => "2em";]];
        if custom_goal.goals.len() > 1 {
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
                    if custom_goal.kind == GoalKind::Any {
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
                    if custom_goal.kind == GoalKind::All {
                        attrs![At::Selected => ""]
                    } else {
                        attrs![]
                    },
                    "All of these",
                ],
            ]);
        }

        for (index, goal_part) in custom_goal.goals.iter().enumerate() {
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
                color_select.add_child(option![attrs, color.to_string()]);
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
    } else {
        seed::empty()
    }
}
