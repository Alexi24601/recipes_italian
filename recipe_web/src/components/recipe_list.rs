use dioxus::prelude::*;
use crate::fuzzy::{self, SearchResult};

#[component]
pub fn RecipeList(
    results: Vec<SearchResult>,
    selected_id: Signal<Option<i32>>,
    mut search: Signal<String>,
    query: String,
) -> Element {
    rsx! {
        div { class: "sidebar",
            input {
                class: "search",
                placeholder: "Cerca ricetta o ingrediente...",
                value: "{search}",
                oninput: move |e| search.set(e.value()),
            }
            div { class: "recipe-items",
                for result in &results {
                    div {
                        key: "{result.recipe.id}",
                        class: if selected_id() == Some(result.recipe.id) { "recipe-item selected" } else { "recipe-item" },
                        onclick: {
                            let id = result.recipe.id;
                            move |_| selected_id.set(Some(id))
                        },
                        span {
                            for (text, is_match) in fuzzy::highlight_matches(&result.recipe.name, &query) {
                                if is_match {
                                    span { class: "highlight", "{text}" }
                                } else {
                                    span { "{text}" }
                                }
                            }
                        }
                        if let Some(ing) = &result.matched_ingredient {
                            span { class: "ingredient-match", " ({ing})" }
                        }
                    }
                }
            }
        }
    }
}
