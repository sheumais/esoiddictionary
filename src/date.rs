use std::sync::OnceLock;

use chrono::{DateTime, NaiveDate};
use yew::prelude::*;
use yew_router::components::Link;

use crate::Route;
use crate::fetch::get_skill;
use crate::format::{render_ability_icon_by_id, render_ability_link};
use crate::id::get_abilities;
use crate::index_state::INDEX_MAP;

const MAX_RESULTS: usize = 500;

static RECENT: OnceLock<Vec<(u32, i64)>> = OnceLock::new();

fn recent_edits() -> &'static Vec<(u32, i64)> {
    RECENT.get_or_init(|| {
        let mut edits: Vec<(u32, i64)> = Vec::new();

        let Some(index) = INDEX_MAP.get() else {
            return edits;
        };

        for &id in index.keys() {
            let Some(skill) = get_skill(&id) else {
                continue;
            };
            let ts = skill.last_edited_timestamp();
            if ts > 0 {
                edits.push((id, ts));
            }
        }

        edits.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        edits.truncate(MAX_RESULTS);
        edits
    })
}

#[function_component(DateSummary)]
pub fn date_summary() -> Html {
    use_effect(|| {
        if let Some(document) = web_sys::window().and_then(|w| w.document()) {
            document.set_title("Recently Edited - ESO ID Dictionary");
        }
        || ()
    });

    let ability_names = get_abilities();
    let edits = recent_edits();

    let mut days: Vec<(NaiveDate, Vec<(u32, i64)>)> = Vec::new();
    for &(id, ts) in edits {
        let Some(dt) = DateTime::from_timestamp(ts, 0) else {
            continue;
        };
        let date = dt.date_naive();
        match days.last_mut() {
            Some((d, ids)) if *d == date => ids.push((id, ts)),
            _ => days.push((date, vec![(id, ts)])),
        }
    }

    html! {
        <div>
            <nav style="margin-bottom: 1em;">
                <Link<Route> to={Route::Home}>
                    {"ESO ID Dictionary"}
                </Link<Route>>
                <span>{ " / Recently Edited" }</span>
            </nav>
            { for days.iter().map(|(date, entries)| html! {
                <div key={date.to_string()}>
                    <h4 style="margin-bottom: 0.25em;">
                        { date.format("%A, %-d %B %Y").to_string() }
                        { format!(" ({})", entries.len()) }
                    </h4>
                    { for entries.iter().map(|(id, ts)| {
                        let name = ability_names
                            .get(id)
                            .map(|n| n.trim_matches('"'))
                            .unwrap_or("???");
                        let time = DateTime::from_timestamp(*ts, 0)
                            .map(|dt| dt.format("%H:%M:%S").to_string())
                            .unwrap_or_default();
                        html! {
                            <div key={*id} style="font-size: 0.9em; margin: 1px;">
                                <span style="color: #555; font-variant-numeric: tabular-nums;">{ format!("{} ", time) }</span>
                                { render_ability_icon_by_id(id, 16) }
                                { render_ability_link(id, format!("{} ({})", name, id)) }
                            </div>
                        }
                    }) }
                </div>
            }) }
        </div>
    }
}
