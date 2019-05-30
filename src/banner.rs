use seed::prelude::*;

use crate::{Color, Msg};

#[derive(Copy, Clone, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Banner {
    pub focus_sizes: [u8; 4],
    pub starting_rates: (u8, u8),
}

impl Default for Banner {
    fn default() -> Self {
        Banner {
            focus_sizes: [1, 1, 1, 1],
            starting_rates: (3, 3),
        }
    }
}

impl Banner {
    pub fn from_query_string(s: &str) -> Option<Self> {
        let data = base64::decode(s).ok()?;
        bincode::deserialize(&data).ok()
    }
}

pub fn banner_selector(banner: &Banner) -> El<Msg> {
    let rate_option = |rates: (u8, u8), label: &str| -> El<Msg> {
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
                    if let Ok(quantity) = text.parse::<u8>() {
                        Msg::BannerFocusSizeChange {
                            color: Color::Red,
                            quantity,
                        }
                    } else {
                        Msg::Null
                    }
                }),
                attrs![
                    At::Type => "number";
                    At::Class => "small_number";
                    At::Value => banner.focus_sizes[0];
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
                    if let Ok(quantity) = text.parse::<u8>() {
                        Msg::BannerFocusSizeChange {
                            color: Color::Blue,
                            quantity,
                        }
                    } else {
                        Msg::Null
                    }
                }),
                attrs![
                    At::Type => "number";
                    At::Class => "small_number";
                    At::Value => banner.focus_sizes[1];
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
                    if let Ok(quantity) = text.parse::<u8>() {
                        Msg::BannerFocusSizeChange {
                            color: Color::Green,
                            quantity,
                        }
                    } else {
                        Msg::Null
                    }
                }),
                attrs![
                    At::Type => "number";
                    At::Class => "small_number";
                    At::Value => banner.focus_sizes[2];
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
                    if let Ok(quantity) = text.parse::<u8>() {
                        Msg::BannerFocusSizeChange {
                            color: Color::Colorless,
                            quantity,
                        }
                    } else {
                        Msg::Null
                    }
                }),
                attrs![
                    At::Type => "number";
                    At::Class => "small_number";
                    At::Value => banner.focus_sizes[3];
                    At::Min => 0;
                    At::Required => true;
                ],
            ],
        ],
    ]
}
