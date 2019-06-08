use seed::prelude::*;

use std::fmt::Write;

use wasm_bindgen::JsCast;

use crate::counter::Counter;
use crate::stats;
use crate::Msg;
const XMIN: f32 = 0.0;
const YMIN: f32 = 0.0;
const WIDTH: f32 = 100.0;
const HEIGHT: f32 = 60.0;

/// SVG elements for displaying the results within the graph. If `highlight` is
/// given, places a label on the graph at the specified point. Otherwise, labels
/// are placed at pre-set locations. Returns two elements, one for the line and
/// one for the collection of labels.
fn graph_line(data: &Counter, highlight: Option<f32>) -> (El<Msg>, El<Msg>) {
    // Sample every 0.1% in ranges 0%-10% and 90%-100%, and every 1% in between.
    // Probabilities only change sharply near the extremes, so this makes things
    // render more quickly without hurting smoothness.
    let sample_points = (0..100)
        .map(|x| x as f32 / 1000.0)
        .chain((10..90).map(|x| x as f32 / 100.0))
        .chain((900..1000).map(|x| x as f32 / 1000.0))
        .collect::<Vec<_>>();
    let data_points = stats::percentiles(data, &sample_points);

    // Helper functions for converting between data values and graph coordinates.
    let x = |pct: f32| pct as f32 * WIDTH + XMIN;
    let y = |val: f32| {
        let max = *data_points.last().unwrap() as f32;
        HEIGHT - (val / max) * HEIGHT
    };

    let mut path = String::new();
    if !data.is_empty() {
        write!(
            path,
            "M {} {} ",
            x(sample_points[0]),
            y(data_points[0] as f32)
        )
        .unwrap();
        for i in 1..data_points.len() {
            if data_points[i] != data_points[i - 1] {
                write!(
                    path,
                    "L {} {}",
                    x(sample_points[i]),
                    y(data_points[i] as f32)
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
        let value = stats::percentile(data, pct) as f32;
        points_el.add_child(circle![attrs![
            "cx" => x(pct);
            "cy" => y(value);
            "r" => "0.75px";
        ]]);
        let label_text = format!("{}%: {} orbs", (pct * 1000.0).round() / 10.0, value);
        points_el.add_child(text![
            attrs![
                "font-size" => "15%";
            ],
            // By default, put the label up and to the left of the point.
            if pct > 0.24 {
                attrs![
                    "dx" => x(pct) - 1.0;
                    "dy" => y(value) - 1.0;
                    "text-anchor" => "end";
                    "dominant-baseline" => "baseline";
                ]
            } else {
                // If the point is too far to the left for the label to fit,
                // then put it down and to the right if there is room there.
                if y(value) < HEIGHT * 0.9 {
                    attrs![
                        "dx" => x(pct) + 1.0;
                        "dy" => y(value) + 1.0;
                        "text-anchor" => "begin";
                        "dominant-baseline" => "hanging";
                    ]
                } else {
                    // If there isn't room in either of those places, then just
                    // snap it to the left edge, high enough to not intersect
                    // the graph line.
                    attrs![
                        "dx" => 1.0;
                        "dy" => y(stats::percentile(data, 0.24) as f32) - 1.0;
                        "text-anchor" => "begin";
                        "dominant-baseline" => "baseline";
                    ]
                }
            },
            label_text,
        ]);
    };
    if !data.is_empty() {
        if let Some(highlight) = highlight {
            add_point(highlight);
        } else {
            for &pct in &[0.25, 0.5, 0.75, 0.9, 0.99] {
                add_point(pct);
            }
        }
    }
    (path_el, points_el)
}

/// Graph for displaying the results. If `highlight` is given, places a label
/// on the graph at the specified point. Otherwise, labels are placed at pre-set
/// locations.
pub fn graph(data: &Counter, highlight: Option<f32>) -> El<Msg> {
    let (path_el, points_el) = graph_line(data, highlight);
    fn get_graph_width(event: &web_sys::Event) -> Option<f64> {
        let target = event.target()?;
        let target_el: &web_sys::Element = target.dyn_ref::<web_sys::SvgsvgElement>()?.as_ref();
        let width = target_el.get_bounding_client_rect().width();
        Some(width)
    }
    svg![
        id!["graph"],
        mouse_ev(Ev::Click, |click| {
            if let Some(width) = get_graph_width(&click) {
                let width_frac = (click.offset_x() as f32 / width as f32).min(0.999).max(0.0);
                Msg::GraphHighlight {
                    frac: (1000.0 * width_frac).round() / 1000.0,
                }
            } else {
                Msg::Null
            }
        }),
        attrs![
            At::ViewBox => format!("{} {} {} {}", XMIN, YMIN, WIDTH, HEIGHT);
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
