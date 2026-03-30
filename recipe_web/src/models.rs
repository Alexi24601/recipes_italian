use serde::{Deserialize, Serialize};

// ── Recipe ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[derive(diesel::Queryable, diesel::Selectable, diesel::Identifiable)]
#[diesel(table_name = crate::schema::recipes)]
pub struct Recipe {
    pub id: i32,
    pub name: String,
    pub page_number: Option<i32>,
    pub baking_temp_c: Option<i32>,
    pub baking_time_min: Option<String>,
    pub baking_method: Option<String>,
    pub notes: Option<String>,
}

// ── Ingredient ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[derive(diesel::Queryable, diesel::Selectable, diesel::Identifiable)]
#[diesel(table_name = crate::schema::ingredients)]
pub struct Ingredient {
    pub id: i32,
    pub name: String,
}

// ── RecipeIngredient (raw join-table row) ─────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[derive(diesel::Queryable, diesel::Selectable, diesel::Identifiable, diesel::Associations)]
#[diesel(table_name = crate::schema::recipe_ingredients)]
#[diesel(belongs_to(Recipe))]
#[diesel(belongs_to(Ingredient))]
pub struct RecipeIngredient {
    pub id: i32,
    pub recipe_id: i32,
    pub ingredient_id: i32,
    pub quantity: Option<f32>,
    pub unit: Option<String>,
    pub step_number: Option<i32>,
    pub notes: Option<String>,
}

// ── RecipeIngredientFull (joined result for display) ──────────────────────────

/// Flattened view of a recipe_ingredients row joined with the ingredient name.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecipeIngredientFull {
    pub id: i32,
    pub ingredient_id: i32,
    pub ingredient_name: String,
    pub quantity: Option<f32>,
    pub unit: Option<String>,
    pub step_number: Option<i32>,
}
