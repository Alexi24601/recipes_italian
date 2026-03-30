use dioxus::prelude::*;
use crate::{connection, db};
use crate::components::ingredient_table::IngredientTable;

#[component]
pub fn RecipeDetail(recipe_id: i32) -> Element {
    let recipe_data = use_memo(move || {
        connection::with_conn(|c| db::find_recipe(c, recipe_id)).ok()
    });
    let ingredients_data = use_memo(move || {
        connection::with_conn(|c| db::ingredients_for_recipe(c, recipe_id)).unwrap_or_default()
    });

    let Some(recipe) = recipe_data() else {
        return rsx! { div { class: "detail-panel", "Caricamento..." } };
    };

    rsx! {
        div { class: "detail-panel",
            h1 { class: "recipe-name", "{recipe.name}" }

            div { class: "meta-row",
                if let Some(temp) = recipe.baking_temp_c {
                    label { "Temp °C" }
                    span { class: "meta-value", "{temp}" }
                }
                if let Some(ref time) = recipe.baking_time_min {
                    label { "Tempo" }
                    span { class: "meta-value", "{time}" }
                }
                if let Some(ref method) = recipe.baking_method {
                    label { "Metodo" }
                    span { class: "meta-value", "{method}" }
                }
            }

            IngredientTable {
                recipe_id,
                ingredients: ingredients_data(),
            }

            if let Some(ref notes) = recipe.notes {
                div { class: "recipe-notes",
                    h3 { class: "notes-title", "PROCEDIMENTO" }
                    ul { class: "notes-list",
                        for note in notes.split(';').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()) {
                            li { "{note}" }
                        }
                    }
                }
            }
        }
    }
}
