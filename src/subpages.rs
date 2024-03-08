use crate::Msg;
use seed::prelude::*;

/// The header of one of the informational pages.
fn header() -> Node<Msg> {
    header![
        style![
            "text-align" => "start";
        ],
        a![
            "Back",
            attrs! [
                At::Href => "/";
            ]
        ],
    ]
}

/// Page contents for the help page.
pub fn help() -> Vec<Node<Msg>> {
    let mut els = vec![header()];
    els.extend(El::from_markdown(include_str!("subpages/help.md")));
    els
}

/// Page contents for the changelog page.
pub fn changelog() -> Vec<Node<Msg>> {
    let mut els = vec![header()];
    els.extend(El::from_markdown(include_str!("subpages/changelog.md")));
    els
}
