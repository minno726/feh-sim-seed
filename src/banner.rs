use seed::prelude::*;

use crate::{Color, Msg};

/// Representation of a summoning focus.
#[derive(Copy, Clone, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Banner {
    pub focus_sizes: [i8; 4],
    pub starting_rates: (u8, u8),
    pub focus_charges: bool,
    pub fourstar_focus: Option<Color>,
}

impl Default for Banner {
    fn default() -> Self {
        Banner {
            focus_sizes: [1, 1, 1, 1],
            starting_rates: (3, 3),
            focus_charges: true,
            fourstar_focus: None,
        }
    }
}

impl Banner {
    /// Parses data from the representation used in query strings to share settings.
    pub fn from_query_string(s: &str) -> Option<Self> {
        let data = base64::decode(s).ok()?;
        bincode::deserialize(&data).ok()
    }
}

/// Section for choosing banner parameters.
pub fn banner_selector(banner: &Banner) -> Node<Msg> {
    let rate_option = |rates: (u8, u8), label: &str| -> Node<Msg> {
        let mut attrs = attrs![
            At::Value => format!("{} {}", rates.0, rates.1);
        ];
        if rates == banner.starting_rates {
            attrs.add(At::Selected, "");
        }
        option![attrs, label]
    };
    div![
        id!["banner_selector"],
        div![
            select![
                id!["starting_rates"],
                input_ev("input", |text| {
                    if let &[Ok(first), Ok(second)] = &*text
                        .split_whitespace()
                        .map(str::parse::<u8>)
                        .collect::<Vec<_>>()
                    {
                        Msg::BannerRateChange {
                            rates: (first, second),
                        }
                    } else {
                        Msg::Null
                    }
                }),
                rate_option((3, 3), "3%/3% (Normal)"),
                rate_option((5, 3), "5%/3% (Hero Fest)"),
                rate_option((8, 0), "8%/0% (Legendary)"),
                rate_option((4, 2), "4%/2% (Weekly Focus)"),
                rate_option((6, 0), "6%/0% (Double Special Heroes)"),
            ],
            input![
                id!["focus_charges_banner"],
                simple_ev(Ev::Input, Msg::BannerFocusChargesToggle),
                attrs![At::Type => "checkbox"; At::Checked => banner.focus_charges.as_at_value()],
            ],
            label![attrs![At::For => "focus_charges_banner"], "Focus charges?"]
        ],
        div![
            id!["focus_counts"],
            label![
                attrs![
                    At::For => "focus_count_r";
                ],
                "R:",
            ],
            input![
                id!["focus_count_r"],
                input_ev("input", |text| {
                    if let Ok(quantity) = text.parse::<i8>() {
                        Msg::BannerFocusSizeChange {
                            color: Color::Red,
                            quantity,
                        }
                    } else {
                        Msg::BannerFocusSizeChange {
                            color: Color::Red,
                            quantity: -1,
                        }
                    }
                }),
                attrs![
                    At::Type => "number";
                    At::Class => "small_number";
                    At::Value => if banner.focus_sizes[0] >= 0 {
                        banner.focus_sizes[0].to_string()
                    } else {
                        "".to_string()
                    };
                    At::Min => 0;
                    At::Required => true;
                ]
            ],
            label![
                attrs![
                    At::For => "focus_count_b";
                ],
                "B:",
            ],
            input![
                id!["focus_count_b"],
                input_ev("input", |text| {
                    if let Ok(quantity) = text.parse::<i8>() {
                        Msg::BannerFocusSizeChange {
                            color: Color::Blue,
                            quantity,
                        }
                    } else {
                        Msg::BannerFocusSizeChange {
                            color: Color::Blue,
                            quantity: -1,
                        }
                    }
                }),
                attrs![
                    At::Type => "number";
                    At::Class => "small_number";
                    At::Value => if banner.focus_sizes[1] >= 0 {
                        banner.focus_sizes[1].to_string()
                    } else {
                        "".to_string()
                    };
                    At::Min => 0;
                    At::Required => true;
                ]
            ],
            label![
                attrs![
                    At::For => "focus_count_g";
                ],
                "G:",
            ],
            input![
                id!["focus_count_g"],
                input_ev("input", |text| {
                    if let Ok(quantity) = text.parse::<i8>() {
                        Msg::BannerFocusSizeChange {
                            color: Color::Green,
                            quantity,
                        }
                    } else {
                        Msg::BannerFocusSizeChange {
                            color: Color::Green,
                            quantity: -1,
                        }
                    }
                }),
                attrs![
                    At::Type => "number";
                    At::Class => "small_number";
                    At::Value => if banner.focus_sizes[2] >= 0 {
                        banner.focus_sizes[2].to_string()
                    } else {
                        "".to_string()
                    };
                    At::Min => 0;
                    At::Required => true;
                ]
            ],
            label![
                attrs![
                    At::For => "focus_count_c";
                ],
                "C:",
            ],
            input![
                id!["focus_count_c"],
                input_ev("input", |text| {
                    if let Ok(quantity) = text.parse::<i8>() {
                        Msg::BannerFocusSizeChange {
                            color: Color::Colorless,
                            quantity,
                        }
                    } else {
                        Msg::BannerFocusSizeChange {
                            color: Color::Colorless,
                            quantity: -1,
                        }
                    }
                }),
                attrs![
                    At::Type => "number";
                    At::Class => "small_number";
                    At::Value => if banner.focus_sizes[3] >= 0 {
                        banner.focus_sizes[3].to_string()
                    } else {
                        "".to_string()
                    };
                    At::Min => 0;
                    At::Required => true;
                ],
            ],
            if banner.starting_rates == (3, 3) {
                nodes![
                    label![
                        attrs![
                            At::For => "fourstar_focus";
                        ],
                        "4* focus:",
                    ],
                    select![
                        id!["fourstar_focus"],
                        input_ev("input", |text| {
                            use Color::*;
                            let focus = match &*text {
                                "Red" => Some(Red),
                                "Blue" => Some(Blue),
                                "Green" => Some(Green),
                                "Colorless" => Some(Colorless),
                                _ => None,
                            };

                            Msg::BannerFourstarFocusChange { focus }
                        }),
                        option![
                            if banner.fourstar_focus == None {
                                attrs![At::Selected => "1"]
                            } else {
                                attrs![]
                            },
                            "None"
                        ],
                        option![
                            if banner.fourstar_focus == Some(Color::Red) {
                                attrs![At::Selected => "1"]
                            } else {
                                attrs![]
                            },
                            "Red"
                        ],
                        option![
                            if banner.fourstar_focus == Some(Color::Blue) {
                                attrs![At::Selected => "1"]
                            } else {
                                attrs![]
                            },
                            "Blue"
                        ],
                        option![
                            if banner.fourstar_focus == Some(Color::Green) {
                                attrs![At::Selected => "1"]
                            } else {
                                attrs![]
                            },
                            "Green"
                        ],
                        option![
                            if banner.fourstar_focus == Some(Color::Colorless) {
                                attrs![At::Selected => "1"]
                            } else {
                                attrs![]
                            },
                            "Colorless"
                        ],
                    ],
                ]
            } else {
                vec![]
            }
        ],
    ]
}
