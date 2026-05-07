use yew_router::prelude::*;
use yew::prelude::*;
use crate::id::IdData;
use crate::index_state::IndexState;
use crate::fetch::fetch_bytes;
use crate::index_state::parse_index;

mod id;
mod fetch;
mod index_state;

const INDEX_URL: &str = "static/index.bin";

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/esoiddictionary/")]
    Home,
    #[at("/esoiddictionary/:id")]
    Ability { id: u32 },
    #[not_found]
    #[at("/esoiddictionary/unknown")]
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
    match route {
        Route::Home     => html! { <Home /> },
        Route::Ability { id } => html! { <IdData {id} {index} /> },
        Route::NotFound => html! {
            <div>
                <h1>{ "404" }</h1>
                <p>{ "No ability with that ID exists." }</p>
                <a href="/esoiddictionary/">{ "Home" }</a>
            </div>
        },
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
        <div>
            <header>
                <h1>{ "ESO ID Dictionary" }</h1>
            </header>
            <form onsubmit={onsubmit}>
                <input
                    ref={input_ref}
                    type="number"
                    placeholder="Enter ability ID"
                    min="0"
                    max="300000"
                    style={"width: 200px;"}
                />
                <button type="submit">{ "Search" }</button>
            </form>
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
                    Err(e)      => IndexState::Failed(e),
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