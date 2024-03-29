use seed::prelude::*;

use crate::counter::Counter;
use crate::Msg;

mod svg_graph;

/// Section for displaying the results. If `highlight` is given, places a label
/// on the graph at the specified point. Otherwise, labels are placed at pre-set
/// locations.
pub fn results(data: &Counter, highlight: Option<f32>) -> Node<Msg> {
    div![id!["results"], svg_graph::graph(data, highlight),]
}
