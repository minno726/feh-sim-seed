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

const HELP_MD: &str = r#"
## Settings

### Goal

Choose a goal from the dropdown menu. The options are:

* **Custom goal** - not implemented yet. Plan is to allow goals like "get a +10 of this and also get one of this other one along the way".
* **Any focus unit** - take every orb that has a focus unit, and continue until any focus unit appears.
* **All focus units** - take every orb that has a focus unit that hasn't been acquired yet, and continue until they have all appeared.
* **Specific <color> focus unit** - take every orb of that color, and continue until a certain one of that color's focus units appears, ignoring any others that share that color.
* **Any <color> focus unit** - take every orb of that color, and continue until any of the focus units from that color appears.

The *count* option isn't working. I'm planning to get it working before I publish this. If this sentence appears on the web page after the update, I messed up.

### Banner selection

Select the starting rates from the dropdown menu.

Enter the number of focus units that the banner has on each color in the R/B/G/C boxes.

## Results

The graph shows how many orbs you need to spend to get a certain percent chance of reaching your goal, with labels at a few milestones for hard numbers. Each label shows the number of orbs spent before the indicated percentage of simulated results reach the goal.

Don't forget that there is no amount of spending that can guarantee that you reach the goal. The 99th percentile shows a really high cost, but one out of every hundred people who read this will spend more than that next time they go to summon.
"#;

pub fn help() -> Vec<El<Msg>> {
    let mut els = vec![header()];
    els.extend(El::from_markdown(HELP_MD));
    els
}

const CHANGELOG_MD: &str = r#"
#### v0.1.0 - Date TBD

* (TODO) Make the interface look better and be easier to use.

* Add advanced goals.

* Add permalink for saving/sharing banner and goal settings.

* (TODO) Add control for checking costs at arbitrary percentiles.

* (TODO) Add control for checking what percentile a certain cost is.

#### v0.0.3 - 7 Mar 2019

* Add graph of results

#### v0.0.2 - 22 Feb 2019

* Add option to try for multiple copies of a unit.

* Make banner selector easier to use.

#### v0.0.1 - 19 Feb 2019

* Initial release.
"#;

pub fn changelog() -> Vec<El<Msg>> {
    let mut els = vec![header()];
    els.extend(El::from_markdown(CHANGELOG_MD));
    els
}
