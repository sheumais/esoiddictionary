use eso_skill_data::{SkillData34, enums::coefficient_type::CoefficientType};
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

    fn render_coef(c: &eso_skill_data::SkillCoef) -> Option<String> {
        let h1 = c.type1 != 0 || c.coef1 != 0.0;
        let h2 = c.type2 != 0 || c.coef2 != 0.0;
        let h3 = c.type3 != 0 || c.coef3 != 0.0;
        let h4 = c.type4 != 0 || c.coef4 != 0.0;

        let is_mirror =
            h1 && h2 && h3 && h4
                && c.coef1 == c.coef3
                && c.coef2 == c.coef4
                && Self::is_weapon_spell(c.type1)
                && Self::is_weapon_spell(c.type3)
                && Self::is_resource(c.type2)
                && Self::is_resource(c.type4);

        if !h1 && !h2 && !h3 && !h4 {
            return None;
        }

        if is_mirror {
            return Some(format!(
                "{} + {}",
                Self::paired_term(c.type1, c.type3, c.coef1),
                Self::paired_term(c.type2, c.type4, c.coef2),
            ));
        }

        if h1 && !h2 && h3 && !h4 {
            return Some(Self::paired_term(c.type1, c.type3, c.coef1));
        }

        if h1 && !h2 && !h3 && !h4 {
            return Some(format!(
                "{}×{}",
                c.coef1,
                CoefficientType::from_id(&c.type1).unwrap().as_str()
            ));
        }

        let mut terms = Vec::new();
        if h1 { terms.push(CoefficientType::from_id(&c.type1).unwrap().as_str().to_string()); }
        if h2 { terms.push(CoefficientType::from_id(&c.type2).unwrap().as_str().to_string()); }
        if h3 { terms.push(CoefficientType::from_id(&c.type3).unwrap().as_str().to_string()); }
        if h4 { terms.push(CoefficientType::from_id(&c.type4).unwrap().as_str().to_string()); }

        Some(terms.join(" + "))
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
    format!("{}m", (cm / 100))
}

pub fn with_skill<F>(id: &u32, ability_name: &str, f: F) -> String
where
    F: FnOnce(&SkillData34) -> Option<String>,
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