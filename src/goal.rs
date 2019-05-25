use seed::prelude::*;

use std::convert::TryFrom;

use crate::{banner::Banner, Color, Msg};

#[derive(Copy, Clone, Debug)]
enum GoalKind {
    //Any,
    All,
}

#[derive(Copy, Clone, Debug)]
pub struct GoalPart {
    pub unit_color: Color,
    pub num_copies: u8,
}

#[derive(Clone, Debug)]
pub struct Goal {
    kind: GoalKind,
    pub goals: Vec<GoalPart>,
}

impl Default for Goal {
    fn default() -> Self {
        Goal {
            kind: GoalKind::All,
            goals: vec![GoalPart {
                unit_color: Color::Red,
                num_copies: 1,
            }],
        }
    }
}

impl Goal {
    pub fn has_colors(&self) -> [bool; 4] {
        let mut result = [false; 4];
        for &goal in &self.goals {
            result[goal.unit_color as usize] = true;
        }
        result
    }

    pub fn has_color(&self, color: Color) -> bool {
        self.has_colors()[color as usize]
    }
}

#[derive(Copy, Clone, Debug)]
pub enum GoalChangeKind {
    Color,
    Quantity,
}

pub fn goal_selector(goal: &Goal, banner: &Banner) -> El<Msg> {
    let mut select = select![
        id!["goal"],
        input_ev("input", |text| Msg::GoalChange {
            text,
            kind: GoalChangeKind::Color
        }),
    ];
    let mut any_selected = false;
    for color in (0u8..4).map(|c| Color::try_from(c).unwrap()) {
        if banner.focus_sizes[color as usize] > 0 {
            let mut attrs = attrs! [
                At::Value => color as usize;
            ];
            if !any_selected && goal.has_color(color) {
                attrs.add(At::Selected, "");
                any_selected = true;
            }
            select.add_child(option![attrs, format!("{:?} focus", color),])
        }
    }
    div![
        id!["goal_selector"],
        select,
        label![
            attrs! [
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
                At::Value => goal.goals[0].num_copies;
                At::Class => "small_number";
                At::Min => 1;
                At::Required => true;
            ],
        ],
    ]
}
