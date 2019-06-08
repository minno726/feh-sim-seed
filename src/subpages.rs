use crate::Msg;
use seed::prelude::*;


/// The header of one of the informational pages.
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

/// Page contents for the help page.
pub fn help() -> Vec<El<Msg>> {
    let mut els = vec![header()];
    els.extend(El::from_markdown(include_str!("subpages/help.md")));
    els
}

/// Page contents for the changelog page.
pub fn changelog() -> Vec<El<Msg>> {
    let mut els = vec![header()];
    els.extend(El::from_markdown(include_str!("subpages/changelog.md")));
    els
}
