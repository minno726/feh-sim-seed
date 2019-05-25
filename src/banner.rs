use seed::prelude::*;

use crate::Msg;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
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

#[derive(Copy, Clone, Debug)]
pub enum BannerChangeKind {
    FocusSize(usize),
    StartingRates,
}

pub fn banner_selector(banner: &Banner) -> El<Msg> {
    let option = |rates: (u8, u8), label: &str| -> El<Msg> {
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
            input_ev("input", |text| Msg::BannerChange {
                text,
                kind: BannerChangeKind::StartingRates
            }),
            option((3, 3), "3%/3% (Normal)"),
            option((5, 3), "5%/3% (Hero Fest)"),
            option((8, 0), "8%/0% (Legendary)"),
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
                input_ev("input", |text| Msg::BannerChange {
                    text,
                    kind: BannerChangeKind::FocusSize(0)
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
                input_ev("input", |text| Msg::BannerChange {
                    text,
                    kind: BannerChangeKind::FocusSize(1)
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
                input_ev("input", |text| Msg::BannerChange {
                    text,
                    kind: BannerChangeKind::FocusSize(2)
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
                input_ev("input", |text| Msg::BannerChange {
                    text,
                    kind: BannerChangeKind::FocusSize(3)
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
