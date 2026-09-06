use std::collections::HashMap;
use std::sync::OnceLock;

use eso_skill_data::enums::ability_tag::AbilityTag;
use yew::prelude::*;
use yew_router::components::Link;

use crate::{Route, player_ability_ids};
use crate::fetch::get_skill;
use crate::format::render_ability_with_summary;
use crate::id::get_abilities;

const MAX_RESULTS: usize = 5000;

static TAG_COUNTS: OnceLock<HashMap<u16, usize>> = OnceLock::new();

fn tag_counts() -> &'static HashMap<u16, usize> {
    TAG_COUNTS.get_or_init(|| {
        let abilities = get_abilities();
        let mut ids: Vec<u32> = abilities.keys().copied().collect();
        ids.sort();

        let mut counts: HashMap<u16, usize> = HashMap::new();

        for id in ids {
            let Some(skill) = get_skill(&id) else {
                continue;
            };

            for tag in &skill.ability_tags {
                *counts.entry(*tag).or_insert(0) += 1;
            }
        }

        counts
    })
}

fn collect_tag_matches(tag_id: u16) -> (Vec<u32>, usize) {
    let abilities = get_abilities();

    let mut ids: Vec<u32> = abilities.keys().copied().collect();
    ids.sort();

    let mut matching: Vec<u32> = Vec::new();
    let mut matching_total = 0usize;

    for id in ids {
        let Some(skill) = get_skill(&id) else {
            continue;
        };

        if skill.ability_tags.contains(&tag_id) {
            matching_total += 1;
            if matching.len() < MAX_RESULTS {
                matching.push(id);
            }
        }
    }

    let player_ids = player_ability_ids();
    matching.sort_by_key(|id| (!player_ids.contains(id), *id));

    (matching, matching_total)
}

fn render_id_list(ids: &[u32], ability_names: &HashMap<u32, String>) -> Html {
    html! {
        <div>
            { for ids.iter().map(|id| html! {
                <div style="margin: 1px;">
                    { render_ability_with_summary(id, ability_names.get(id).map(String::as_str).unwrap_or("???")) }
                </div>
            }) }
        </div>
    }
}

fn tag_label(tag_id: u16) -> String {
    match AbilityTag::from_id(&tag_id) {
        Some(tag) => format!("{} ({})", tag.as_str(), tag_id),
        None => format!("Tag {}", tag_id),
    }
}

#[derive(Properties, PartialEq)]
pub struct TagProps {
    pub index: String,
}

#[function_component(TagSummary)]
pub fn tag_summary(props: &TagProps) -> Html {
    let tag_id: u16 = props.index.trim().parse().unwrap_or(0);
    let ability_names = get_abilities();

    let (matching, matching_total) = collect_tag_matches(tag_id);

    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
        document.set_title(format!("{} - ESO ID Dictionary", tag_label(tag_id)).as_str());
    }

    html! {
        <div>
            <nav style="margin-bottom: 1em;">
                <Link<Route> to={Route::Home}>
                    {"ESO ID Dictionary"}
                </Link<Route>>
                <span>{ format!(" / Tag / {}", tag_label(tag_id)) }</span>
            </nav>
            <p>{ format!("{} abilities with this tag", matching_total) }</p>
            { render_id_list(&matching, ability_names) }
        </div>
    }
}

#[function_component(TagsSummary)]
pub fn tags_summary() -> Html {
    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
        document.set_title("Tags - ESO ID Dictionary");
    }

    let counts = tag_counts();
    let mut rows: Vec<(u16, usize)> = counts.iter().map(|(id, count)| (*id, *count)).collect();
    rows.sort_by_key(|(id, _)| *id);

    html! {
        <div>
            <nav style="margin-bottom: 1em;">
                <Link<Route> to={Route::Home}>
                    {"ESO ID Dictionary"}
                </Link<Route>>
                <span>{ " / Tags" }</span>
            </nav>
            <table style="border-collapse: collapse; width: 100%; max-width: 40em;">
                <thead>
                    <tr>
                        <th style="text-align: left; border-bottom: 1px solid #888; padding: 0.25em 0.5em;">{"Tag"}</th>
                        <th style="text-align: right; border-bottom: 1px solid #888; padding: 0.25em 0.5em;">{"Count"}</th>
                    </tr>
                </thead>
                <tbody>
                    { for rows.iter().map(|(id, count)| html! {
                        <tr key={*id}>
                            <td style="padding: 0.15em 0.5em;">
                                { tag_label(*id) }
                            </td>
                            <td style="text-align: right; padding: 0.15em 0.5em;">
                                <Link<Route> to={Route::Tag { index: id.to_string() }}>
                                    { count }
                                </Link<Route>>
                            </td>
                        </tr>
                    }) }
                </tbody>
            </table>
        </div>
    }
}
