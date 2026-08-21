use std::{collections::HashMap, sync::OnceLock};
use chrono::DateTime;
use eso_skill_data::enums::ability_tag::AbilityTag;
use eso_skill_data::enums::ability_type::AbilityType;
use eso_skill_data::enums::damage_type::DamageType;
use eso_skill_data::enums::flags::*;
use eso_skill_data::enums::major_minor::MajorMinorBuff;
use eso_skill_data::enums::mechanic::Mechanic;
use eso_skill_data::enums::skill_line::SkillLine;
use eso_skill_data::enums::tooltip_type::TooltipType;
use eso_skill_data::SkillData34;
use yew::prelude::*;
use yew_router::components::Link;
use crate::Route::{self, FlagsCompare};
use crate::fetch::{get_skill, read_bytes};
use crate::format::{SkillEquationFormatter, format_angle, format_distance, format_duration, list26_u2_value, render_ability_link, render_ability_link_current, render_ability_reference, resolve_id, with_skill};
use crate::index_state::{IndexState, find_entry};


const ABILITY_CSV: &str = include_str!("../static/ability_names.csv");
const TOOLTIP_CSV: &str = include_str!("../static/ability_tooltips.csv");

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
            let equation = SkillEquationFormatter::format(skill);

            let mut tags: Vec<String> = Vec::new();
            for id in &skill.ability_tags {
                let s = match AbilityTag::from_id(id) {
                    Some(i) => {format!("{} ({})", i.as_str(), id)},
                    None => {format!("{}", id)},
                };
                tags.push(s);
            }

            let tags = html! { <Field label={"Ability Tags: "} value={ tags.join(", ") } /> };

            fn render_value_field(label: &'static str, val: i32, prev: i32) -> Html {
                if val != 0 && val != prev {
                    html! { <Field label={label} value={val.to_string()} /> }
                } else {
                    html! {}
                }
            }

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
                    } else if let Some(weapon_skill_line) = match &skill.u18[8] { // 267785
                        1 => Some("One Hand and Shield"),
                        2 => Some("Dual Wield"),
                        3 => Some("Two Handed"),
                        4 => Some("Bow"),
                        5 => Some("Destruction Staff (General)"),
                        6 => Some("Restoration Staff"),
                        7 => Some("Destruction Staff (Fire)"),
                        8 => Some("Destruction Staff (Frost)"),
                        9 => Some("Destruction Staff (Lightning)"),
                        12 => Some("Werewolf"),
                        _ => None,
                    } {
                        <Field label="Skill Line 2: " value={format!("{}", weapon_skill_line)} />
                    }
                    if let Some(scribing_ability) = match skill.base_data.scribing_index {
                        0 => None,
                        1 => Some("Vault"),
                        2 => Some("Wield Soul"),
                        3 => Some("Shield Throw"),
                        4 => Some("Smash"),
                        5 => Some("Elemental Explosion"),
                        6 => Some("Mender's Bond"),
                        7 => Some("Travelling Knife"),
                        8 => Some("Soul Burst"),
                        9 => Some("Ulfsild's Contingency"),
                        10 => Some("Torchbearer"),
                        11 => Some("Trample"),
                        12 => Some("Banner Bearer"),
                        _ => Some("Unknown Scribing Ability")
                    } {
                        <Field label="Scribing: " value={format!("{} ({})", scribing_ability, skill.base_data.scribing_index)} />
                    }
                    if skill.base_data.caused_by != 0 && skill.base_data.caused_by != skill.ability_id1 {
                        {render_ability_reference(
                            "Caused By: ",
                            skill.base_data.caused_by,
                            abilities
                                .get(&skill.base_data.caused_by)
                                .map(String::as_str)
                                .unwrap_or("Unknown Ability"),
                        )}
                    }
                    if let Some(list25) = skill.list25.first() {
                        if list25.base_ability_id > 0 && list25.base_ability_id != skill.ability_id1 && skill.base_data.scribing_index == 0 {
                            <div>
                                <span>{"Base ability: "}</span>
                                <span>
                                    {
                                        render_ability_link(&list25.base_ability_id, format!(
                                            "{} ({})",
                                            list25.base_ability_id,
                                            abilities
                                                .get(&list25.base_ability_id)
                                                .unwrap_or(&"Unknown Ability".to_string())))
                                    }
                                </span>
                            </div>
                        }
                    }
                    if skill.u8c != 0 && skill.u8c != skill.ability_id1 {
                        {render_ability_reference(
                            "Replaces: ",
                            skill.u8c,
                            abilities
                                .get(&skill.u8c)
                                .map(String::as_str)
                                .unwrap_or("Unknown Ability"),
                        )}
                    }
                    if let Some(ability_type) = AbilityType::from_id(&skill.base_data.ability_type) && skill.base_data.ability_type != 0 {
                        <Field label="Ability Type: " value={format!("{} ({})", ability_type, skill.base_data.ability_type)} />
                        {
                            match ability_type {
                                AbilityType::ActionList => list26_u2_value(skill, 0)
                                    .map(|id| {
                                        html! {
                                            <div>
                                                <span>{"Do action list #: "}</span>
                                                <span>{format!("{id} (server-side function reference id)")}</span>
                                            </div>
                                        }
                                    })
                                    .unwrap_or_else(|| html! {}),

                                AbilityType::GrantAbility => list26_u2_value(skill, 1)
                                    .map(|id| {
                                        render_ability_reference(
                                            "Grants Synergy: ",
                                            id,
                                            abilities
                                                .get(&id)
                                                .map(String::as_str)
                                                .unwrap_or("Unknown Ability"),
                                        )
                                    })
                                    .unwrap_or_else(|| html! {}),

                                AbilityType::CreateInventoryItem => list26_u2_value(skill, 1)
                                    .map(|id| {
                                        html! {
                                            <div>
                                                <span>{"Creates item: "}</span>
                                                <span>
                                                    <a href={format!("https://esoitem.uesp.net/itemLink.php?itemid={id}&summary")}>
                                                        {id}
                                                    </a>
                                                </span>
                                            </div>
                                        }
                                    })
                                    .unwrap_or_else(|| html! {}),

                                AbilityType::AreaTeleport => list26_u2_value(skill, 1)
                                    .map(|id| {
                                        html! {
                                            <div>
                                                <span>{"Teleports players to location id: "}</span>
                                                <span>{format!("{id}")}</span>
                                            </div>
                                        }
                                    })
                                    .unwrap_or_else(|| html! {}),

                                AbilityType::SetCooldown | AbilityType::ModifyCooldown => list26_u2_value(skill, 1)
                                    .map(|id| {
                                        render_ability_reference(
                                            "Sets cooldown of: ",
                                            id,
                                            abilities
                                                .get(&id)
                                                .map(String::as_str)
                                                .unwrap_or("Unknown Ability"),
                                        )
                                    })
                                    .unwrap_or_else(|| html! {}),

                                AbilityType::RemoveType => list26_u2_value(skill, 1)
                                    .map(|id| {
                                        render_ability_reference(
                                            "Removes the buff: ",
                                            id,
                                            abilities
                                                .get(&id)
                                                .map(String::as_str)
                                                .unwrap_or("Unknown Ability"),
                                        )
                                    })
                                    .unwrap_or_else(|| html! {}),

                                _ => html! {},
                            }
                        }
                    }
                    if let Some(damage_type) = DamageType::from_id(&skill.u4[3]) {
                        <Field label="Damage Type: " value={format!("{} ({})", damage_type, skill.u4[3])} />
                    }
                    <>
                        {render_value_field("Value 0: ", skill.base_data.value0, 0)}
                        {render_value_field("Value 1: ", skill.base_data.value1, skill.base_data.value0)}
                        {render_value_field("Value 2: ", skill.base_data.value2, skill.base_data.value1)}
                        if let Some(major_minor_buff) = MajorMinorBuff::from_id(&(skill.major_minor_id as u32)) { // 103564
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
                    if let Some(mech) = Mechanic::from_id(&skill.mechanic) {
                        if skill.base_data.cost != 0 {
                            <Field label="Resource Cost: " value={format!("{} ({})", skill.base_data.cost.to_string(), mech)} />
                        }
                    }
                    if skill.flags[FLAG_TOGGLED] == 1 {
                        <Field label="Toggled: " value={"True"} />
                    }
                    if skill.flags[FLAG_COST_PER_TICK] == 1 {
                        <Field label="Cost drained multiple times over activation: " value={"True"} />
                    }
                    if !skill.ability_tags.is_empty() {
                        { tags }
                    }
                    if let Some(eq) = equation {
                        <Field label="Equation: " value={eq} />
                    }
                    if skill.size19 == 1 { // always 1 or 0, guaranteed by debug_assert in SkillData34 struct
                        if let Some(entry) = skill.list19.first() {
                            if entry.threshold_below_health_pct > 0 {
                                <Field label="Increase equation below health: " value={format!("{}%", entry.threshold_below_health_pct.to_string())} />
                            }
                            if entry.bonus_up_to_pct > 0 {
                                <Field label="Increase equation by up to: " value={format!("{}%", entry.bonus_up_to_pct.to_string())} />
                            }
                        }
                    }
                    if let Some(list26) = skill.list26.first() {
                        if let Some(i) = list26.u2.first() {
                            <div>{
                                match i {
                                    1  => "Has effect when cast",
                                    4  => "Affects magicka/stamina resource pool",
                                    7  => "Affects duration of eaten food",
                                    8  => "Affects ultimate pool",
                                    9  => "Has effect when you heal yourself or a group member/ally",
                                    14 => "Has effect when your attack is blocked",
                                    15 => "Has effect when you successfully roll dodge (mixed category, not guaranteed)",
                                    18 => "Has effect when you block",
                                    39 => "Has effect when you cast a specific kind of ability",
                                    49 => "Has effect when you deal critical damage",
                                    50 => "Has effect when you take critical damage",
                                    76 => "Tel Var District Bonus",
                                    78 => "Has effect when you bash (or blade of woe?) an enemy",
                                    86 => "Has effect every second in combat",
                                    88 => "Increases your ability range",
                                    92 => "Reduces chance for enchantment proc to reduce charge",
                                    93 => "Reduces wayshrine recall cost",
                                    96 => "Reduces the time it takes for a fish to bite",
                                    98 => "Reduces the chance to consume a potion",
                                    _  => "",
                                }
                            }</div>
                        }
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

                                                TooltipType::Percentage => {
                                                    let display = with_skill(id, &ability_name, |skill| {
                                                        let value = skill.base_data.value1;
                                                        if let Some(d) = SkillEquationFormatter::format(skill) {
                                                            return Some(d);
                                                        } else if value != 0 {
                                                            return Some(format!("{}%", value));
                                                        } else {
                                                            MajorMinorBuff::from_id(&(skill.major_minor_id as u32))
                                                                .map(|b| format!("{}%", b.tooltip_value()))
                                                        }

                                                    });

                                                    render_ability_link_current(id, display, is_current)
                                                }

                                                TooltipType::StatPercentage => {
                                                    let display = with_skill(id, &ability_name, |skill| {
                                                        let value = skill.base_data.value1;
                                                        (value != 0).then(|| format!("{}%", value))
                                                    });

                                                    render_ability_link_current(id, display, is_current)
                                                }

                                                TooltipType::ReduceHeatPercent => { // 79865
                                                    let display = with_skill(id, &ability_name, |skill| {
                                                        let value = skill.base_data.value1;
                                                        (value != 0).then(|| format!("{}%", value / 10))
                                                    });

                                                    render_ability_link_current(id, display, is_current)
                                                }

                                                                                                    // 47374
                                                TooltipType::Duration | TooltipType::DelayedStrike | TooltipType::DeprecatedZeroDuration => {
                                                    let display = with_skill(id, &ability_name, |skill| {
                                                        let value = skill.base_data.duration;
                                                        (value != 0).then(|| format_duration(&value))
                                                    });

                                                    render_ability_link_current(id, display, is_current)
                                                }

                                                TooltipType::MinimumCooldown => {
                                                    let display = with_skill(id, &ability_name, |skill| {
                                                        let value = skill.base_data.value0;
                                                        (value != 0).then(|| format_duration(&(value as u32)))
                                                    });

                                                    render_ability_link_current(id, display, is_current)
                                                }

                                                TooltipType::IncreaseDurationOf => { // 45214
                                                    let display = with_skill(id, &ability_name, |skill| {
                                                        let value = skill.base_data.value1;
                                                        (value != 0).then(|| format_duration(&(value as u32)))
                                                    });

                                                    render_ability_link_current(id, display, is_current)
                                                }

                                                //TooltipType:: IncreasedDurationVsMonsters => { // 107202, 39076
                                                //
                                                //}

                                                TooltipType::Knockback | TooltipType::SelfHeal => {
                                                    let display = with_skill(id, &ability_name, |skill| {
                                                        let value = skill.base_data.value1;
                                                        (value != 0).then(|| format_distance(&(value as u32)))
                                                    });

                                                    render_ability_link_current(id, display, is_current)
                                                }

                                                TooltipType::ReduceCostIncreaseRecovery => { // 132401
                                                    let display = with_skill(id, &ability_name, |skill| {
                                                        let value = skill.base_data.value1;
                                                        (value != 0).then(|| format!("{}", value))
                                                    });

                                                    render_ability_link_current(id, display, is_current)
                                                }

                                                TooltipType::TickRate => {
                                                    let display = with_skill(id, &ability_name, |skill| {
                                                        let value = skill.base_data.tick;
                                                        (value != 0).then(|| format_duration(&value))
                                                    });

                                                    render_ability_link_current(id, display, is_current)
                                                }

                                                TooltipType::MagicalDamage
                                                | TooltipType::MartialDamage
                                                | TooltipType::SingleTargetDoT
                                                | TooltipType::AreaHoT
                                                | TooltipType::SingleTargetHeal
                                                | TooltipType::NoblesConquest
                                                | TooltipType::DeprecatedMultiHit => {
                                                    let display = with_skill(id, &ability_name, |skill| {
                                                        if let Some(v) = SkillEquationFormatter::format(skill) {
                                                            Some(v)
                                                        } else {
                                                            let value = skill.base_data.value1;
                                                            (value != 0).then(|| format!("{}", value))
                                                        }
                                                    });

                                                    render_ability_link_current(id, display, is_current)
                                                }

                                                TooltipType::ResourceGain => {
                                                    let display = with_skill(id, &ability_name, |skill| {
                                                        if let Some(major_minor_buff) = MajorMinorBuff::from_id(&(skill.major_minor_id as u32)) {
                                                            Some(format!("{}", major_minor_buff.tooltip_value()))
                                                        } else if let Some(d) = SkillEquationFormatter::format(skill) {
                                                            Some(d)
                                                        } else {
                                                            Some(skill.base_data.value1.to_string())
                                                        }
                                                    });

                                                    render_ability_link_current(id, display, is_current)
                                                }

                                                TooltipType::BonusUpToPercent => {
                                                    let display = with_skill(id, &ability_name, |skill| {
                                                        let value = if let Some(i) = skill.list19.first(){i.bonus_up_to_pct} else {0};
                                                        (value != 0).then(|| format!("{}%", &value))
                                                    });

                                                    render_ability_link_current(id, display, is_current)
                                                }

                                                TooltipType::ThresholdBelowHealthPercent => {
                                                    let display = with_skill(id, &ability_name, |skill| {
                                                        let value = if let Some(i) = skill.list19.first(){i.threshold_below_health_pct} else {0};
                                                        (value != 0).then(|| format!("{}%", &value))
                                                    });

                                                    render_ability_link_current(id, display, is_current)
                                                }

                                                TooltipType::TargetCap => {
                                                    let display = with_skill(id, &ability_name, |skill| {
                                                        let value = skill.pre.u0;
                                                        (value != 0).then(|| format!("{}", &value))
                                                    });

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
                            { for skill.causes_ids.iter().map(|cid: &u32| html! {
                                <>
                                    {
                                        render_ability_link(cid, format!("{} ({})", cid, abilities.get(cid).unwrap_or(&"?".to_string())))
                                    }
                                <br />
                                </>
                            })}
                        </div>
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