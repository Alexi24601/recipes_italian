use dioxus::prelude::*;
use crate::models::RecipeIngredientFull;

#[component]
pub fn IngredientTable(
    recipe_id: i32,
    ingredients: Vec<RecipeIngredientFull>,
) -> Element {
    // Store the scaling ratio (1.0 = original quantities)
    let mut scale: Signal<f64> = use_signal(|| 1.0);
    // Track which ingredient ID is being used as the reference for scaling
    let mut scale_ref: Signal<Option<i32>> = use_signal(|| None);

    // Group ingredients by step_number (sorted from DB: stepped first, NULLs last).
    let mut groups: Vec<(Option<i32>, Vec<&RecipeIngredientFull>)> = Vec::new();
    for ing in &ingredients {
        match groups.last_mut() {
            Some((step, items)) if *step == ing.step_number => items.push(ing),
            _ => groups.push((ing.step_number, vec![ing])),
        }
    }

    let has_steps = groups.iter().any(|(s, _)| s.is_some());
    let max_step = groups.iter().filter_map(|(s, _)| *s).max().unwrap_or(0);
    let is_scaled = (scale() - 1.0).abs() > 0.001;

    rsx! {
        div { class: "ingredient-section",
            if is_scaled {
                div { class: "scale-bar",
                    span { class: "scale-label",
                        {format!("Scala: {:.0}%", scale() * 100.0)}
                    }
                    button {
                        class: "scale-reset",
                        onclick: move |_| {
                            scale.set(1.0);
                            scale_ref.set(None);
                        },
                        "Reset"
                    }
                }
            }
            table { class: "ingredient-table",
                thead { tr {
                    th { "Qty" }
                    th { "Unità" }
                    th { "Ingrediente" }
                    if is_scaled {
                        th { "Originale" }
                    }
                }}
                tbody {
                    for (step, items) in &groups {
                        if has_steps {
                            {
                                let label = match step {
                                    Some(s) => *s,
                                    None => max_step + 1,
                                };
                                let col_span = if is_scaled { "4" } else { "3" };
                                rsx! {
                                    tr { class: "step-header",
                                        td { colspan: col_span, "Step {label}" }
                                    }
                                }
                            }
                        }
                        for ing in items {
                            {
                                let ing_id = ing.id;
                                let original_qty = ing.quantity;
                                let scaled_qty = original_qty.map(|q| q as f64 * scale());
                                let is_ref = scale_ref() == Some(ing_id);
                                rsx! {
                                    tr {
                                        key: "{ing_id}",
                                        class: if is_ref { "scale-ref-row" } else { "" },
                                        td {
                                            if let Some(_orig) = original_qty {
                                                input {
                                                    class: "qty-input",
                                                    r#type: "number",
                                                    step: "any",
                                                    value: format!("{:.1}", scaled_qty.unwrap_or(0.0)),
                                                    onchange: move |e| {
                                                        if let Ok(new_val) = e.value().parse::<f64>() {
                                                            if let Some(orig) = original_qty {
                                                                let orig_f64 = orig as f64;
                                                                if orig_f64 > 0.0 {
                                                                    scale.set(new_val / orig_f64);
                                                                    scale_ref.set(Some(ing_id));
                                                                }
                                                            }
                                                        }
                                                    },
                                                }
                                            }
                                        }
                                        td { {ing.unit.clone().unwrap_or_default()} }
                                        td { "{ing.ingredient_name}" }
                                        if is_scaled {
                                            td { class: "original-qty",
                                                {original_qty.map(|q| format!("{q}")).unwrap_or_default()}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
