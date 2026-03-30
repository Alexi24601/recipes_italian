use dioxus::prelude::*;
use gloo_timers::callback::Timeout;

mod schema;
mod models;
mod db;
mod connection;
mod components;
mod fuzzy;

use components::recipe_detail::RecipeDetail;
use components::recipe_list::RecipeList;
use fuzzy::SearchResult;

fn main() {
    connection::init_db();
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let selected_id: Signal<Option<i32>> = use_signal(|| None);
    let search: Signal<String> = use_signal(String::new);
    let mut debounced_query: Signal<String> = use_signal(String::new);
    let mut timer_handle: Signal<Option<Timeout>> = use_signal(|| None);

    // Preload all data once at startup
    let all_recipes = use_hook(|| {
        connection::with_conn(|c| db::all_recipes(c)).unwrap_or_default()
    });
    let all_ing_names = use_hook(|| {
        connection::with_conn(|c| db::all_recipe_ingredient_names(c)).unwrap_or_default()
    });

    // Debounce: cancel previous timer, set new 200ms timer
    use_memo(move || {
        let q = search();
        timer_handle.set(Some(Timeout::new(200, move || {
            debounced_query.set(q);
        })));
    });

    // Fuzzy search on debounced query
    let results = use_memo(move || {
        let q = debounced_query();
        if q.is_empty() {
            all_recipes
                .iter()
                .map(|r| SearchResult {
                    recipe: r.clone(),
                    score: 1.0,
                    matched_ingredient: None,
                })
                .collect()
        } else {
            fuzzy::fuzzy_search(&all_recipes, &all_ing_names, &q)
        }
    });

    rsx! {
        document::Stylesheet { href: asset!("/assets/style.css") }
        div { class: "app-layout",
            RecipeList {
                results: results(),
                selected_id,
                search,
                query: debounced_query(),
            }
            match selected_id() {
                Some(id) => rsx! {
                    RecipeDetail {
                        key: "{id}",
                        recipe_id: id,
                    }
                },
                None => rsx! {
                    div { class: "empty-state",
                        "← Seleziona una ricetta"
                    }
                },
            }
        }
    }
}
