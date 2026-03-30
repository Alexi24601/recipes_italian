use diesel::prelude::*;

use crate::models::{Ingredient, Recipe, RecipeIngredient, RecipeIngredientFull};
use crate::schema::{ingredients, recipe_ingredients, recipes};

type Conn = SqliteConnection;

// ── Recipes ───────────────────────────────────────────────────────────────────

pub fn all_recipes(conn: &mut Conn) -> QueryResult<Vec<Recipe>> {
    recipes::table.order(recipes::name.asc()).load(conn)
}

pub fn find_recipe(conn: &mut Conn, id: i32) -> QueryResult<Recipe> {
    recipes::table.find(id).first(conn)
}

/// All (recipe_id, ingredient_name) pairs — used by fuzzy search.
pub fn all_recipe_ingredient_names(conn: &mut Conn) -> QueryResult<Vec<(i32, String)>> {
    recipe_ingredients::table
        .inner_join(ingredients::table)
        .select((recipe_ingredients::recipe_id, ingredients::name))
        .load(conn)
}

// ── Ingredients ───────────────────────────────────────────────────────────────

pub fn ingredients_for_recipe(
    conn: &mut Conn,
    recipe_id: i32,
) -> QueryResult<Vec<RecipeIngredientFull>> {
    // Sort by step_number with NULLs last (SQLite sorts NULLs first in ASC).
    let rows: Vec<(RecipeIngredient, Ingredient)> = recipe_ingredients::table
        .inner_join(ingredients::table)
        .filter(recipe_ingredients::recipe_id.eq(recipe_id))
        .order((
            recipe_ingredients::step_number.is_null().asc(),
            recipe_ingredients::step_number.asc(),
        ))
        .select((RecipeIngredient::as_select(), Ingredient::as_select()))
        .load(conn)?;

    Ok(rows
        .into_iter()
        .map(|(ri, ing)| RecipeIngredientFull {
            id: ri.id,
            ingredient_id: ing.id,
            ingredient_name: ing.name,
            quantity: ri.quantity,
            unit: ri.unit,
            step_number: ri.step_number,
        })
        .collect())
}
