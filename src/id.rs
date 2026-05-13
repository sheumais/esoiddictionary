use std::{collections::HashMap, sync::OnceLock};
use chrono::DateTime;
use eso_skill_data::{SkillData34, data_enum::*};
use yew::prelude::*;
use crate::fetch::fetch_bytes;
use crate::index_state::{IndexState, find_entry};


const ABILITY_CSV: &str = include_str!("../static/ability_names.csv");
const TOOLTIP_CSV: &str = include_str!("../static/ability_tooltips.csv");
const DATA_URL:    &str = "static/data.bin";

static ABILITIES: OnceLock<HashMap<u32, String>> = OnceLock::new();
static TOOLTIPS:  OnceLock<HashMap<u32, Vec<String>>> = OnceLock::new();

pub fn get_abilities() -> &'static HashMap<u32, String> {
    ABILITIES.get_or_init(|| {
        ABILITY_CSV
            .lines()
            .filter_map(|line| {
                let parts = csv_split(line);
                let id: u32 = parts.first()?.trim().parse().ok()?;
                let name    = parts.last()?.trim().to_string();
                Some((id, name))
            })
            .collect()
    })
}

pub fn get_tooltips() -> &'static HashMap<u32, Vec<String>> {
    TOOLTIPS.get_or_init(|| {
        TOOLTIP_CSV
            .lines()
            .filter_map(|line| {
                let parts = csv_split(line);
                let id: u32 = parts.first()?.trim().parse().ok()?;
                let tooltip = parts[1..].iter().map(|s| s.trim_matches('"').to_string()).collect::<Vec<String>>();
                Some((id, tooltip))
            })
            .collect()
    })
}

fn csv_split(line: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut in_quotes = false;
    for (i, c) in line.char_indices() {
        match c {
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                parts.push(&line[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&line[start..]);
    parts
}

#[derive(Clone, PartialEq)]
enum FetchState<T: PartialEq> {
    Idle,
    Loading,
    Done(T),
    Failed(String),
}

#[derive(Properties, PartialEq)]
pub struct IdProps {
    pub id:    u32,
    pub index: IndexState,
}

fn format_duration(ms: &u32) -> String { 
    let hours = ms / 3_600_000;
    let mins  = (ms % 3_600_000) / 60_000;
    let secs  = (ms % 60_000) / 1_000;
    let millis = ms % 1_000;

    match (hours, mins, secs, millis) {
        (h, 0, 0, 0) if h > 0 => format!("{}h", h),
        (0, m, 0, 0) if m > 0 => format!("{}m", m),
        (0, 0, s, 0) if s > 0 => format!("{}s", s),
        (h, m, 0, 0) if h > 0 => format!("{}h {}m", h, m),
        (0, m, s, 0) if m > 0 => format!("{}m {}s", m, s),
        _ => format!("{}ms", ms),
    }
}

#[function_component(IdData)]
pub fn id_data(props: &IdProps) -> Html {
    let abilities = get_abilities();
    let tooltips = get_tooltips();
    let skill_state = use_state(|| FetchState::<SkillData34>::Idle);
    let id = props.id;

    use_effect_with((id, props.index.clone()), {
        let skill_state = skill_state.clone();
        move |(id, index)| {
            let id = *id;
            match index {
                IndexState::Loading => {
                    skill_state.set(FetchState::Loading);
                }
                IndexState::Failed(e) => {
                    skill_state.set(FetchState::Failed(format!("Index failed to load: {e}")));
                }
                IndexState::Ready(entries) => {
                    skill_state.set(FetchState::Loading);
                    let entries = entries.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        let result = async {
                            let entry = find_entry(&entries, id)
                                .ok_or_else(|| format!("No record found for ID {id}"))?;
                            let bytes = fetch_bytes(
                                DATA_URL,
                                Some((entry.start_offset, entry.end_offset)),
                            )
                            .await?;
                            SkillData34::from_bytes(&bytes).map_err(|e| e.to_string())
                        }
                        .await;

                        skill_state.set(match result {
                            Ok(skill) => FetchState::Done(skill),
                            Err(e) => FetchState::Failed(e),
                        });
                    });
                }
            }
            || ()
        }
    });

    let name_line = match abilities.get(&id) {
        Some(name) => html! { <h1>{ format!("{} ({})", name, id) }</h1> },
        None => html! { <p>{ "ID has no recorded name" }</p> },
    };

    let data_section = match &*skill_state {
        FetchState::Idle => html! {},
        FetchState::Loading => html! {
            <div>
                <span>{ "Fetching record…" }</span>
            </div>
        },
        FetchState::Failed(e) => html! {
            <div>
                <strong>{ "Error" }</strong>
                <p>{ e }</p>
            </div>
        },
        FetchState::Done(skill) => {
            let equation = {
                let c = &skill.coef;
                let h1 = c.type1 != 0 || c.coef1 != 0.0;
                let h2 = c.type2 != 0 || c.coef2 != 0.0;
                let h3 = c.type3 != 0 || c.coef3 != 0.0;
                let h4 = c.type4 != 0 || c.coef4 != 0.0;

                let is_weapon_spell = |t: u8| matches!(t, 25 | 35);
                let is_resource    = |t: u8| matches!(t, 4 | 29);

                let paired_term = |t1: u8, t2: u8, coef: f32| -> String {
                    match (t1, t2) {
                        // (25, 35) | (35, 25) => format!("{coef}×MaxPower"),
                        // (4, 29)  | (29, 4)  => format!("{coef}×MaxResource"),
                        _ => format!(
                            "{coef}×max({}, {})",
                            CoefficientType::from_id(&t1).unwrap().as_str(),
                            CoefficientType::from_id(&t2).unwrap().as_str(),
                        ),
                    }
                };

                let is_mirror = h1 && h2 && h3 && h4
                    && c.coef1 == c.coef3 && c.coef2 == c.coef4
                    && is_weapon_spell(c.type1) && is_weapon_spell(c.type3)
                    && is_resource(c.type2)     && is_resource(c.type4);

                if !h1 && !h2 && !h3 && !h4 {
                    None
                } else if is_mirror {
                    Some(format!("{} + {}", paired_term(c.type1, c.type3, c.coef1), paired_term(c.type2, c.type4, c.coef2)))
                } else if h1 && !h2 && h3 && !h4 {
                    Some(paired_term(c.type1, c.type3, c.coef1))
                } else {
                    let mut terms = vec![];
                    if h1 { terms.push(CoefficientType::from_id(&c.type1).unwrap().as_str()); }
                    if h2 { terms.push(CoefficientType::from_id(&c.type2).unwrap().as_str()); }
                    if h3 { terms.push(CoefficientType::from_id(&c.type3).unwrap().as_str()); }
                    if h4 { terms.push(CoefficientType::from_id(&c.type4).unwrap().as_str()); }
                    Some(terms.join(" + "))
                }
            };

            html! {
            <div>
                <Field label="Last Edited: " value={format!("{}", DateTime::from_timestamp(skill.base_data.date_time.into(), 0).unwrap())} />
                if let Some(mech) = Mechanic::from_id(&skill.mechanic) {
                    <Field label="Mechanic: " value={format!("{} ({})", mech, skill.mechanic.to_string())} />
                }
                if let Some(skill_line) = SkillLine::from_id(&skill.base_data.skill_line_id) {
                    <Field label="Skill Line: " value={format!("{} ({})", skill_line, skill.base_data.skill_line_id)} />
                } else if skill.base_data.skill_line_id != 0 {
                    <Field label="Skill Line: " value={format!("? ({})", skill.base_data.skill_line_id)} />
                }
                if skill.base_data.caused_by != 0 {
                    <div> 
                        <span>{"Caused By: "}</span>
                        <span>
                            <a href={format!("/esoiddictionary/{}", skill.base_data.caused_by)}>
                                {format!("{} ({})", skill.base_data.caused_by.to_string(), abilities.get(&skill.base_data.caused_by).unwrap_or(&"Unknown name".to_string()))}
                            </a>
                        </span>
                    </div>
                }
                if let Some(ability_type) = AbilityType::from_id(&skill.base_data.ability_type) && skill.base_data.ability_type != 0 {
                    <Field label="Ability Type: " value={format!("{} ({})", ability_type, skill.base_data.ability_type)} />
                } else if skill.base_data.ability_type != 0 {
                    <Field label="Ability Type: " value={format!("? ({})", skill.base_data.ability_type)} />
                }
                if let Some(damage_type) = DamageType::from_id(&skill.u4[3]) && skill.u4[3] != 1 {
                    <Field label="Damage Type: " value={format!("{} ({})", damage_type, skill.u4[3])} />
                }
                if skill.base_data.value1 != 0 {
                    <Field label="Value: " value={format!("{}", skill.base_data.value1.to_string())} />
                }
                if skill.base_data.value2 != 0 && skill.base_data.value2 != skill.base_data.value1 {
                    <Field label="Value 2: " value={format!("{}", skill.base_data.value2.to_string())} />
                }
                if skill.base_data.cast_time != 0 {
                    <Field label="Cast Time: " value={format!("{}", format_duration(&skill.base_data.cast_time) )} />
                }
                if skill.base_data.duration != 0 {
                    <Field label="Duration: " value={format!("{}", format_duration(&skill.base_data.duration) )} />
                }
                if skill.base_data.tick != 0 {
                    <Field label="Tick: " value={format!("{}", format_duration(&skill.base_data.tick) )} />
                }
                if skill.base_data.start_tick != 0 {
                    <Field label="Start Tick: " value={format!("{}", format_duration(&skill.base_data.start_tick.into()) )} />
                }
                if skill.base_data.range != 0 {
                    <Field label="Range: " value={format!("{}m", (skill.base_data.range / 100).to_string())} />
                }
                if skill.base_data.radius != 0 {
                    <Field label="Radius: " value={format!("{}m", (skill.base_data.radius / 100).to_string())} />
                }
                if let Some(mech) = Mechanic::from_id(&skill.mechanic) {
                    if skill.base_data.cost != 0 {
                        <Field label="Resource Cost: " value={format!("{} ({})", skill.base_data.cost.to_string(), mech)} />
                    }
                }
                // if skill.coef.coef1 != 0f32 {
                //     <Field label="Coef 1: " value={format!("{} ({})", skill.coef.coef1, match_coefficient_type(&skill.coef.type1).unwrap_or("Unknown".to_string()))} />
                // }
                // if skill.coef.coef2 != 0f32 {
                //     <Field label="Coef 2: " value={format!("{} ({})", skill.coef.coef2, match_coefficient_type(&skill.coef.type2).unwrap_or("Unknown".to_string()))} />
                // }
                // if skill.coef.coef3 != 0f32 {
                //     <Field label="Coef 3: " value={format!("{} ({})", skill.coef.coef3, match_coefficient_type(&skill.coef.type3).unwrap_or("Unknown".to_string()))} />
                // }
                // if skill.coef.coef4 != 0f32 {
                //     <Field label="Coef 4: " value={format!("{} ({})", skill.coef.coef4, match_coefficient_type(&skill.coef.type4).unwrap_or("Unknown".to_string()))} />
                // }
                if let Some(eq) = equation {
                    <Field label="Equation: " value={eq} />
                }
                if let Some(tooltip) = tooltips.get(&skill.ability_id1) {
                    <h4>{"Tooltip"}</h4>
                    for t in tooltip {
                        <div>{t.to_owned()}</div>
                    }
                    { for skill.tooltip_data.iter().flat_map(|td| {
                        td.tooltip_ids.iter().zip(td.tooltip_types.iter()).enumerate().map(|(i, (id, ty))| {
                            let tooltip_type = TooltipType::from_id(ty).unwrap();
                            let label: String = format!("{} ({}): ", i + 1, tooltip_type).into();

                            let is_ability = *id >= u8::MAX as u32;

                            html! {
                                <div>
                                    <span>{ label }</span>

                                    {
                                        if is_ability {
                                            let ability_name = abilities
                                                .get(id)
                                                .unwrap_or(&"???".to_string())
                                                .clone();

                                            html! {
                                                <a href={format!("/esoiddictionary/{}", id)}>
                                                    { format!("{} ({})", ability_name, id) }
                                                </a>
                                            }
                                        } else {
                                            let value = match tooltip_type {
                                                TooltipType::BuffExplanationText => {
                                                    MajorMinorBuff::from_id(id)
                                                        .map(|buff| buff.tooltip_value().to_string())
                                                        .unwrap_or_else(|| {
                                                            format!("Missing Tooltip Major/Minor Buff ({})", id)
                                                        })
                                                },
                                                _ => {
                                                    MajorMinorBuff::from_id(id)
                                                        .map(|buff| buff.to_string())
                                                        .unwrap_or_else(|| {
                                                            format!("Missing Tooltip Major/Minor Buff ({})", id)
                                                        })
                                                },
                                            };

                                            html! {
                                                <span>{ value }</span>
                                            }
                                        }
                                    }
                                </div>
                            }
                        })
                    }) }
                }
                if !skill.causes_ids.is_empty() {
                    <h4>{ "Causes IDs" }</h4>
                    <div>
                        { for skill.causes_ids.iter().map(|cid: &u32| html! {
                            <a href={format!("/esoiddictionary/{cid}")}>
                                { format!("{} ({})", cid.to_string(), abilities.get(cid).unwrap_or(&"?".to_string())) }
                            <br />
                            </a>
                        })}
                    </div>
                }
                <div style="overflow: wrap;">
                    <h4>{"Raw JSON Data"}</h4>
                    <div>
                        {serde_json::to_string_pretty(skill).unwrap_or_default()}
                    </div>
                </div>
            </div>
            }
        },
    };

    html! {
        <div>
            <nav>
                <a href="/esoiddictionary/">{ "ESO ID Dictionary" }</a>
                <span>{ " / " }</span>
                <span>{ id.to_string() }</span>
            </nav>

            <header>
                { name_line }
            </header>

            { data_section }
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct FieldProps {
    label: &'static str,
    value: String,
}

#[function_component(Field)]
fn field(props: &FieldProps) -> Html {
    html! {
        <div>
            <span>{ props.label }</span>
            <span>{ &props.value }</span>
        </div>
    }
}