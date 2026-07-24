use eso_skill_data::{SkillCoef, SkillData34, enums::{coefficient_type::CoefficientType, major_minor::MajorMinorBuff, tooltip_type::TooltipType}};
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

pub fn format_distance(cm: &u32) -> String {
    format!("{}m", (*cm as f32 / 100.0))
}

pub fn format_angle(angle: &f32) -> String {
    format!("{}°", angle)
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

fn tooltip_value_present(skill: &SkillData34, tooltip_type: TooltipType) -> bool {
    match tooltip_type {
        TooltipType::Percentage => {
            skill.base_data.value1 != 0
                || MajorMinorBuff::from_id(&(skill.major_minor_id as u32)).is_some()
        }
        TooltipType::StatPercentage | TooltipType::ReduceHeatPercent => {
            skill.base_data.value1 != 0
        }
        TooltipType::Duration | TooltipType::DelayedStrike | TooltipType::DeprecatedZeroDuration => {
            skill.base_data.duration != 0
        }
        TooltipType::MinimumCooldown => skill.base_data.value0 != 0,
        TooltipType::IncreaseDurationOf
        | TooltipType::Knockback
        | TooltipType::SelfHeal
        | TooltipType::ReduceCostIncreaseRecovery => skill.base_data.value1 != 0,
        TooltipType::TickRate => skill.base_data.tick != 0,
        TooltipType::MagicalDamage
        | TooltipType::MartialDamage
        | TooltipType::SingleTargetDoT
        | TooltipType::AreaHoT
        | TooltipType::SingleTargetHeal
        | TooltipType::NoblesConquest
        | TooltipType::DeprecatedMultiHit => SkillEquationFormatter::format(skill).is_some(),
        TooltipType::ResourceGain => {
            MajorMinorBuff::from_id(&(skill.major_minor_id as u32)).is_some()
                || skill.base_data.value1 != 0
                || SkillEquationFormatter::format(skill).is_some()
        }
        TooltipType::BonusUpToPercent => {
            skill.list19.first().map_or(0, |i| i.bonus_up_to_pct) != 0
        }
        TooltipType::ThresholdBelowHealthPercent => {
            skill.list19.first().map_or(0, |i| i.threshold_below_health_pct) != 0
        }
        _ => false,
    }
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

pub fn list26_u2_value(skill: &SkillData34, index: usize) -> Option<u32> {
    skill
        .list26
        .first()
        .and_then(|l26| l26.u2.get(index))
        .copied()
        .filter(|v| *v != 0)
}