use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use yew_router::prelude::*;
use yew::prelude::*;
use crate::flag::{FlagSummary, FlagsCompare, FlagsSummary};
use crate::id::{IdData, get_abilities};
use crate::index_state::{IndexState, init_index_cache};
use crate::fetch::init_data;
use crate::search::{Search, SkillLineComponent, SkillLineSummary};

mod id;
mod fetch;
mod index_state;
mod format;
mod search;
mod flag;

pub const SKILL_CSV: &str = include_str!("../static/player_abilities.csv");

static TIMESTAMPS: OnceLock<Vec<(u32, Vec<u32>)>> = OnceLock::new();

pub fn get_timestamps() -> &'static Vec<(u32, Vec<u32>)> {
    TIMESTAMPS.get_or_init(|| {
        let mut ts_map: HashMap<u32, Vec<u32>> = HashMap::new();
        for line in SKILL_CSV.lines() {
            let mut parts = line.splitn(3, ',');
            if let (Some(id), Some(_sl), Some(ts)) = (parts.next(), parts.next(), parts.next())
                && let (Ok(id), Ok(ts)) = (id.trim().parse::<u32>(), ts.trim().parse::<u32>()) {
                    ts_map.entry(ts).or_default().push(id);
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


static PLAYER_ABILITY_IDS: OnceLock<HashSet<u32>> = OnceLock::new();

pub fn player_ability_ids() -> &'static HashSet<u32> {
    PLAYER_ABILITY_IDS.get_or_init(|| {
        SKILL_CSV
            .lines()
            .filter_map(|line| line.splitn(3, ',').next())
            .filter_map(|id| id.trim().parse::<u32>().ok())
            .collect()
    })
}

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/search")]
    Search,
    #[at("/search/:query")]
    SearchQuery { query: String },
    #[at("/:id")]
    Ability { id: u32 },
    #[at("/skill-line/:id")]
    SkillLine { id: u32 },
    // ':index' carries both views: "3" -> flag set, "!3" -> flag excluded.
    // (There used to be a separate FlagExclude route pinned to this exact
    // same path, which the router could never actually reach.)
    #[at("/flag/:index")]
    Flag { index: String },
    #[at("/flags")]
    Flags,
    #[at("/flags/:ids")]
    FlagsCompare { ids: String },
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
    if index == IndexState::Loading {
        html! {
            <div style="height: 100vh; display: flex; justify-content: center; align-items: center;">
                <span class="loader"></span>
            </div>
        }
    } else {
        html! {
            <Switch<Route> render={move |route| switch(route, index.clone())} />
        }
    }
}

fn switch(route: Route, index: IndexState) -> Html {
    let content = match route {
        Route::Home => html! { <Home /> },
        Route::SearchQuery {query } => html! { <Search {query} /> },
        Route::Search => html! {<Search query={String::new()} />},
        Route::Ability { id } => html! { <IdData {id} {index} /> },
        Route::SkillLine { id } => html! { <SkillLineSummary {id} /> },
        Route::Flag { index } => html! { <FlagSummary {index} /> },
        Route::Flags => html! { <FlagsSummary /> },
        Route::FlagsCompare { ids } => html! { <FlagsCompare {ids} /> },
        Route::NotFound => html! {
            <div>
                <h1>{ "404" }</h1>
                <p>{ "No ability with that ID exists." }</p>
                <p>
                    <Link<Route> to={Route::SearchQuery {query: String::new()}}>
                        {"Search by name"}
                    </Link<Route>>
                </p>
            </div>
        },
    };
    html! {
        <>
            <div class="content">
            { content }
            </div>
            <footer>
                {"Made by "}<a target="_blank" href="https://github.com/sheumais">{"sheumais"}</a>{", with huge thanks to Dave from UESP. "}<a target="_blank" href="https://github.com/sheumais/esoiddictionary/">{"Source code"}</a>{" licensed under GPLv2"}
            </footer>
        </>
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
                    navigator.push( &Route::Ability { id } );
                } else {
                    navigator.push( &Route::SearchQuery { query: val } );
                }
            }
        })
    };

    let on_random = {
        let navigator = navigator.clone();
        Callback::from(move |_: MouseEvent| {
            let abilities = get_abilities();
            let ids: Vec<u32> = abilities.keys().copied().collect();
            let index = (js_sys::Math::random() * ids.len() as f64) as usize;
            navigator.push(&Route::Ability { id: ids[index] });
        })
    };

    use_effect(|| {
        if let Some(document) = web_sys::window().and_then(|w| w.document()) {
            document.set_title("ESO ID Dictionary");
        }
        || ()
    });

    html! {
        <div style="max-width: 66%; margin: 0 auto;">
            <div style="display: flex; justify-content: center; align-items: center; flex-direction: column; margin: 10em; min-width: 275px;">
                <img style="max-width: 10em; height: auto; text-align: center; user-select: none; -webkit-user-drag: none; -moz-user-select: none;" src="static/book.png" />
                <header>
                    <h1>{ "ESO ID Dictionary" }</h1>
                </header>
                <form onsubmit={onsubmit}>
                    <input
                        ref={input_ref}
                        type="text"
                        placeholder="Enter ability ID or name"
                        style={"width: 200px; margin-right: 1em;"}
                    />
                    <button type="submit">{ "Go" }</button>
                </form>
                <p>
                    <span>{ "or take me somewhere " }</span>
                    <span>
                        <a onclick={on_random} style="cursor: pointer; color: LinkText">{"random"}</a>
                    </span>
                </p>
            </div>
            <SkillLineComponent />
        </div>
    }
}

#[function_component(Main)]
fn app() -> Html {
    let index = use_state(|| IndexState::Loading);

    let idx_clone = index.clone();
    use_effect(move || {
        let index = idx_clone.clone();
        let result = init_index_cache();

        match result {
            Ok(()) => {
                if let Err(e) = init_data() {
                    index.set(IndexState::Failed(e));
                }
            }
            Err(e) => {
                index.set(IndexState::Failed(e));
            }
        }

        index.set(IndexState::Ready);
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