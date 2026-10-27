use eso_skill_data::{SkillData34, enums::{ability_type::AbilityType, damage_type::DamageType, tooltip_type::TooltipType}};
use yew::{Html, html};
use yew_router::components::Link;

use crate::{Route::{self, Ability}, fetch::get_skill, id::get_icons};
pub use eso_skill_data::display::{format_angle, format_distance, format_duration};

pub fn with_skill<F>(id: &u32, ability_name: &str, mut f: F) -> String
where
    F: FnMut(&SkillData34) -> Option<String>,
{
    match get_skill(id) {
        Some(skill) => f(&skill).unwrap_or_else(|| fallback(ability_name, id)),
        None => fallback(ability_name, id),
    }
}

pub fn fallback(ability_name: &str, id: &u32) -> String {
    format!("{} ({})", ability_name, id)
}

pub fn format_ability_summary(id: &u32) -> Html {
    match get_skill(id) {
        Some(skill) => {
            let parts: Vec<String> = [
                AbilityType::from_id(&skill.base_data.ability_type)
                    .filter(|f| f.ne(&AbilityType::None)).map(|a| format!("{}", a)),
                skill.damage_type()
                    .filter(|f| f.ne(&DamageType::Generic)).map(|d| format!("{}", d)),
                skill.skill_line()
                    .map(|sl| format!("{}", sl)),
                (skill.base_data.duration != 0)
                    .then(|| format!("{}", format_duration(&skill.base_data.duration))),
                (skill.base_data.range != 0)
                    .then(|| format_distance(&skill.base_data.range)),
                (skill.base_data.angle != 0.0)
                    .then(|| format_angle(&skill.base_data.angle)),
                (skill.base_data.value1 != 0)
                    .then(|| skill.base_data.value1.to_string()),
            ]
            .into_iter()
            .flatten()
            .collect();

            html! {
                <>
                    { render_ability_icon(&skill, 20) }
                    <span>{ format!(" {}", parts.join(" ")) }</span>
                </>
            }
        }
        None => html! {},
    }
}

pub fn render_ability_with_summary(id: &u32, name: &str) -> Html {
    let summary = format_ability_summary(id);
    html! {
        <>
            { render_ability_link(id, format!("{} ({})", name, id)) }
            <span> { summary } </span>
            <br />
        </>
    }
}

pub fn render_ability_link(id: &u32, display: String) -> Html {
    html! {
        <span>
            <Link<Route> to={Ability { id: *id }}>
                { display }
            </Link<Route>>
        </span>
    }
}

pub fn render_ability_link_current(id: &u32, display: String, is_current: bool) -> Html {
    if is_current {
        html! { <span>{ display }</span> }
    } else {
        html! {
            <Link<Route> to={Ability { id: *id }}>
                { display }
            </Link<Route>>
        }
    }
}

fn tooltip_value_present(skill: &SkillData34, tooltip_type: TooltipType) -> bool {
    skill.has_tooltip_value(tooltip_type)
}

pub fn resolve_id(initial_id: u32, tooltip_type: TooltipType, get_skill: &impl Fn(&u32) -> Option<SkillData34>) -> u32 {
    let mut current = initial_id;
    for _ in 0..=3 {
        let Some(s) = get_skill(&current) else { break };
        if tooltip_value_present(&s, tooltip_type) {
            break;
        }
        if s.causes_ids.len() == 1 {
            current = s.causes_ids[0];
        } else if let Some(t) = s.tooltip_data.first() {
            if t.num_tooltip_ids == 1 {
                current = t.tooltip_ids.first().copied().unwrap_or(initial_id);
            } else if s.u15.u34c_ability_id > 0 {
                current = s.u15.u34c_ability_id;
            } else {
                break;
            }
        } else {
            break;
        }
    }
    current
}

pub fn render_ability_reference(label: &'static str, id: u32, ability_name: &str) -> Html {
    html! {
        <div>
            <span>{label}</span>
            <span>
                {render_ability_link(
                    &id,
                    format!(
                        "{} ({})",
                        id,
                        ability_name
                    ),
                )}
            </span>
        </div>
    }
}

const ICON_DIR: &str = "static/icons";

pub fn render_ability_icon(skill: &SkillData34, size: u32) -> Html {
    match get_icons().get(&(skill.icon_file_index as u32)) {
        Some(icon) => html! {
            <img
                src={format!("{}/{}", ICON_DIR, icon)}
                width={size.to_string()}
                height={size.to_string()}
                loading="lazy"
                alt=""
                style="vertical-align: middle; margin-left: 0.4em; margin-right: 0.4em;"
            />
        },
        None => html! {
            <img
                src={format!("{}/{}", ICON_DIR, "icon_missing.png")}
                width={size.to_string()}
                height={size.to_string()}
                loading="lazy"
                alt=""
                style="vertical-align: middle; margin-left: 0.4em; margin-right: 0.4em;"
            />
        },
    }
}

pub fn render_ability_icon_by_id(id: &u32, size: u32) -> Html {
    match get_skill(id) {
        Some(skill) => render_ability_icon(&skill, size),
        None => html! {},
    }
}