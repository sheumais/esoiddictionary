use std::collections::HashMap;
use std::sync::OnceLock;

use eso_skill_data::FLAGSIZE;
use eso_skill_data::enums::flags::*;
use yew::prelude::*;
use yew_router::components::Link;

use crate::{Route, player_ability_ids};
use crate::fetch::get_skill;
use crate::format::render_ability_link;
use crate::id::get_abilities;

const MAX_RESULTS: usize = 5000;

fn flag_name(index: usize) -> Option<&'static str> {
    match index {
        FLAG_TOGGLED => Some("Toggled"),
        FLAG_COST_PER_TICK => Some("Cost drained per tick"),
        FLAG_CHANNELED_AOE => Some("Channeled AOE"),
        FLAG_PLAYER_SOURCED_EFFECT => Some("Player-sourced effect"),
        _ => None,
    }
}

static FLAG_COUNTS: OnceLock<HashMap<usize, (usize, usize)>> = OnceLock::new();

fn flag_counts() -> &'static HashMap<usize, (usize, usize)> {
    FLAG_COUNTS.get_or_init(|| {
        let abilities = get_abilities();
        let mut ids: Vec<u32> = abilities.keys().copied().collect();
        ids.sort();

        let mut set_counts: Vec<usize> = vec![0; FLAGSIZE];
        let mut abilities_scanned = 0usize;

        for id in ids {
            let Some(skill) = get_skill(&id) else {
                continue;
            };
            abilities_scanned += 1;

            for i in 0..FLAGSIZE {
                let is_set = skill.flags.get(i).map(|v| *v != 0).unwrap_or(false);
                if is_set {
                    set_counts[i] += 1;
                }
            }
        }

        set_counts
            .into_iter()
            .enumerate()
            .map(|(index, set)| (index, (set, abilities_scanned.saturating_sub(set))))
            .collect()
    })
}

#[derive(Properties, PartialEq)]
pub struct FlagProps {
    pub index: String,
}

fn parse_flag_index(raw: &str) -> (usize, bool) {
    let t = raw.trim();
    match t.strip_prefix('!') {
        Some(rest) => (rest.trim().parse::<usize>().unwrap_or(0), true),
        None => (t.parse::<usize>().unwrap_or(0), false),
    }
}

fn collect_flag_matches(index: usize) -> (Vec<u32>, usize, Vec<u32>, usize) {
    let abilities = get_abilities();

    let mut ids: Vec<u32> = abilities.keys().copied().collect();
    ids.sort();

    let mut matching: Vec<u32> = Vec::new();
    let mut non_matching: Vec<u32> = Vec::new();
    let mut matching_total = 0usize;
    let mut non_matching_total = 0usize;

    for id in ids {
        let Some(skill) = get_skill(&id) else {
            continue;
        };

        let has_flag = skill
            .flags
            .get(index)
            .map(|v| *v != 0)
            .unwrap_or(false);

        if has_flag {
            matching_total += 1;
            if matching.len() < MAX_RESULTS {
                matching.push(id);
            }
        } else {
            non_matching_total += 1;
            if non_matching.len() < MAX_RESULTS {
                non_matching.push(id);
            }
        }
    }

    let player_ids = player_ability_ids();
    matching.sort_by_key(|id| (!player_ids.contains(id), *id));
    non_matching.sort_by_key(|id| (!player_ids.contains(id), *id));

    (matching, matching_total, non_matching, non_matching_total)
}

fn render_id_list(ids: &[u32], ability_names: &HashMap<u32, String>) -> Html {
    html! {
        <div>
            { for ids.iter().map(|id| html! {
                <div style="margin: 1px;">
                    {
                        render_ability_link(
                            id,
                            format!(
                                "{} ({})",
                                ability_names.get(id).unwrap_or(&"???".to_string()),
                                id,
                            ),
                        )
                    }
                    <br />
                </div>
            }) }
        </div>
    }
}

#[function_component(FlagSummary)]
pub fn flag_summary(props: &FlagProps) -> Html {
    let (index, excluded) = parse_flag_index(&props.index);
    let ability_names = get_abilities();

    let (matching, matching_total, non_matching, non_matching_total) =
        collect_flag_matches(index);

    let (shown, other_total) = if excluded {
        (&non_matching, matching_total)
    } else {
        (&matching, non_matching_total)
    };

    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
        let title = if excluded {
            format!("Flag {} excluded - ESO ID Dictionary", index)
        } else {
            format!("Flag {} - ESO ID Dictionary", index)
        };
        document.set_title(title.as_str());
    }

    let toggle_link = if excluded {
        html! {
            <Link<Route> to={Route::Flag { index: index.to_string() }}>
                { format!("See the {} abilities with this flag instead", other_total) }
            </Link<Route>>
        }
    } else {
        html! {
            <Link<Route> to={Route::Flag { index: format!("!{}", index) }}>
                { format!("See the {} abilities without this flag instead", other_total) }
            </Link<Route>>
        }
    };

    html! {
        <div>
            <nav style="margin-bottom: 1em;">
                <Link<Route> to={Route::Home}>
                    {"ESO ID Dictionary"}
                </Link<Route>>
                <span>{ format!(" / Flag / {}{}", index, if excluded { " / without" } else { "" }) }</span>
            </nav>
            if !excluded {
                <p style="margin-top: 1em;">{ toggle_link.clone() }</p>
            }
            { render_id_list(shown, ability_names) }
            if excluded {
                <p style="margin-top: 1em;">{ toggle_link }</p>
            }
        </div>
    }
}

#[function_component(FlagsSummary)]
pub fn flags_summary() -> Html {
    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
        document.set_title("Flags - ESO ID Dictionary");
    }

    let counts = flag_counts();
    let mut rows: Vec<(usize, usize, usize)> = counts
        .iter()
        .map(|(index, (set, unset))| (*index, *set, *unset))
        .collect();
    rows.sort_by_key(|(index, _, _)| *index);

    html! {
        <div>
            <nav style="margin-bottom: 1em;">
                <Link<Route> to={Route::Home}>
                    {"ESO ID Dictionary"}
                </Link<Route>>
                <span>{ " / Flags" }</span>
            </nav>
            <table style="border-collapse: collapse; width: 100%; max-width: 40em;">
                <thead>
                    <tr>
                        <th style="text-align: left; border-bottom: 1px solid #888; padding: 0.25em 0.5em;">{"Flag"}</th>
                        <th style="text-align: right; border-bottom: 1px solid #888; padding: 0.25em 0.5em;">{"Set"}</th>
                        <th style="text-align: right; border-bottom: 1px solid #888; padding: 0.25em 0.5em;">{"Unset"}</th>
                    </tr>
                </thead>
                <tbody>
                    { for rows.iter().map(|(index, set, unset)| html! {
                        <tr key={*index}>
                            <td style="padding: 0.15em 0.5em;">
                                {
                                    match flag_name(*index) {
                                        Some(name) => format!("Flag {} ({})", index, name),
                                        None => format!("Flag {}", index),
                                    }
                                }
                            </td>
                            <td style="text-align: right; padding: 0.15em 0.5em;">
                                <Link<Route> to={Route::Flag { index: index.to_string() }}>
                                    { set }
                                </Link<Route>>
                            </td>
                            <td style="text-align: right; padding: 0.15em 0.5em;">
                                <Link<Route> to={Route::Flag { index: format!("!{}", index) }}>
                                    { unset }
                                </Link<Route>>
                            </td>
                        </tr>
                    }) }
                </tbody>
            </table>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct FlagsCompareProps {
    pub ids: String,
}

fn parse_compare_ids(raw: &str) -> Vec<(u32, bool)> {
    let mut ids = Vec::new();

    for token in raw.split(',') {
        let t = token.trim();
        if t.is_empty() {
            continue;
        }

        let (inverted, num_part) = match t.strip_prefix('!') {
            Some(rest) => (true, rest.trim()),
            None => (false, t),
        };

        if num_part.is_empty() {
            continue;
        }

        match num_part.parse::<u32>() {
            Ok(id) => ids.push((id, inverted)),
            Err(_) => {}
        }
    }

    let mut seen = std::collections::HashSet::new();
    ids.retain(|pair| seen.insert(*pair));

    ids
}

struct CompareRow {
    index: usize,
    matched: usize,
}

fn collect_compare_matches(ids: &[(u32, bool)]) -> (Vec<CompareRow>, usize) {
    let mut match_counts: Vec<usize> = vec![0; FLAGSIZE];
    let mut loaded = 0usize;

    for (id, inverted) in ids {
        let Some(skill) = get_skill(id) else {
            continue;
        };
        loaded += 1;

        for i in 0..FLAGSIZE {
            let is_set = skill.flags.get(i).map(|v| *v != 0).unwrap_or(false);
            let matches = if *inverted { !is_set } else { is_set };
            if matches {
                match_counts[i] += 1;
            }
        }
    }

    let rows: Vec<CompareRow> = match_counts
        .into_iter()
        .enumerate()
        .filter(|(_, matched)| *matched > 0)
        .map(|(index, matched)| CompareRow { index, matched })
        .collect();

    (rows, loaded)
}

#[function_component(FlagsCompare)]
pub fn flags_compare(props: &FlagsCompareProps) -> Html {
    let ability_names = get_abilities();
    let ids = parse_compare_ids(&props.ids);

    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
        document.set_title("Compare Flags - ESO ID Dictionary");
    }

    let (mut rows, loaded) = collect_compare_matches(&ids);

    let counts = flag_counts();

    rows.sort_by(|a, b| {
        b.matched
            .cmp(&a.matched)
            .then_with(|| a.index.cmp(&b.index))
    });

    html! {
        <div>
            <nav style="margin-bottom: 1em;">
                <Link<Route> to={Route::Home}>
                    {"ESO ID Dictionary"}
                </Link<Route>>
                <span>{ " / Flags / Compare" }</span>
            </nav>
            <div style="margin-bottom: 1em;">
                { for ids.iter().enumerate().map(|(i, (id, inverted))| html! {
                    <>
                        if i > 0 { {", "} }
                        if *inverted { {"!"} }
                        {
                            render_ability_link(
                                id,
                                format!(
                                    "{} ({})",
                                    ability_names.get(id).unwrap_or(&"???".to_string()),
                                    id,
                                ),
                            )
                        }
                    </>
                }) }
            </div>
            <table style="border-collapse: collapse; width: 100%; max-width: 40em;">
                <thead>
                    <tr>
                        <th style="text-align: left; padding: 0.25em 0.5em;">{"Flag"}</th>
                        <th style="text-align: right; padding: 0.25em 0.5em;">{"Matched"}</th>
                        <th style="text-align: right; padding: 0.25em 0.5em;">{"Total"}</th>
                    </tr>
                </thead>
                <tbody>
                    { for rows.iter().map(|row| {
                        html! {
                            <tr key={row.index}>
                                <td style="padding: 0.15em 0.5em;">
                                    <Link<Route> to={Route::Flag { index: row.index.to_string() }}>
                                        {
                                            match flag_name(row.index) {
                                                Some(name) => format!("Flag {} ({})", row.index, name),
                                                None => format!("Flag {}", row.index),
                                            }
                                        }
                                    </Link<Route>>
                                </td>
                                <td style="text-align: right; padding: 0.15em 0.5em;">
                                    { format!("{} / {}", row.matched, loaded) }
                                </td>
                                <td style="text-align: right; padding: 0.15em 0.5em;">
                                    { format!("{}", counts.get(&row.index).map(|(set, _)| *set).unwrap_or(0)) }
                                </td>
                            </tr>
                        }
                    }) }
                </tbody>
            </table>
        </div>
    }
}