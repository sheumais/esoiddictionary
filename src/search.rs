use std::collections::BTreeMap;
use std::{collections::HashMap, sync::OnceLock};

use chrono::Datelike;
use eso_skill_data::SkillData34;
use eso_skill_data::enums::ability_type::AbilityType;
use eso_skill_data::enums::damage_type::DamageType;
use eso_skill_data::enums::skill_line::SkillLine;
use web_sys::HtmlInputElement;
use yew::Html;
use yew::prelude::*;
use yew_router::components::Link;
use yew_router::hooks::use_navigator;

use crate::Route;
use crate::fetch::{get_skill, read_bytes};
use crate::format::{format_angle, format_distance, format_duration, get_value_adjusted, render_ability_link};
use crate::index_state::find_entry;
use crate::{SKILL_CSV, get_timestamps, id::get_abilities};

static SKILL_GROUPS: OnceLock<BTreeMap<u32, Vec<u32>>> = OnceLock::new();

pub fn get_groups() -> &'static BTreeMap<u32, Vec<u32>> {
    SKILL_GROUPS.get_or_init(|| {
        let mut map: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
        for line in SKILL_CSV.lines() {
            let mut parts = line.splitn(3, ',');
            if let (Some(id), Some(sl), Some(_ts)) = (parts.next(), parts.next(), parts.next())
                && let (Ok(id), Ok(sl)) = (id.trim().parse::<u32>(), sl.trim().parse::<u32>()) {
                    map.entry(sl).or_default().push(id);
                }
        }
        map
    })
}

#[function_component(SkillLineComponent)]
pub fn skill_line_index() -> Html {
    let ability_names = get_abilities();

    let groups: Vec<_> = get_groups()
        .iter()
        .filter(|(t, _)| {
            !SkillLine::from_id(t)
                .map(|s| s.is_vengeance())
                .unwrap_or(false)
        })
        .collect();

    let monthly_groups: Vec<((i32, u32), Vec<u32>)> = {
        let mut month_map: HashMap<(i32, u32), Vec<u32>> = HashMap::new();
        for (ts, ids) in get_timestamps().iter() {
            let dt = chrono::DateTime::from_timestamp(*ts as i64, 0)
                .unwrap_or_default()
                .with_timezone(&chrono::Utc);
            let key = (dt.year(), dt.month());
            month_map.entry(key).or_default().extend(ids);
        }
        let mut months: Vec<((i32, u32), Vec<u32>)> = month_map.into_iter().collect();
        months.sort_by(|(a, _), (b, _)| b.cmp(a));
        months.truncate(6);
        for (_, ids) in &mut months {
            ids.sort();
        }
        months
    };

    html! {
        <div>
            <h2 style="margin-top: 3em; text-align: center;">{"Player Skill Lines"}</h2>
            <div style="columns: 10rem; column-gap: 1rem;">
                { for groups.iter().map(|(sl, _ids)| html! {
                    <div key={**sl} style="break-inside: avoid; padding-top: 1rem;">
                        <h4 style="margin-bottom: 0em; margin-top: 0em;">
                            <Link<Route> to={Route::SkillLine { id: **sl }}>
                                { format!("{}", SkillLine::from_id(sl).unwrap_or(SkillLine::Emperor),) }
                            </Link<Route>>
                        </h4>
                    </div>
                })}
            </div>
            <h2 style="margin-top: 3em; text-align: center;">{"Player Skills by Recently Edited"}</h2>
            <div style="display: flex; flex-flow: row wrap; justify-content: space-between;">
                { for monthly_groups.iter().map(|((year, month), ids)| html! {
                    <div key={format!("{}-{}", year, month)}>
                        <h4 style="margin-bottom:0.25em;">
                            { format!("{} {}", chrono::Month::try_from(*month as u8).map(|m| m.name()).unwrap_or("?"), year) }
                        </h4>
                        { for ids.iter()
                            .filter(|i| !ability_names.get(i).map_or("?", |f| f.as_str()).contains("Vengeance"))
                            .map(|id| html! {
                                <div style="font-size: 0.9em; margin: 1px;">
                                    {
                                        render_ability_link(id, ability_names.get(id).unwrap_or(&"???".to_string()).to_string())
                                    }
                                    <br />
                                </div>
                            })
                        }
                    </div>
                })}
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct SearchProps {
    pub query: String,
}


#[function_component(Search)]
pub fn search(props: &SearchProps) -> Html {
    let ability_names = get_abilities();

    let query = use_state(|| props.query.clone());

    {
        let query = query.clone();
        let route_query = props.query.clone();

        use_effect_with(route_query, move |q| {
            query.set(q.clone());
            if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                if q.is_empty() {
                    document.set_title("Search - ESO ID Dictionary");
                } else {
                    document.set_title(format!("'{}' Search - ESO ID Dictionary", q.clone()).as_str());
                }
            }
            || ()
        });
    }

    let navigator = use_navigator().unwrap();

    let on_input = {
        let query = query.clone();
        let navigator = navigator.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<HtmlInputElement>();
            let value = input.value();

            query.set(value.clone());

            if value.trim().is_empty() {
                navigator.replace(&Route::Search);
            } else {
                navigator.replace(&Route::SearchQuery {
                    query: value,
                });
            }
        })
    };

    let filtered: Vec<_> = {
        let q = query.to_lowercase();
        if q.is_empty() {
            let mut v: Vec<_> = ability_names.iter().collect();
            v.sort_by_key(|(id, _)| *id);
            v.into_iter().take(50).collect()
        } else if q.trim().len() <= 5 {
            let mut v: Vec<_> = ability_names.iter()
                .filter(|(id, name)| {
                    name.to_lowercase().contains(&q) || id.to_string().contains(&q)
                })
                .collect();
            v.sort_by_key(|(id, _)| *id);
            v.into_iter().take(50).collect()
        } else {
            let mut v: Vec<_> = ability_names.iter()
                .filter(|(id, name)| {
                    name.to_lowercase().contains(&q) || id.to_string().contains(&q)
                })
                .collect();
            v.sort_by_key(|(id, _)| *id);
            v
        }
    };

    html! {
        <div>
            <nav style="margin-bottom: 1em;">
                <Link<Route> to={Route::Home}>
                    {"ESO ID Dictionary"}
                </Link<Route>>
                <span>{ " / Search" }</span>
            </nav>
            <input
                type="text"
                placeholder="Search by name or ID"
                oninput={on_input}
                value={(*query).clone()}
            />
            <p>{ format!("Showing {} results", filtered.len()) }</p>

            {
                filtered.iter().map(|(id, name)|
                {
                    let summary = if let Some(skill) = get_skill(id) {
                        let parts: Vec<String> = [
                            AbilityType::from_id(&skill.base_data.ability_type)
                                .filter(|f| f.ne(&AbilityType::None)).map(|a| format!("{}", a)),
                            DamageType::from_id(&skill.u4[3])
                                .filter(|f| f.ne(&DamageType::Generic)).map(|d| format!("{}", d)),
                            SkillLine::from_id(&skill.base_data.skill_line_id)
                                .map(|sl| format!("{}", sl)),
                            (skill.base_data.duration != 0)
                                .then(|| format!("{}", format_duration(&skill.base_data.duration))),
                            (skill.base_data.range != 0)
                                .then(|| format_distance(&skill.base_data.range)),
                            (skill.base_data.angle != 0.0)
                                .then(|| format_angle(&skill.base_data.angle)),
                            (skill.base_data.value1 != 0)
                                .then(|| get_value_adjusted(&skill.base_data.value1).to_string()),
                        ]
                        .into_iter()
                        .flatten()
                        .collect();

                        format!("  {}", parts.join("  "))
                    } else {
                        String::new()
                    };

                    html! {
                        <>
                            { render_ability_link(id, format!("{} ({})", name, id)) }
                            <span> { summary } </span>
                            <br />
                        </>
                }}).collect::<Html>()
            }
        </div>
    }
}


#[derive(Properties, PartialEq)]
pub struct SkillLineProps {
    pub id: u32,
}

#[function_component(SkillLineSummary)]
pub fn skill_line(props: &SkillLineProps) -> Html {
    let skill_line_id = props.id.clone();
    let skill_line = SkillLine::from_id(&skill_line_id);
    if let Some(sl) = skill_line {
        let ability_names = get_abilities();
        let groups: Vec<u32> = get_groups()
            .get(&skill_line_id)
            .cloned()
            .unwrap_or_default();
        if let Some(document) = web_sys::window().and_then(|w| w.document()) {
            document.set_title(format!("{} - ESO ID Dictionary", sl.as_str()).as_str());
        }
        html!{
            <>
                <nav style="margin-bottom: 1em;">
                    <Link<Route> to={Route::Home}>
                        {"ESO ID Dictionary"}
                    </Link<Route>>
                    <span>{ format!(" / Skill Line / {}", sl.as_str()) }</span>
                </nav>
                <div>
                    { for groups.iter().map(|id| html! {
                        <div style="margin: 1px;">
                            {
                                render_ability_link(
                                    id,
                                    ability_names
                                        .get(id)
                                        .unwrap_or(&"???".to_string())
                                        .to_string(),
                                )
                            }
                            <br />
                        </div>
                    })}
                </div>
            </>
        }
    } else {
        html!{
            <>
                <h1>
                    { format!("Unknown skill line id: {}", skill_line_id) }
                </h1>
            </>
        }
    }
}