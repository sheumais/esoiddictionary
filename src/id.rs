use std::{collections::HashMap, sync::OnceLock};
use chrono::{DateTime, NaiveDateTime};
use eso_skill_data::{SkillData34, ability_type_name, match_coefficient_type, match_damage_type, match_mechanic, skill_line_name};
use yew::prelude::*;
use crate::fetch::fetch_bytes;
use crate::index_state::{IndexState, find_entry};

const ABILITY_CSV: &str  = include_str!("../static/ability_names.csv");
const DATA_URL:    &str  = "static/data.bin";
const FLAG_COST_DETERMINED_BY_MAX_RESOURCE: usize = 180;

static ABILITIES: OnceLock<HashMap<u32, String>> = OnceLock::new();

fn get_abilities() -> &'static HashMap<u32, String> {
    ABILITIES.get_or_init(|| {
        ABILITY_CSV
            .lines()
            .skip(1)
            .filter_map(|line| {
                let mut parts = line.splitn(2, ',');
                let id: u32 = parts.next()?.trim().parse().ok()?;
                let name    = parts.next()?.trim().to_string();
                Some((id, name))
            })
            .collect()
    })
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

#[function_component(IdData)]
pub fn id_data(props: &IdProps) -> Html {
    let abilities   = get_abilities();
    let skill_state = use_state(|| FetchState::<SkillData34>::Idle);
    let id          = props.id;

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
                            Err(e)    => FetchState::Failed(e),
                        });
                    });
                }
            }
            || ()
        }
    });

    let name_line = match abilities.get(&id) {
        Some(name) => html! { <h1>{ format!("{} ({})", name, id) }</h1> },
        None       => html! { <p>{ "ID has no recorded name" }</p> },
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
        FetchState::Done(skill) => html! {
            <div>
                <Field label="Last Edited: "     value={format!("{}", DateTime::from_timestamp(skill.base_data.date_time.into(), 0).unwrap())} />
                if let Some(mech) = match_mechanic(skill.mechanic) {
                    <Field label="Mechanic: "     value={format!("{} ({})", mech, skill.mechanic.to_string())} />
                }
                if let Some(skill_line) = skill_line_name(skill.base_data.skill_line_id) {
                    <Field label="Skill Line: "   value={format!("{} ({})", skill_line, skill.base_data.skill_line_id)} />
                } else if skill.base_data.skill_line_id != 0 {
                    <Field label="Skill Line: "   value={format!("? ({})", skill.base_data.skill_line_id)} />
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
                if let Some(ability_type) = ability_type_name(skill.base_data.ability_type) && skill.base_data.ability_type != 0 {
                    <Field label="Ability Type: "   value={format!("{} ({})", ability_type, skill.base_data.ability_type)} />
                } else if skill.base_data.ability_type != 0 {
                    <Field label="Ability Type: "   value={format!("? ({})", skill.base_data.ability_type)} />
                }
                if let Some(damage_type) = match_damage_type(skill.u4[3]) && skill.u4[3] != 1 {
                    <Field label="Damage Type: "   value={format!("{} ({})", damage_type, skill.u4[3])} />
                }
                if skill.base_data.value1 != 0 {
                    <Field label="Value 1: "    value={format!("{}", skill.base_data.value1.to_string())} />
                }
                if skill.base_data.value2 != 0 && skill.base_data.value2 != skill.base_data.value1 {
                    <Field label="Value 2: "    value={format!("{}", skill.base_data.value2.to_string())} />
                }
                if skill.base_data.cast_time != 0 {
                    <Field label="Cast Time: "    value={format!("{}ms", skill.base_data.cast_time.to_string())} />
                }
                if skill.base_data.duration != 0 {
                    <Field label="Duration: "     value={format!("{}ms", skill.base_data.duration.to_string())} />
                }
                if skill.base_data.tick != 0 {
                    <Field label="Tick: "         value={format!("{}ms", skill.base_data.tick.to_string())} />
                }
                if skill.base_data.start_tick != 0 {
                    <Field label="Start Tick: "   value={format!("{}ms", skill.base_data.start_tick.to_string())} />
                }
                if skill.base_data.range != 0 {
                    <Field label="Range: "        value={format!("{}m", (skill.base_data.range / 100).to_string())} />
                }
                if skill.base_data.radius != 0 {
                    <Field label="Radius: "       value={format!("{}m", (skill.base_data.radius / 100).to_string())} />
                }
                if let Some(mech) = match_mechanic(skill.mechanic) {
                    if skill.base_data.cost == 0 {
                        // intentionally blank
                    } else if skill.flags[FLAG_COST_DETERMINED_BY_MAX_RESOURCE] == 1 {
                        <Field label="Resource Cost: "         value={format!("{} ({})", skill.base_data.cost.to_string(), "Lower Max Resource")} />
                    } else {
                        <Field label="Resource Cost: "         value={format!("{} ({})", skill.base_data.cost.to_string(), mech)} />
                    }
                }
                if skill.coef.coef1 != 0f32 {
                    <Field label="Coef 1: " value={format!("{} ({})", skill.coef.coef1, match_coefficient_type(skill.coef.type1).unwrap_or("Unknown".to_string()))} />
                }
                if skill.coef.coef2 != 0f32 {
                    <Field label="Coef 2: " value={format!("{} ({})", skill.coef.coef2, match_coefficient_type(skill.coef.type2).unwrap_or("Unknown".to_string()))} />
                }
                if skill.coef.coef3 != 0f32 {
                    <Field label="Coef 3: " value={format!("{} ({})", skill.coef.coef3, match_coefficient_type(skill.coef.type3).unwrap_or("Unknown".to_string()))} />
                }
                if skill.coef.coef4 != 0f32 {
                    <Field label="Coef 4: " value={format!("{} ({})", skill.coef.coef4, match_coefficient_type(skill.coef.type4).unwrap_or("Unknown".to_string()))} />
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