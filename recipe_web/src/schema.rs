// @generated automatically by Diesel CLI.
// Regenerate with: diesel print-schema > src/schema.rs
//
// Note: `id` columns were promoted from Nullable<Integer> to Integer — SQLite
// INTEGER PRIMARY KEY is technically nullable but our rows always have an id.
// Note: quantity is Float (f32) as reported by SQLite via Diesel.

diesel::table! {
    ingredients (id) {
        id   -> Integer,
        name -> Text,
    }
}

diesel::table! {
    recipe_ingredients (id) {
        id            -> Integer,
        recipe_id     -> Integer,
        ingredient_id -> Integer,
        quantity      -> Nullable<Float>,
        unit          -> Nullable<Text>,
        step_number   -> Nullable<Integer>,
        notes         -> Nullable<Text>,
    }
}

diesel::table! {
    recipes (id) {
        id              -> Integer,
        name            -> Text,
        page_number     -> Nullable<Integer>,
        baking_temp_c   -> Nullable<Integer>,
        baking_time_min -> Nullable<Text>,
        baking_method   -> Nullable<Text>,
        notes           -> Nullable<Text>,
    }
}

diesel::joinable!(recipe_ingredients -> ingredients (ingredient_id));
diesel::joinable!(recipe_ingredients -> recipes     (recipe_id));

diesel::allow_tables_to_appear_in_same_query!(
    ingredients,
    recipe_ingredients,
    recipes,
);
