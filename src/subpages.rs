use crate::Msg;
use seed::prelude::*;
fn header() -> El<Msg> {
    header![
        style![
            "text-align" => "start";
        ],
        a![
            "Back",
            attrs! [
                At::Href => "/fehstatsim/";
            ]
        ],
    ]
}

pub fn help() -> Vec<El<Msg>> {
    let mut els = vec![header()];
    els.extend(El::from_markdown(include_str!("subpages/help.md")));
    els
}

pub fn changelog() -> Vec<El<Msg>> {
    let mut els = vec![header()];
    els.extend(El::from_markdown(include_str!("subpages/changelog.md")));
    els
}
