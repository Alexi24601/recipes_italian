use rusqlite::{params, Connection, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Default)]
struct Recipe {
    name: String,
    page_number: Option<i64>,
    baking_temp_c: Option<i64>,
    baking_time_min: Option<String>,
    baking_method: Option<String>,
    notes: Option<String>,
}

#[derive(Debug)]
struct RecipeIngredient {
    ingredient_name: String,
    quantity: Option<f64>,
    unit: Option<String>,
    step_number: Option<i64>,
}

/// Parse the baking line into (temp_c, time_min, method).
/// Examples handled:
///   "160°C, bagno maria"
///   "180°C, 20 min"
///   "160°C, 25-30 min"
///   "170°C, 40-45 min"
///   "180 min"
fn parse_baking(s: &str) -> (Option<i64>, Option<String>, Option<String>) {
    let lower = s.to_lowercase();

    // Detect special methods
    let method = if lower.contains("bagno maria") {
        Some("bagno maria".to_string())
    } else if lower.contains("frigo") {
        Some("frigo".to_string())
    } else {
        None
    };

    // Extract temperature: number immediately before '°'
    let temp = s.find('°').and_then(|idx| {
        let before = &s[..idx];
        let digits: String = before
            .chars()
            .rev()
            .take_while(|c| c.is_ascii_digit())
            .collect::<String>()
            .chars()
            .rev()
            .collect();
        digits.parse::<i64>().ok()
    });

    // Extract time: content after the first comma, stripped of "min" and method keywords
    let time = if let Some(comma) = s.find(',') {
        let after = s[comma + 1..].trim();
        let cleaned = after
            .replace("bagno maria", "")
            .replace("min", "")
            .trim()
            .to_string();
        if cleaned.is_empty() { None } else { Some(cleaned) }
    } else if lower.contains("min") {
        // e.g. "180 min" — no comma, no °C
        let cleaned = s.replace("min", "").trim().to_string();
        if cleaned.is_empty() { None } else { Some(cleaned) }
    } else {
        None
    };

    (temp, time, method)
}

/// Parse a markdown table data row.
/// Expected format: `| Qty | Unit | Ingredient | Step |`
/// Returns None for header/separator rows or malformed lines.
fn parse_ingredient_row(row: &str) -> Option<RecipeIngredient> {
    let parts: Vec<&str> = row.split('|').map(str::trim).collect();
    // parts: ["", qty, unit, ingredient, step, ""]
    if parts.len() < 5 {
        return None;
    }

    let qty_str = parts[1];
    let unit_str = parts[2];
    let ingredient = parts[3];
    let step_str = parts[4];

    // Skip header and separator rows
    if qty_str == "Qty" || qty_str.starts_with('-') {
        return None;
    }
    if ingredient.is_empty() {
        return None;
    }

    let quantity = if qty_str.is_empty() {
        None
    } else {
        // Accept both '.' and ',' as decimal separator
        qty_str.replace(',', ".").parse::<f64>().ok()
    };

    let unit = if unit_str.is_empty() { None } else { Some(unit_str.to_string()) };

    let step_number = if step_str.is_empty() {
        None
    } else {
        step_str.parse::<i64>().ok()
    };

    Some(RecipeIngredient {
        ingredient_name: ingredient.to_string(),
        quantity,
        unit,
        step_number,
    })
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let md_path = args.get(1).map(String::as_str).unwrap_or("../recipes.md");
    let db_path = args.get(2).map(String::as_str).unwrap_or("../recipes.db");

    let content = fs::read_to_string(md_path)
        .unwrap_or_else(|e| panic!("Cannot read '{}': {}", md_path, e));

    // Remove existing database so we start fresh
    if Path::new(db_path).exists() {
        fs::remove_file(db_path).unwrap_or_else(|e| panic!("Cannot remove '{}': {}", db_path, e));
    }

    let conn = Connection::open(db_path)?;

    conn.execute_batch(
        "PRAGMA foreign_keys = ON;

        CREATE TABLE recipes (
            id              INTEGER PRIMARY KEY,
            name            TEXT NOT NULL,
            page_number     INTEGER,
            baking_temp_c   INTEGER,
            baking_time_min TEXT,
            baking_method   TEXT,
            notes           TEXT
        );

        CREATE TABLE ingredients (
            id   INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE
        );

        CREATE TABLE recipe_ingredients (
            id            INTEGER PRIMARY KEY,
            recipe_id     INTEGER NOT NULL REFERENCES recipes(id),
            ingredient_id INTEGER NOT NULL REFERENCES ingredients(id),
            quantity      REAL,
            unit          TEXT,
            step_number   INTEGER,
            notes         TEXT
        );",
    )?;

    // ── Parse markdown ──────────────────────────────────────────────────────
    let mut all_recipes: Vec<(Recipe, Vec<RecipeIngredient>)> = Vec::new();
    let mut current_recipe: Option<Recipe> = None;
    let mut current_ingredients: Vec<RecipeIngredient> = Vec::new();
    let mut in_table = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("## ") {
            // Flush the previous recipe
            if let Some(recipe) = current_recipe.take() {
                all_recipes.push((recipe, std::mem::take(&mut current_ingredients)));
            }
            in_table = false;
            current_recipe = Some(Recipe {
                name: trimmed[3..].trim().to_string(),
                ..Default::default()
            });
        } else if let Some(ref mut recipe) = current_recipe {
            if trimmed.starts_with("- **Page:**") {
                let val = trimmed.trim_start_matches("- **Page:**").trim();
                recipe.page_number = val.parse().ok();
            } else if trimmed.starts_with("- **Baking:**") {
                let val = trimmed.trim_start_matches("- **Baking:**").trim();
                let (temp, time, method) = parse_baking(val);
                recipe.baking_temp_c = temp;
                recipe.baking_time_min = time;
                recipe.baking_method = method;
            } else if trimmed.starts_with("- **Notes:**") {
                let val = trimmed.trim_start_matches("- **Notes:**").trim().to_string();
                recipe.notes = Some(val);
            } else if trimmed.starts_with('|') {
                in_table = true;
                if let Some(ing) = parse_ingredient_row(trimmed) {
                    current_ingredients.push(ing);
                }
            } else if in_table && trimmed.is_empty() {
                in_table = false;
            }
        }
    }

    // Flush last recipe
    if let Some(recipe) = current_recipe.take() {
        all_recipes.push((recipe, current_ingredients));
    }

    // ── Insert into SQLite ──────────────────────────────────────────────────
    let mut ingredient_cache: HashMap<String, i64> = HashMap::new();

    for (recipe, ingredients) in &all_recipes {
        conn.execute(
            "INSERT INTO recipes (name, page_number, baking_temp_c, baking_time_min, baking_method, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                recipe.name,
                recipe.page_number,
                recipe.baking_temp_c,
                recipe.baking_time_min,
                recipe.baking_method,
                recipe.notes,
            ],
        )?;
        let recipe_id = conn.last_insert_rowid();

        for ing in ingredients {
            // Get or create the ingredient row
            let ing_id = if let Some(&id) = ingredient_cache.get(&ing.ingredient_name) {
                id
            } else {
                conn.execute(
                    "INSERT OR IGNORE INTO ingredients (name) VALUES (?1)",
                    params![ing.ingredient_name],
                )?;
                let id: i64 = conn.query_row(
                    "SELECT id FROM ingredients WHERE name = ?1",
                    params![ing.ingredient_name],
                    |row| row.get(0),
                )?;
                ingredient_cache.insert(ing.ingredient_name.clone(), id);
                id
            };

            conn.execute(
                "INSERT INTO recipe_ingredients (recipe_id, ingredient_id, quantity, unit, step_number)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![recipe_id, ing_id, ing.quantity, ing.unit, ing.step_number],
            )?;
        }

        println!(
            "  ✓ {:.<45} {} ingredients",
            format!("{} ", recipe.name),
            ingredients.len()
        );
    }

    println!(
        "\nDatabase written to '{}'\n  {} recipes\n  {} unique ingredients",
        db_path,
        all_recipes.len(),
        ingredient_cache.len()
    );

    Ok(())
}
