use std::sync::OnceLock;

use eso_skill_data::enums::coefficient_type::CoefStat;
use eso_skill_data::enums::coefficient_type::CoefficientType;
use yew::prelude::*;
use yew_router::components::Link;

use crate::fetch::get_skill;
use crate::format::render_ability_with_summary;
use crate::id::get_abilities;
use crate::Route;
use crate::index_state::INDEX_MAP;

struct CoefEntry {
    id: u32,
    stat: CoefStat,
    value: f32,
}

fn is_weapon_damage_only(stat: CoefStat) -> bool {
    stat.single_type() == Some(CoefficientType::WeaponDamage) 
    || stat.single_type() == Some(CoefficientType::SpellDamage)
    || stat.single_type() == Some(CoefficientType::PhysicalResistance)
    || stat.single_type() == Some(CoefficientType::SpellResistance)
    || stat.single_type() == Some(CoefficientType::Power)
}

static RANKED_COEFS: OnceLock<Vec<CoefEntry>> = OnceLock::new();

fn ranked_coefs() -> &'static Vec<CoefEntry> {
    RANKED_COEFS.get_or_init(|| {
        let mut entries: Vec<CoefEntry> = Vec::new();

        let Some(index) = INDEX_MAP.get() else {
            return entries;
        };

        for &id in index.keys() {
            let Some(skill) = get_skill(&id) else {
                continue;
            };

            let mut seen: Vec<(CoefStat, u32)> = Vec::new();
            for block in &skill.coef {
                for term in block.combined_terms() {
                    if is_weapon_damage_only(term.stat) {
                        continue;
                    }
                    if term.value > 6.0 {
                        continue;
                    }
                    let key = (term.stat, term.value.to_bits());
                    if seen.contains(&key) {
                        continue;
                    }
                    seen.push(key);
                    entries.push(CoefEntry { id, stat: term.stat, value: term.value });
                }
            }
        }

        entries.sort_by(|a, b| {
            b.value
                .total_cmp(&a.value)
                .then(a.id.cmp(&b.id))
                .then(a.stat.cmp(&b.stat))
        });
        entries
    })
}

#[function_component(CoefsSummary)]
pub fn coefs_summary() -> Html {
    use_effect(|| {
        if let Some(document) = web_sys::window().and_then(|w| w.document()) {
            document.set_title("Coefficients - ESO ID Dictionary");
        }
        || ()
    });

    let ability_names = get_abilities();
    let entries = ranked_coefs();

    html! {
        <div>
            <nav style="margin-bottom: 1em;">
                <Link<Route> to={Route::Home}>
                    {"ESO ID Dictionary"}
                </Link<Route>>
                <span>{ " / Coefficients" }</span>
            </nav>
            <table style="border-collapse: collapse; width: 100%; max-width: 90em;">
                <tbody>
                    { for entries.iter().enumerate().map(|(i, e)| {
                        let name = ability_names
                            .get(&e.id)
                            .map(|n| n.trim_matches('"'))
                            .unwrap_or("???");
                        html! {
                            <tr key={i}>
                                <td style="text-align: right; padding: 0.15em 0.5em; color: #555;">{ i + 1 }</td>
                                <td style="text-align: right; padding: 0.15em 0.5em;">{ e.value }</td>
                                <td style="padding: 0.15em 0.5em;">{ e.stat.label() }</td>
                                <td style="padding: 0.15em 0.5em;">
                                    { render_ability_with_summary(&e.id, name) }
                                </td>
                            </tr>
                        }
                    }) }
                </tbody>
            </table>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_lone_weapon_damage_term_is_excluded() {
        let weapon = CoefficientType::WeaponDamage;
        assert!(is_weapon_damage_only(CoefStat::Single(35)));
        assert_eq!(CoefficientType::from_id(&35), Some(weapon));
        // Weapon + Spell Damage merged (MaxPower) is kept, as are other stats.
        assert!(!is_weapon_damage_only(CoefStat::Pair(35, 25)));
        assert!(!is_weapon_damage_only(CoefStat::Single(25)));
        assert!(!is_weapon_damage_only(CoefStat::Single(7)));
    }
}