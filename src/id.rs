use std::{collections::HashMap, sync::OnceLock};
use chrono::DateTime;
use eso_skill_data::enums::major_minor::MajorMinorBuff;
use eso_skill_data::enums::tooltip_type::TooltipType;
use eso_skill_data::{SkillData34, TypeLink};
use yew::prelude::*;
use yew_router::components::Link;
use crate::Route::{self, FlagsCompare};
use crate::fetch::{get_skill, read_bytes};
use crate::format::{format_angle, format_distance, format_duration, render_ability_link, render_ability_link_current, render_ability_reference, resolve_id, with_skill};
use crate::index_state::{IndexState, find_entry};


const ABILITY_CSV: &str = include_str!("../static/ability_names.csv");
const TOOLTIP_CSV: &str = include_str!("../static/ability_tooltips.csv");
/// Based on u8b in the ability struct definition which points to a eso manifest file (https://en.uesp.net/wiki/ESO_Mod:MNF_File_Format) file entry of the icon
const ICONS_CSV: &str   = include_str!("../static/icons.csv");

static ABILITIES: OnceLock<HashMap<u32, String>> = OnceLock::new();
static TOOLTIPS:  OnceLock<HashMap<u32, Vec<String>>> = OnceLock::new();

pub fn get_abilities() -> &'static HashMap<u32, String> {
    ABILITIES.get_or_init(|| {
        ABILITY_CSV
            .lines()
            .filter_map(|line| {
                let parts = csv_split(line);
                let id: u32 = parts.first()?.trim().parse().ok()?;
                let name = parts.last()?.trim().to_string();
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

static ICONS: OnceLock<HashMap<u32, String>> = OnceLock::new();

pub fn get_icons() -> &'static HashMap<u32, String> {
    ICONS.get_or_init(|| {
        ICONS_CSV
            .lines()
            .filter_map(|line| {
                let (idx, name) = line.split_once(',')?;
                Some((idx.trim().parse().ok()?, name.trim().to_string()))
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

#[function_component(IdData)]
pub fn id_data(props: &IdProps) -> Html {
    let abilities = get_abilities();
    let tooltips = get_tooltips();
    let icons = get_icons();
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
                    skill_state.set(
                        FetchState::Failed(
                            format!("Index failed to load: {e}")
                        )
                    );
                }
                IndexState::Ready => {
                    skill_state.set(FetchState::Loading);
                    wasm_bindgen_futures::spawn_local(async move {
                        let result = async {
                            let entry = find_entry(id)
                                .ok_or_else(|| {
                                    format!("No record found for ID {id}")
                                })?;

                            let bytes = read_bytes(
                                Some((entry.start_offset, entry.end_offset))
                            )?;

                            SkillData34::from_bytes(bytes)
                                .map_err(|e| e.to_string())
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
        Some(name) => {
            if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                document.set_title(format!("{} ({}) - ESO ID Dictionary", name.trim_matches('"'), id).as_str());
            }
            html! { <h1>{ format!("{} ({})", name.trim_matches('"'), id) }</h1> }
        },
        None => html! { <p>{ "ID has no recorded name" }</p> },
    };

    let data_section = match &*skill_state {
        FetchState::Idle => html! {},
        FetchState::Loading => html! {
            <div>
                <span>{ "Fetching record…" }</span>
                <span>{ "If this takes a very long time (5+ seconds), please try refreshing the page."}</span>
                <p>{ "If it never loads, please send the link to the page you are currently on to me (@themrpancake at discord.gg/FjJjXHjUQ4)."}</p>
            </div>
        },
        FetchState::Failed(e) => html! {
            <div>
                <strong>{ "Error" }</strong>
                <p>{ e }</p>
            </div>
        },
        FetchState::Done(skill) => {
            let equation = skill.equation();

            let name_of = |id: u32| -> &'static str {
                abilities.get(&id).map(String::as_str).unwrap_or("Unknown Ability")
            };

            let mut tags: Vec<Html> = Vec::new();
            for (i, id) in skill.ability_tags.iter().enumerate() {
                if i > 0 {
                    tags.push(html! { {", "} });
                }
                let s = SkillData34::tag_label(*id);
                tags.push(html! {
                    <Link<Route> to={Route::Tag { index: id.to_string() }}>
                        { s }
                    </Link<Route>>
                });
            }

            let tags = html! {
                <div>
                    <span>{"Ability Tags: "}</span>
                    <span>{ for tags }</span>
                </div>
            };

            fn render_value_field(label: &'static str, val: i32, prev: i32) -> Html {
                if val != 0 && val != prev {
                    html! { <Field label={label} value={val.to_string()} /> }
                } else {
                    html! {}
                }
            }

            html! {
                <div>
                    {
                        match icons.get(&(skill.icon_file_index as u32)) {
                            Some(icon) => {
                                html!{
                                    <img
                                        src={format!("static/icons/{}", icon)}
                                        alt={format!("/esoui/art/icons/{}.dds", icon.trim_end_matches(".png").to_string())}
                                        width="64"
                                        height="64"
                                    />
                                }
                            },
                            None => {
                                html!{
                                    <img
                                        src={format!("static/icons/icon_missing.png")}
                                        alt={format!("/esoui/art/icons/icon_missing.dds")}
                                        width="64"
                                        height="64"
                                    />
                                }
                            }
                        }
                    }
                    <Field label="Last Edited: " value={format!("{}", DateTime::from_timestamp(skill.last_edited_timestamp(), 0).unwrap())} />
                    if let Some(mech) = skill.mechanic() {
                        <Field label="Mechanic: " value={format!("{} ({})", mech, skill.mechanic.to_string())} />
                    }
                    if let Some(skill_line) = skill.skill_line() {
                        <Field label="Skill Line: " value={format!("{} ({})", skill_line, skill.base_data.skill_line_id)} />
                    } else if skill.base_data.skill_line_id != 0 {
                        <Field label="Skill Line: " value={format!("? ({})", skill.base_data.skill_line_id)} />
                    }
                    if let Some(scribing_ability) = skill.scribing_ability_name() {
                        <Field label="Scribing: " value={format!("{} ({})", scribing_ability, skill.base_data.scribing_index)} />
                    }
                    if let Some(caused_by) = skill.caused_by_id() {
                        {render_ability_reference(
                            "Caused By: ",
                            caused_by,
                            abilities
                                .get(&caused_by)
                                .map(String::as_str)
                                .unwrap_or("Unknown Ability"),
                        )}
                    }
                    if let Some(base_id) = skill.base_ability_id() {
                        <div>
                            <span>{"Base ability: "}</span>
                            <span>
                                {
                                    render_ability_link(&base_id, format!(
                                        "{} ({})",
                                        base_id,
                                        abilities
                                            .get(&base_id)
                                            .unwrap_or(&"Unknown Ability".to_string())))
                                }
                            </span>
                        </div>
                    }
                    if let Some(replaces) = skill.replaces_id() {
                        {render_ability_reference(
                            "Replaces: ",
                            replaces,
                            abilities
                                .get(&replaces)
                                .map(String::as_str)
                                .unwrap_or("Unknown Ability"),
                        )}
                    }
                    if let Some(ability_type) = skill.ability_type() {
                        <Field label="Ability Type: " value={format!("{} ({})", ability_type, skill.base_data.ability_type)} />
                        {
                            match skill.type_link() {
                                Some(TypeLink::ActionList(id)) => html! {
                                    <div>
                                        <span>{"Do action list #: "}</span>
                                        <span>{format!("{id} (server-side function reference id)")}</span>
                                    </div>
                                },
                                Some(TypeLink::GrantsSynergy(id)) => render_ability_reference("Grants Synergy: ", id, name_of(id)),
                                Some(TypeLink::CreatesItem(id)) => html! {
                                    <div>
                                        <span>{"Creates item: "}</span>
                                        <span>
                                            <a href={format!("https://esoitem.uesp.net/itemLink.php?itemid={id}&summary")}>
                                                {id}
                                            </a>
                                        </span>
                                    </div>
                                },
                                Some(TypeLink::AreaTeleport(id)) => html! {
                                    <div>
                                        <span>{"Teleports players to location id: "}</span>
                                        <span>{format!("{id}")}</span>
                                    </div>
                                },
                                Some(TypeLink::SetsCooldown(id)) => render_ability_reference("Sets cooldown of: ", id, name_of(id)),
                                Some(TypeLink::RemovesBuff(id)) => render_ability_reference("Removes the buff: ", id, name_of(id)),
                                None => html! {},
                            }
                        }
                    }
                    <Field label="Target Type: " value={format!("{} ({})", skill.target_type_label(), skill.base_data.target_type)} />
                    if let Some(damage_type) = skill.damage_type() {
                        <Field label="Damage Type: " value={format!("{} ({})", damage_type, skill.damage_type_id())} />
                    }
                    <>
                        {render_value_field("Value 0: ", skill.base_data.value0, 0)}
                        {render_value_field("Value 1: ", skill.base_data.value1, skill.base_data.value0)}
                        {render_value_field("Value 2: ", skill.base_data.value2, skill.base_data.value1)}
                        if let Some(major_minor_buff) = skill.major_minor_buff() {
                            <Field label="Major/Minor buff: " value={format!("{} ({})", major_minor_buff.as_str(), major_minor_buff.to_id())} />
                        }
                    </>
                    if skill.base_data.cast_time != 0 {
                        <Field label="Cast Time: " value={format_duration(&skill.base_data.cast_time) } />
                    }
                    if skill.base_data.duration != 0 {
                        <Field label="Duration: " value={format_duration(&skill.base_data.duration) } />
                    }
                    if skill.base_data.tick != 0 {
                        <Field label="Tick: " value={format_duration(&skill.base_data.tick) } />
                    }
                    if skill.base_data.start_tick != 0 {
                        <Field label="Start Tick: " value={format_duration(&skill.base_data.start_tick.into())} />
                    }
                    if skill.base_data.range != 0 {
                        <Field label="Range: " value={format_distance(&skill.base_data.range) } />
                    }
                    if skill.base_data.radius != 0 {
                        <Field label="Radius: " value={format_distance(&skill.base_data.radius)} />
                    }
                    if skill.base_data.angle != 0.0 {
                        <Field label="Angle: " value={format_angle(&skill.base_data.angle)} />
                    }
                    if let Some(mech) = skill.mechanic() {
                        if skill.base_data.cost != 0 {
                            <Field label="Resource Cost: " value={format!("{} ({})", skill.base_data.cost.to_string(), mech)} />
                        }
                    }
                    if skill.is_toggled() {
                        <Field label="Toggled: " value={"True"} />
                    }
                    if skill.is_cost_per_tick() {
                        <Field label="Cost drained multiple times over activation: " value={"True"} />
                    }
                    if let Some(cc) = skill.cc_immunity_label() {
                        <Field label="CC Immunity Type: " value={cc} />
                    }
                    if !skill.ability_tags.is_empty() {
                        { tags }
                    }
                    if let Some(eq) = equation {
                        <Field label="Equation: " value={eq} />
                    }
                    if skill.threshold_below_health_pct() > 0 {
                        <Field label="Increase equation below health: " value={format!("{}%", skill.threshold_below_health_pct())} />
                    }
                    if skill.bonus_up_to_pct() > 0 {
                        <Field label="Increase equation by up to: " value={format!("{}%", skill.bonus_up_to_pct())} />
                    }
                    if let Some(label) = skill.list26_trigger_label() {
                        <div>{ label }</div>
                    }
                    if let Some(class) = skill.crime_label() {
                        <div style="color: red;">
                            { format!("Criminal {} ability", class) }
                        </div>
                    }
                    if skill.target_cap() != 0 {
                        <Field label="Target Cap: " value={skill.target_cap().to_string()} />
                    }
                    if let Some(tooltip) = tooltips.get(&skill.ability_id1) {
                        <h4>{"Tooltip"}</h4>
                        for t in tooltip {
                            <div>{t.trim_matches('"').to_owned()}</div>
                        }
                        { for skill.tooltip_data.iter().flat_map(|td| {
                            td.tooltip_ids.iter().zip(td.tooltip_types.iter()).enumerate().map(|(i, (id, ty))| {
                                let tooltip_type = TooltipType::from_id(ty).unwrap();
                                let id = &resolve_id(*id, tooltip_type, &get_skill);
                                let tooltip_type = TooltipType::from_id(ty).unwrap();
                                let label: String = format!("{} ({}): ", i + 1, tooltip_type).into();

                                let is_ability = *id >= u8::MAX as u32;
                                let is_current = *id == skill.ability_id1; // should remain immutable. if it needs to be mut, then struct understanding needs evolution
                                let ability_name = abilities
                                    .get(id)
                                    .unwrap_or(&"???".to_string())
                                    .clone();

                                html! {
                                    <div>
                                        <span>{ label }</span>

                                        {
                                            match tooltip_type {
                                                TooltipType::FlatConstant => {
                                                    html! { <span>{ id }</span> }
                                                }

                                                TooltipType::PercentageConstant => {
                                                    html! { <span>{ format!("{}%", id) }</span> }
                                                }

                                                t if SkillData34::is_value_tooltip(t) => {
                                                    let display = with_skill(id, &ability_name, |s| s.tooltip_value(t));

                                                    render_ability_link_current(id, display, is_current)
                                                }

                                                TooltipType::BuffGain => {
                                                    render_ability_link_current(id, ability_name, is_current)
                                                }


                                                _ => {
                                                    if is_ability {
                                                        let ability_name = abilities
                                                            .get(id)
                                                            .unwrap_or(&"???".to_string())
                                                            .clone();

                                                            render_ability_link_current(id, format!("{} ({})", ability_name, id), is_current)
                                                    } else {
                                                        let value = match tooltip_type {
                                                            TooltipType::BuffExplanationText => {
                                                                MajorMinorBuff::from_id(id)
                                                                    .map(|b| b.tooltip_value().to_string())
                                                                    .unwrap_or_else(|| {
                                                                        format!("Missing Tooltip Major/Minor Buff ({})", id)
                                                                    })
                                                            }
                                                            _ => {
                                                                MajorMinorBuff::from_id(id)
                                                                    .map(|b| b.to_string())
                                                                    .unwrap_or_else(|| {
                                                                        format!("Missing Tooltip Major/Minor Buff ({})", id)
                                                                    })
                                                            }
                                                        };

                                                        html! { 
                                                            <span>{ value }</span> 
                                                        }
                                                    }
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
                            { for skill.links().into_iter().map(|(cid, t, label)| {
                                let cid = &cid;
                                let name = abilities.get(cid).map(|s| s.as_str()).unwrap_or("?");
                                let text = match label {
                                    Some(l) => format!("{} ({}) {}", cid, name, l),
                                    None => format!("{} ({}) [{}]", cid, name, t),
                                };

                                html! {
                                    <>
                                        { render_ability_link(cid, text) }
                                        <br />
                                    </>
                                }
                            })}
                        </div>
                    } else {
                        { html! {} }
                    }
                    <div style="overflow: wrap;">
                        <h4>{"Raw JSON Data"}</h4>
                        <div>
                            {serde_json::to_string_pretty(skill).unwrap_or_default()}
                        </div>
                    </div>
                    <p>
                        <Link<Route> to={FlagsCompare { ids: skill.ability_id1.to_string() }}>
                            { "View ability flags" }
                        </Link<Route>>
                    </p>
                </div>
            }
        },
    };

    html! {
        <div>
            <nav>
                <Link<Route> to={Route::Home}>
                    {"ESO ID Dictionary"}
                </Link<Route>>
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