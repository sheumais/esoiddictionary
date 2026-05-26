use std::collections::HashMap;
use std::sync::OnceLock;

use chrono::Datelike;
use web_sys::HtmlInputElement;
use yew_router::prelude::*;
use yew::prelude::*;
use crate::id::{IdData, get_abilities};
use crate::index_state::IndexState;
use crate::fetch::fetch_bytes;
use crate::index_state::parse_index;
use eso_skill_data::data_enum::*;

mod id;
mod fetch;
mod index_state;

const SKILL_CSV:   &str = include_str!("../static/player_abilities.csv");
const INDEX_URL:   &str = "static/index.bin";

static SKILL_GROUPS: OnceLock<Vec<(u32, Vec<u32>)>> = OnceLock::new();
static TIMESTAMPS: OnceLock<Vec<(u32, Vec<u32>)>> = OnceLock::new();

fn get_groups() -> &'static Vec<(u32, Vec<u32>)> {
    SKILL_GROUPS.get_or_init(|| {
        let mut map: HashMap<u32, Vec<u32>> = HashMap::new();
        for line in SKILL_CSV.lines() {
            let mut parts = line.splitn(3, ',');
            if let (Some(id), Some(sl), Some(_ts)) = (parts.next(), parts.next(), parts.next()) {
                if let (Ok(id), Ok(sl)) = (id.trim().parse::<u32>(), sl.trim().parse::<u32>()) {
                    map.entry(sl).or_default().push(id);
                }
            }
        }
        let mut groups: Vec<(u32, Vec<u32>)> = map.into_iter().collect();
        groups.sort_by_key(|(sl, _)| *sl);
        for (_, ids) in &mut groups {
            ids.sort();
        }
        groups
    })
}

fn get_timestamps() -> &'static Vec<(u32, Vec<u32>)> {
    TIMESTAMPS.get_or_init(|| {
        let mut ts_map: HashMap<u32, Vec<u32>> = HashMap::new();
        for line in SKILL_CSV.lines() {
            let mut parts = line.splitn(3, ',');
            if let (Some(id), Some(_sl), Some(ts)) = (parts.next(), parts.next(), parts.next()) {
                if let (Ok(id), Ok(ts)) = (id.trim().parse::<u32>(), ts.trim().parse::<u32>()) {
                    ts_map.entry(ts).or_default().push(id);
                }
            }
        }
        let mut timestamps: Vec<(u32, Vec<u32>)> = ts_map.into_iter().collect();
        timestamps.sort_by_key(|(ts, _)| *ts);
        for (_, ids) in &mut timestamps {
            ids.sort();
        }
        timestamps
    })
}

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/search")]
    Search,
    #[at("/:id")]
    Ability { id: u32 },
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[derive(Clone, PartialEq, Properties)]
struct SwitchProps {
    index: IndexState,
}

#[function_component(SwitchWithIndex)]
fn switch_with_index(props: &SwitchProps) -> Html {
    let index = props.index.clone();
    html! {
        <Switch<Route> render={move |route| switch(route, index.clone())} />
    }
}

fn switch(route: Route, index: IndexState) -> Html {
    let content = match route {
        Route::Home => html! { <Home /> },
        Route::Search => html! { <Search /> },
        Route::Ability { id } => html! { <IdData {id} {index} /> },
        Route::NotFound => html! {
            <div>
                <h1>{ "404" }</h1>
                <p>{ "No ability with that ID exists." }</p>
                <a href="/esoiddictionary/search/">{ "Search by name" }</a>
                <a href="/esoiddictionary/">{ "Home" }</a>
            </div>
        },
    };
    html! {
        <>
            <div class="content">
            { content }
            </div>
            <footer>
                {"Made by "}<a href="https://github.com/sheumais">{"sheumais"}</a>{", with huge thanks to Dave from UESP. "}<a href="https://github.com/sheumais/esoiddictionary/">{"Source code"}</a>{" licensed under GPLv2"}
            </footer>
        </>
    }
}

#[function_component(SkillLineComponent)]
pub fn skill_line_index() -> Html {
    let ability_names = get_abilities();

    let groups: Vec<&(u32, Vec<u32>)> = {
        get_groups()
            .iter()
            .filter(|(t, _)| {
                !SkillLine::from_id(t)
                    .map(|s| s.is_vengeance())
                    .unwrap_or(false)
                })
            .collect()
    };

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
            <h2 style="margin-top: 3em; text-align: center;">{"Player Skills"}</h2>
            <div style="columns: 10rem; column-gap: 1rem;">
                { for groups.iter().map(|(sl, ids)| html! {
                    <div key={*sl} style="break-inside:avoid;padding-top:1rem">
                        <h4 style="margin-bottom:0.25em;margin-top:0em;">
                            { format!("{} ({})", SkillLine::from_id(sl).unwrap_or(SkillLine::Scrying), sl) }
                        </h4>
                        <div>
                            { for ids.iter()
                                .map(|id| html! {
                                    <div>
                                        <a style="font-size: 0.9em;" href={format!("{}", id)} key={*id}>
                                            { format!("{}", ability_names.get(id).unwrap_or(&"?".to_string())) }
                                        </a>
                                        <br />
                                    </div>
                                })
                            }
                        </div>
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
                                <div>
                                    <a style="font-size: 0.9em;" href={format!("{}", id)} key={*id}>
                                        { format!("{}", ability_names.get(id).unwrap_or(&"?".to_string())) }
                                    </a>
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

#[function_component(Home)]
fn home() -> Html {
    let navigator = use_navigator().unwrap();
    let input_ref = use_node_ref();

    let onsubmit = {
        let navigator = navigator.clone();
        let input_ref = input_ref.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            if let Some(input) = input_ref.cast::<web_sys::HtmlInputElement>() {
                let val = input.value().trim().to_string();
                if let Ok(id) = val.parse::<u32>() {
                    navigator.push(&Route::Ability { id });
                }
            }
        })
    };

    html! {
        <div style="max-width: 66%; margin: 0 auto;">
            <div style="display: flex; justify-content: center; align-items: center; flex-direction: column; margin: 10em; min-width: 275px;">
                <img style="max-width: 10em; height: auto; text-align: center; user-select: none; image-rendering: smooth; user-drag: none;" src="static/julianos.png" />
                <header>
                    <h1>{ "ESO ID Dictionary" }</h1>
                </header>
                <form onsubmit={onsubmit}>
                    <input
                        ref={input_ref}
                        type="text"
                        placeholder="Enter ability ID"
                        style={"width: 200px; margin-right: 1em;"}
                    />
                    <button type="submit">{ "Go" }</button>
                </form>
                <span style="margin: 1em;"> 
                {"or "}
                <a href="/esoiddictionary/search">{"search by name"}</a>
                </span>
            </div>
            <SkillLineComponent />
        </div>
    }
}

#[function_component(Search)]
pub fn list_component() -> Html {
    let ability_names = get_abilities();
    let query = use_state(|| String::new());

    let on_input = {
        let query = query.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<HtmlInputElement>();
            query.set(input.value());
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
            v.into_iter().collect()
        }
    };

    html! {
        <div>
            <nav style="margin-bottom: 1em;">
                <a href="/esoiddictionary/">{ "ESO ID Dictionary" }</a>
                <span>{ " / " }</span>
                <span>{ "Search" }</span>
            </nav>
            <input
                type="text"
                placeholder="Search by name or ID"
                oninput={on_input}
                value={(*query).clone()}
            />
            <p>{ format!("Showing {} results", filtered.len()) }</p>
            { filtered.iter().map(|(id, name)| html! {
                <><a href={format!("/esoiddictionary/{}", id)}>{ format!("{} ({})", name, id) }</a><br/></>
            }).collect::<Html>() }
        </div>
    }
}

#[function_component(Main)]
fn app() -> Html {
    let index = use_state(|| IndexState::Loading);

    use_effect_with((), {
        let index = index.clone();
        move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let result = async {
                    let bytes = fetch_bytes(INDEX_URL, None).await?;
                    parse_index(&bytes)
                }
                .await;

                index.set(match result {
                    Ok(entries) => IndexState::Ready(entries),
                    Err(e)                    => IndexState::Failed(e),
                });
            });
            || ()
        }
    });

    html! {
        <BrowserRouter>
            <SwitchWithIndex index={(*index).clone()} />
        </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<Main>::new().render();
}