use crate::models::Recipe;

#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    pub recipe: Recipe,
    pub score: f64,
    pub matched_ingredient: Option<String>,
}

/// Search recipes by name and ingredient using case-insensitive substring matching.
pub fn fuzzy_search(
    recipes: &[Recipe],
    ingredient_names: &[(i32, String)],
    query: &str,
) -> Vec<SearchResult> {
    let q = query.to_lowercase();
    let mut results: Vec<SearchResult> = Vec::new();

    for recipe in recipes {
        let name_lower = recipe.name.to_lowercase();
        let mut matched = false;
        let mut matched_ing: Option<String> = None;

        if name_lower.contains(&q) {
            matched = true;
        }

        // Check ingredient names for this recipe
        for (rid, ing_name) in ingredient_names {
            if *rid != recipe.id {
                continue;
            }
            if ing_name.to_lowercase().contains(&q) {
                if !matched {
                    matched_ing = Some(ing_name.clone());
                }
                matched = true;
                break;
            }
        }

        if matched {
            results.push(SearchResult {
                recipe: recipe.clone(),
                score: 1.0,
                matched_ingredient: matched_ing,
            });
        }
    }

    results
}

/// Split a name into (text, is_highlighted) segments based on substring match.
pub fn highlight_matches(name: &str, query: &str) -> Vec<(String, bool)> {
    if query.is_empty() {
        return vec![(name.to_string(), false)];
    }

    let name_lower = name.to_lowercase();
    let q_lower = query.to_lowercase();

    if let Some(start) = name_lower.find(&q_lower) {
        let end = start + query.len();
        let mut segments = Vec::new();
        if start > 0 {
            segments.push((name[..start].to_string(), false));
        }
        segments.push((name[start..end].to_string(), true));
        if end < name.len() {
            segments.push((name[end..].to_string(), false));
        }
        segments
    } else {
        vec![(name.to_string(), false)]
    }
}
