use seed::prelude::*;

use crate::counter::Counter;

use crate::Msg;

mod svg_graph;

pub fn results(data: &Counter) -> El<Msg> {
    div![id!["results"], class!["no-select"], svg_graph::graph(data),]
}
