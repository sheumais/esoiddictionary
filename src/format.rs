use eso_skill_data::{SkillCoef, SkillData34, enums::coefficient_type::CoefficientType};
use yew::{Html, html};
use yew_router::components::Link;

use crate::{Route::{self, Ability}, fetch::get_skill};

pub struct SkillEquationFormatter;

impl SkillEquationFormatter {
    fn is_weapon_spell(t: u8) -> bool {
        matches!(t, 25 | 35)
    }

    fn is_resource(t: u8) -> bool {
        matches!(t, 4 | 29)
    }

    fn paired_term(t1: u8, t2: u8, coef: f32) -> String {
        match (t1, t2) {
            (25, 35) | (35, 25) => format!("{coef}×MaxPower"),
            (4, 29) | (29, 4) => format!("{coef}×MaxResource"),
            _ => format!(
                "{coef}×max({}, {})",
                CoefficientType::from_id(&t1).unwrap().as_str(),
                CoefficientType::from_id(&t2).unwrap().as_str(),
            ),
        }
    }

    fn render_coef(c: &SkillCoef) -> Option<String> {
        let terms: Vec<(u8, f32)> = [
            (c.type1, c.coef1),
            (c.type2, c.coef2),
            (c.type3, c.coef3),
            (c.type4, c.coef4),
        ]
        .into_iter()
        .filter(|(ty, coef)| *ty > 0 && *coef > 0.0)
        .collect();

        if terms.is_empty() {
            return None;
        }

        let coef_str = |id, coef| {
            CoefficientType::from_id(&id)
                .map(|t| format!("{coef}×{}", t.as_str()))
                .unwrap_or_else(|| format!("{coef}×unknown({id})"))
        };

        if let [(t1, c1), (t2, c2), (t3, c3), (t4, c4)] = terms.as_slice() {
            if c1 == c3
                && c2 == c4
                && Self::is_weapon_spell(*t1)
                && Self::is_resource(*t2)
                && Self::is_weapon_spell(*t3)
                && Self::is_resource(*t4)
            {
                return Some(format!(
                    "{} + {}",
                    Self::paired_term(*t1, *t3, *c1),
                    Self::paired_term(*t2, *t4, *c2),
                ));
            }
        }

        if let [(t1, c1), (t3, c3)] = terms.as_slice() {
            if c1 == c3
                && Self::is_weapon_spell(*t1)
                && Self::is_weapon_spell(*t3)
            {
                return Some(Self::paired_term(*t1, *t3, *c1));
            }
        }

        Some(
            terms
                .into_iter()
                .map(|(id, coef)| coef_str(id, coef))
                .collect::<Vec<_>>()
                .join(" + "),
        )
    }

    pub fn format(skill: &SkillData34) -> Option<String> {
        let parts: Vec<String> = skill
            .coef
            .iter()
            .filter_map(Self::render_coef)
            .collect();

        if parts.is_empty() {
            None
        } else {
            Some(parts.join(" AND "))
        }
    }
}

pub fn format_duration(ms: &u32) -> String { 
    let hours = ms / 3_600_000;
    let mins  = (ms % 3_600_000) / 60_000;
    let secs  = (ms % 60_000) / 1_000;
    let millis = ms % 1_000;

    match (hours, mins, secs, millis) {
        (h, 0, 0, 0) if h > 0 => format!("{}h", h),
        (0, m, 0, 0) if m > 1 => format!("{}mins", m),
        (0, m, 0, 0) if m > 0 => format!("{}min", m),
        (0, 0, s, 0) if s > 0 => format!("{}s", s),
        (h, m, 0, 0) if h > 0 => format!("{}h {}m", h, m),
        (0, m, s, 0) if m > 0 => format!("{}m {}s", m, s),
        _ => format!("{}ms", ms),
    }
}

pub fn get_value_adjusted(value: &u32) -> u32 {
    if value > &2_147_483_647u32 {
        u32::MAX - value + 1
    } else {
        *value
    }
}

pub fn format_distance(cm: &u32) -> String {
    format!("{}m", (*cm as f32 / 100.0))
}

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