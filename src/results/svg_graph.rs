use seed::prelude::*;

use std::fmt::Write;

use crate::counter::Counter;
use crate::Msg;

use crate::stats;

const XMIN: f32 = 0.0;
const YMIN: f32 = 0.0;
const WIDTH: f32 = 100.0;
const HEIGHT: f32 = 60.0;

fn graph_line(data: &Counter) -> (El<Msg>, El<Msg>) {
    let nsamples = 1000;
    let data_points = (0..nsamples)
        .map(|idx| stats::percentile(data, idx as f32 / nsamples as f32) as f32)
        .collect::<Vec<_>>();
    let x = |pct: f32| pct as f32 * WIDTH + XMIN;
    let y = |val: f32| {
        let max = *data_points.last().unwrap() as f32;
        HEIGHT - (val / max) * HEIGHT
    };
    let mut path = String::new();
    if !data.is_empty() {
        write!(path, "M {} {} ", x(0.0), y(data_points[0])).unwrap();
        for i in 1..data_points.len() {
            if data_points[i] != data_points[i - 1] {
                write!(
                    path,
                    "L {} {}",
                    x(i as f32 / nsamples as f32),
                    y(data_points[i])
                )
                .unwrap();
            }
        }
    }
    let path_el = path![
        id!["graph_line"],
        attrs![
            "d" => path;
        ],
    ];
    let mut points_el = g![id!["graph_highlights"],];
    let mut add_point = |pct: f32| {
        let value = stats::percentile(data, pct);
        points_el.add_child(circle![attrs![
            "cx" => x(pct);
            "cy" => y(value as f32);
            "r" => "0.75px";
        ]]);
        points_el.add_child(text![
            attrs![
                "dx" => x(pct) - 1.0;
                "dy" => y(value as f32) - 1.0;
                "text-anchor" => "end";
                "font-size" => "15%";
            ],
            format!("{}%: {} orbs", pct * 100.0, value),
        ]);
    };
    if !data.is_empty() {
        for &pct in &[0.25, 0.5, 0.75, 0.9, 0.99] {
            add_point(pct);
        }
    }
    (path_el, points_el)
}

pub fn graph(data: &Counter) -> El<Msg> {
    let (path_el, points_el) = graph_line(data);
    svg![
        id!["graph"],
        attrs![
            "viewBox" => format!("{} {} {} {}", XMIN, YMIN, WIDTH, HEIGHT);
        ],
        path_el,
        if !data.is_empty() {
            text![
                id!["graph_sample_count"],
                attrs![
                    "dominant-baseline" => "hanging";
                    "font-size" => "10%";
                ],
                format!("{} samples", data.iter().sum::<u32>()),
            ]
        } else {
            seed::empty()
        },
        points_el,
    ]
}
