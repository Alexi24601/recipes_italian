# Recipe Web App — Implementation Todo

Based on `DIOXUS_ROADMAP.md`, pivoted to a **pure client-side WASM** architecture
(no server). The SQLite database is embedded in the binary. Diesel compiles to
WASM via `sqlite-wasm-rs` (automatic when targeting `wasm32-unknown-unknown`).

Progress: `- [ ]` not started · `- [x]` done

---

## Phase 1 — Environment Setup ✅

> One-time terminal tasks. Nothing to write yet.

- [x] Install the WASM compilation target
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
- [x] Install the Dioxus CLI
  ```bash
  cargo install dioxus-cli
  dx --version   # installed: dioxus 0.7.3
  ```
- [x] Install the Diesel CLI with bundled SQLite
  ```bash
  cargo install diesel_cli --no-default-features --features sqlite-bundled
  diesel --version   # installed: 2.3.6, backend: sqlite
  ```
- [x] Verify `recipes.db` exists and has data
  ```bash
  sqlite3 /home/alex/recipes_italian/recipes.db "SELECT COUNT(*) FROM recipes;"
  # result: 50 ✓
  ```

---

## Phase 2 — Project Scaffolding ✅

> Create the skeleton. After this phase `dx serve` should start (blank page is fine).

- [x] Create the Cargo project
  ```bash
  cd /home/alex/recipes_italian
  cargo new recipe_web
  ```
- [x] Write `recipe_web/Cargo.toml` with all dependencies
  - `dioxus = { version = "0.7", features = ["fullstack"] }` ← 0.7, no "router" needed
  - `serde = { version = "1", features = ["derive"] }`
  - server-only deps behind `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`:
    - `diesel = { version = "2", features = ["sqlite", "r2d2"] }`
    - `libsqlite3-sys = { version = "0.28", features = ["bundled"] }`
    - `r2d2 = "0.8"`
    - `tokio = { version = "1", features = ["full"] }`
    - `dotenvy = "0.15"`
  - `[profile.release] opt-level = "z"`
- [x] Write `recipe_web/Dioxus.toml`
  - In 0.7 there is no `default_platform` field — fullstack is enabled via
    `--fullstack` flag or auto-detected from the crate feature
  - `watch_path = ["src", "assets"]`
  - CSS is handled via the `asset!()` macro in 0.7, not Dioxus.toml
- [x] Create `recipe_web/assets/` directory and `assets/style.css` stub
- [x] Create `recipe_web/src/` directory structure
  ```
  src/
  ├── main.rs              ✓
  ├── models.rs            ✓ (stub)
  ├── schema.rs            ✓ (stub — filled by diesel print-schema in Phase 3)
  ├── server/
  │   ├── mod.rs           ✓ (stub)
  │   └── db.rs            ✓ (stub)
  └── components/
      ├── mod.rs           ✓ (stub)
      ├── recipe_list.rs   ✓ (stub)
      ├── recipe_detail.rs ✓ (stub)
      └── ingredient_table.rs ✓ (stub)
  ```
- [x] Write a minimal `src/main.rs` — `dioxus::launch(App)` with a placeholder page
- [x] Confirm the project compiles
  ```bash
  cargo check   # ✓ clean, 387 packages resolved
  ```
- [x] Confirm `dx serve` starts without errors
  ```bash
  dx serve --fullstack --bundle server --target x86_64-unknown-linux-gnu --port 9090
  # ✓ server started cleanly (no error output before termination)
  ```
  > **Note for development:** run `dx serve` from inside `recipe_web/` with full
  > PATH: `export PATH="/home/alex/.cargo/bin:/usr/bin:/bin:$PATH"` — or add
  > `~/.cargo/bin` to your shell's `~/.bashrc`.

---

## Phase 3 — Diesel Schema Generation ✅

> Point Diesel at the existing database and generate `schema.rs`.

- [x] Create `recipe_web/.env` — `DATABASE_URL=../recipes.db`
- [x] Create `recipe_web/diesel.toml` — `file = "src/schema.rs"`
- [x] Run `diesel print-schema` and write output to `src/schema.rs`
- [x] Verify three `table!` blocks: `recipes`, `ingredients`, `recipe_ingredients` ✓
- [x] Verify two `joinable!` macros present ✓
- [x] `mod schema;` already declared in `main.rs` — `cargo check` passes ✓

> **Two type differences from the roadmap — affects Phase 4 models:**
> - `id` columns generated as `Nullable<Integer>` by SQLite; manually promoted to
>   `Integer` in `schema.rs` (ids are never null). Model structs use `i32`, not `Option<i32>`.
> - `quantity` and `baking_temp_c` are `Float` (f32), not `Double` (f64) as the
>   roadmap assumed. Model structs use `Option<f32>` for these fields.

---

## Phase 4 — Data Models ✅

> Write `src/models.rs`. These structs are shared by server and client.

- [x] Write the `Recipe` struct
  - fields: `id`, `name`, `page_number`, `baking_temp_c`, `baking_time_min`,
    `baking_method`, `notes`
  - always derives: `Debug, Clone, Serialize, Deserialize, PartialEq`
  - server-only `cfg_attr` derives: `Queryable, Selectable, Identifiable, AsChangeset`
- [x] Write the `NewRecipe` struct (INSERT only, no `id`)
  - server-only derives: `Insertable`
- [x] Write the `Ingredient` struct
  - server-only derives: `Queryable, Selectable, Identifiable`
- [x] Write the `RecipeIngredient` struct (raw join-table row)
  - server-only derives: `Queryable, Selectable, Identifiable, Associations, AsChangeset`
  - `#[diesel(belongs_to(Recipe))]` and `#[diesel(belongs_to(Ingredient))]`
- [x] Write `RecipeIngredientFull` (joined result sent to client, no Diesel derives)
  - fields: `id`, `ingredient_id`, `ingredient_name`, `quantity`, `unit`, `step_number`
- [x] Write `RecipeIngredientInput` (used for create/update calls from client)
  - derives: `Default` (so new blank rows can be created with `::default()`)
- [x] Add `mod models;` to `src/main.rs`
- [x] Confirm `cargo build` passes — the `cfg_attr` gates are the tricky part here

---

## Phase 5 — Connection Pool ✅

> Server-only. Sets up r2d2 so all server functions share one pool.

- [x] Write `src/server/mod.rs`
  - declare `pub mod db;`
  - define `pub type DbPool`
  - define `static POOL: OnceLock<DbPool>`
  - implement `pub fn pool() -> &'static DbPool` that:
    - calls `dotenvy::dotenv().ok()`
    - reads `DATABASE_URL` from env, falls back to `"../recipes.db"`
    - builds the `r2d2::Pool` with `max_size(4)`
- [x] Add `#[cfg(not(target_arch = "wasm32"))] mod server;` to `src/main.rs`
- [x] Confirm `cargo build` passes

---

## Phase 6 — Database Query Functions ✅

> Write `src/server/db.rs` using the Diesel DSL. No raw SQL strings.

> **Note:** Added `returning_clauses_for_sqlite_3_35` feature to diesel in `Cargo.toml`
> so that `.returning().get_result()` works on SQLite 3.35+.

### Recipes
- [x] `all_recipes(conn)` → `QueryResult<Vec<Recipe>>`
- [x] `find_recipe(conn, id)` → `QueryResult<Recipe>`
- [x] `insert_recipe(conn, name)` → `QueryResult<Recipe>`
- [x] `update_recipe(conn, &recipe)` → `QueryResult<usize>`
- [x] `delete_recipe(conn, id)` → `QueryResult<usize>`
  - deletes child `recipe_ingredients` rows first

### Ingredients
- [x] `ingredients_for_recipe(conn, recipe_id)` → `QueryResult<Vec<RecipeIngredientFull>>`
- [x] `upsert_ingredient(conn, recipe_id, &input)` → `QueryResult<()>`
- [x] `update_ingredient_row(conn, ri_id, &input)` → `QueryResult<()>`
- [x] `delete_ingredient_row(conn, id)` → `QueryResult<usize>`
- [x] Confirm `cargo build` passes

---

## Phase 7 — Architecture Pivot: Fullstack → Client-Side WASM

> Restructure the project for a pure WASM app with no server.
> Diesel compiles to WASM automatically — when targeting `wasm32-unknown-unknown`,
> the `sqlite` feature uses `sqlite-wasm-rs` as FFI instead of `libsqlite3-sys`.
> The `db.rs` query functions already use `type Conn = SqliteConnection` (not a
> pooled type), so they need **zero signature changes**.

### 7a — Rewrite `Cargo.toml`

- [x] Change dioxus feature: `"fullstack"` → `"web"`
- [x] Move `diesel` into regular `[dependencies]`:
  ```toml
  diesel = { version = "2", features = ["sqlite", "returning_clauses_for_sqlite_3_35"] }
  ```
  (drop the `r2d2` feature — no connection pool needed)
- [x] Add `sqlite-wasm-rs` as a direct dependency (needed to call `init_sqlite()`):
  ```toml
  sqlite-wasm-rs = { version = "0.5", default-features = false }
  ```
- [x] Remove the entire `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` section
- [x] Remove `libsqlite3-sys`, `r2d2`, `tokio`, `dotenvy`
- [x] Final `Cargo.toml`:
  ```toml
  [package]
  name    = "recipe_web"
  version = "0.1.0"
  edition = "2021"

  [dependencies]
  dioxus         = { version = "0.7", features = ["web"] }
  serde          = { version = "1",   features = ["derive"] }
  diesel         = { version = "2",   features = ["sqlite", "returning_clauses_for_sqlite_3_35"] }
  sqlite-wasm-rs = { version = "0.5", default-features = false }

  [profile.release]
  opt-level = "z"
  ```

### 7b — Move query functions out of `server/`

- [x] Move `src/server/db.rs` → `src/db.rs` (file content unchanged)
- [x] Delete `src/server/mod.rs`
- [x] Delete the `src/server/` directory

### 7c — Update `src/models.rs` — remove all cfg gates

- [x] Remove every `#[cfg_attr(not(target_arch = "wasm32"), ...)]` wrapper
- [x] Make all Diesel derives unconditional:
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
  #[derive(diesel::Queryable, diesel::Selectable, diesel::Identifiable, diesel::AsChangeset)]
  #[diesel(table_name = crate::schema::recipes)]
  pub struct Recipe { ... }
  ```
- [x] Do the same for `NewRecipe`, `Ingredient`, `RecipeIngredient`
- [x] `RecipeIngredientFull` and `RecipeIngredientInput` have no Diesel derives — leave unchanged

### 7d — Update `src/main.rs`

- [x] Remove `#[cfg(not(target_arch = "wasm32"))] mod server;`
- [x] Add `mod db;`
- [x] Add `mod connection;` (new file, created in Phase 8)
- [x] Keep `mod models;`, `mod schema;`, `mod components;`

### 7e — Checkpoint

- [x] Run `cargo check --target wasm32-unknown-unknown`
  - Diesel + sqlite-wasm-rs compiles cleanly to WASM ✓
  - Only warnings are "unused" functions in db.rs (expected — components not wired yet)

---

## Phase 8 — Database Embedding & Connection Module

> Create `src/connection.rs` — embeds `recipes.db` at compile time, initializes
> SQLite in WASM, and provides a `with_conn()` helper for all query functions.

- [x] Generate SQL dump from recipes.db:
  ```bash
  sqlite3 ../recipes.db .dump > src/recipes_dump.sql   # 484 lines
  ```
- [x] Embed the SQL dump:
  ```rust
  const DB_SQL: &str = include_str!("recipes_dump.sql");
  ```
  > **Note:** `deserialize_readonly_database_from_buffer` uses
  > `SQLITE_DESERIALIZE_READONLY` so the connection would be immutable.
  > Using the SQL dump approach instead gives us a writable `:memory:` DB.
- [x] Create a `thread_local!` for the connection (WASM is single-threaded, no Mutex needed)
- [x] Write `init_db()` — **synchronous** (no async needed; `sqlite3_os_init` is called
  automatically by SQLite when `SqliteConnection::establish()` runs):
  ```rust
  pub fn init_db() {
      let mut conn = SqliteConnection::establish(":memory:")
          .expect("Failed to open in-memory SQLite");
      conn.batch_execute(DB_SQL)
          .expect("Failed to load embedded database");
      CONN.with(|c| c.borrow_mut().replace(conn));
  }
  ```
- [x] Write `with_conn()` helper:
  ```rust
  pub fn with_conn<F, R>(f: F) -> R
  where
      F: FnOnce(&mut SqliteConnection) -> R,
  {
      CONN.with(|c| {
          let mut borrow = c.borrow_mut();
          let conn = borrow.as_mut().expect("DB not initialized — call init_db() first");
          f(conn)
      })
  }
  ```
- [x] Confirm `cargo check --target wasm32-unknown-unknown` passes ✓

---

## Phase 9 — App Skeleton & Global State

> The `App` component owns the recipe list and the selected recipe id.
> All data reads are synchronous via `with_conn()` — no server futures.

- [x] Initialize DB before launch — `init_db()` is synchronous, called in `main()`:
  ```rust
  fn main() {
      connection::init_db();
      dioxus::launch(App);
  }
  ```
- [x] Add `use_signal::<Option<i32>>` for `selected_id`
- [x] Add `use_signal::<u64>` for `refresh_counter` — increment after any mutation
  to trigger re-reads
- [x] Load recipe list synchronously:
  ```rust
  let recipes = use_memo(move || {
      refresh_counter();  // subscribe to changes
      connection::with_conn(|c| db::all_recipes(c)).unwrap_or_default()
  });
  ```
- [x] Render `RecipeList` + conditional `RecipeDetail` (or empty-state div)
  - Uses `key: "{id}"` on `RecipeDetail` to reset component state when switching recipes
- [x] Stub components accept correct props — `cargo check --target wasm32-unknown-unknown` passes ✓

---

## Phase 10 — RecipeList Component

> `src/components/recipe_list.rs`

- [x] Accept props: `recipes: Vec<Recipe>`, `selected_id: Signal<Option<i32>>`,
      `refresh_counter: Signal<u64>`
- [x] Add a `use_signal::<String>` for the search input
- [x] Filter `recipes` by the search string (case-insensitive, `.contains`)
- [x] Render a scrollable list of recipe name divs
  - Highlight the selected one with the `selected` CSS class
  - `onclick` sets `selected_id`
- [x] Implement the **"+ Nuova"** button — calls `insert_recipe` directly, selects
  the new recipe, increments `refresh_counter`
- [ ] Implement **delete recipe** — deferred to Phase 11 (RecipeDetail) where
  the delete button lives alongside the recipe details
- [x] `cargo check --target wasm32-unknown-unknown` passes ✓

---

## Phase 11 — RecipeDetail Component

> `src/components/recipe_detail.rs`

- [x] Accept props: `recipe_id: i32`, `selected_id: Signal<Option<i32>>`,
      `refresh_counter: Signal<u64>`
- [x] Load recipe and ingredients synchronously via `use_memo` subscribed to `refresh_counter`
- [x] Add `use_signal` to hold a local `Option<Recipe>` copy for editing
- [x] Use `use_effect` to sync `recipe_data` into the local signal on data reload
- [x] Render all editable fields:
  - [x] Recipe name (`<input class="recipe-name">`)
  - [x] Page number (`<input type="number">`)
  - [x] Baking temp (`<input type="number">`)
  - [x] Baking time (`<input>` — text, supports ranges like "25-30")
  - [x] Baking method (`<input>`)
  - [x] Notes (`<textarea>`)
- [x] Implement **Salva** button — calls `update_recipe`, increments `refresh_counter`
- [x] Implement **Elimina** button — calls `delete_recipe`, clears `selected_id`,
      increments `refresh_counter`
- [x] Pass ingredient data and `refresh_counter` to `IngredientTable`
- [x] Recipe switching handled by `key: "{id}"` on the component in `App`
- [x] `cargo check --target wasm32-unknown-unknown` passes ✓

---

## Phase 12 — IngredientTable & IngredientRow Components

> `src/components/ingredient_table.rs`

### IngredientTable
- [x] Accept props: `recipe_id: i32`, `ingredients: Vec<RecipeIngredientFull>`,
      `refresh_counter: Signal<u64>`
- [x] Render the `<table>` with header row (Qty / Unità / Ingrediente / Step)
- [x] Render one `IngredientRow` per ingredient (use `key="{ing.id}"`)
- [x] Implement **"+ Aggiungi ingrediente"** button — calls `upsert_ingredient`
      with `default()` input, increments `refresh_counter`

### IngredientRow
- [x] Accept props: `ingredient: RecipeIngredientFull`, `refresh_counter: Signal<u64>`
- [x] Create a local `use_signal::<RecipeIngredientInput>` pre-filled from props
- [x] Render four inputs: Qty (number, step="any"), Unità (text), Ingrediente (text), Step (number)
- [x] On every `onfocusout`: save immediately via `update_ingredient_row`
- [x] Implement the **🗑 delete** button — calls `delete_ingredient_row`,
      increments `refresh_counter`
- [x] `cargo check --target wasm32-unknown-unknown` passes with zero warnings ✓

---

## Phase 13 — CSS

> `assets/style.css` — paste the styles from `DIOXUS_ROADMAP.md` section 13.

- [x] Reset (`box-sizing`, `margin`, `padding`)
- [x] `.app-layout` — flexbox, full viewport height
- [x] `.sidebar` — fixed 260px width, flex column, scrollable recipe list
- [x] `.search` input — full width, rounded corners
- [x] `.btn-new` — warm brown accent colour (`#b5956e`) with hover
- [x] `.recipe-item` — hover state, selected state with left border
- [x] `.detail-panel` — flex 1, padding, `overflow-y: auto`
- [x] `.recipe-name` — large, borderless until focused
- [x] `.meta-row` — flex row, wraps on narrow screens
- [x] `.notes` textarea — resizable, subtle border
- [x] `.btn-save` — accent colour with hover
- [x] `.btn-delete` — red outline, fills red on hover
- [x] `.action-row` — flex row for Salva + Elimina buttons
- [x] `.ingredient-table` — collapsed borders, uppercase column headers
- [x] `.ingredient-table input` — transparent border until focused (spreadsheet feel)
- [x] `.btn-delete-row` — invisible until hovered, red on hover
- [x] `.btn-add-ing` — dashed border, accent colour with hover
- [x] `.empty-state` — centred placeholder text
- [x] Stylesheet linked via `document::Stylesheet { href: asset!("/assets/style.css") }`
- [x] `cargo check --target wasm32-unknown-unknown` passes with zero warnings ✓
- [ ] Confirm layout looks correct at 1280×800 viewport (visual check with `dx serve`)

---

## Phase 14 — Read-Only Remodel + Search by Ingredient

> The editing features (add/edit/delete recipes and ingredients) are not needed.
> The app should be a **read-only recipe viewer** searchable by recipe name
> or ingredient name.

### 14a — Add `search_recipes` query to `db.rs`

- [x] Add `search_recipes(conn, query) -> QueryResult<Vec<Recipe>>`:
  - Returns all recipes if query is empty
  - Otherwise returns recipes where name matches OR any ingredient name
    matches (case-insensitive, `LIKE %query%`)
  - Two queries (by_name + by_ingredient), combine + dedup IDs, final load
- [x] Remove unused mutation functions: `insert_recipe`, `update_recipe`,
      `delete_recipe`, `upsert_ingredient`, `update_ingredient_row`,
      `delete_ingredient_row`, `find_or_create_ingredient`
  > **Note:** components still reference removed functions — compilation
  > will break until phases 14d–14f rewrite them. Checkpoint at 14h.

### 14b — Remove unused models

- [x] Remove `NewRecipe` from `models.rs` (no inserts)
- [x] Remove `RecipeIngredientInput` from `models.rs` (no edits)
- [x] Remove `AsChangeset` derives from `Recipe` and `RecipeIngredient` (no updates)

### 14c — Simplify `main.rs`

- [x] Remove `refresh_counter` signal (no mutations)
- [x] Move search state to `App` so it drives `search_recipes`
- [x] Replace `use_memo` with `search_recipes` query:
  ```rust
  let recipes = use_memo(move || {
      connection::with_conn(|c| db::search_recipes(c, &search())).unwrap_or_default()
  });
  ```
- [x] Remove `refresh_counter` from `RecipeList` and `RecipeDetail` props
- [x] Remove `selected_id` from `RecipeDetail` props (no delete button)

### 14d — Rewrite `RecipeList` as read-only

- [x] Remove `refresh_counter` prop
- [x] Remove **"+ Nuova"** button
- [x] Search input updates the search signal in `App` (passed as prop or lifted)
- [x] List displays recipes from the filtered results (no client-side re-filtering)
- [x] Keep recipe selection (`selected_id`)

### 14e — Rewrite `RecipeDetail` as read-only

- [x] Remove `selected_id` and `refresh_counter` props
- [x] Remove local `Signal<Option<Recipe>>` for editing — display `recipe_data` directly
- [x] Replace all `<input>` / `<textarea>` with read-only text (`span`, `p`, `div`)
- [x] Remove **Salva** and **Elimina** buttons
- [x] Remove `action-row`
- [x] Keep `IngredientTable` but pass no `refresh_counter`

### 14f — Rewrite `IngredientTable` as read-only

- [x] Remove `refresh_counter` prop
- [x] Remove `IngredientRow` component (no editing)
- [x] Remove **"+ Aggiungi ingrediente"** button
- [x] Render a simple read-only `<table>` with `<td>` cells (no `<input>`)

### 14g — Clean up CSS

- [x] Remove `.btn-save`, `.btn-delete`, `.action-row` styles
- [x] Remove `.btn-new` style
- [x] Remove `.btn-add-ing`, `.btn-delete-row` styles
- [x] Remove `.ingredient-table input` focus styles
- [x] Keep `.ingredient-table` base styles for the read-only table
- [x] Add `.meta-value` style for read-only metadata display
- [x] Add `.recipe-notes` style for read-only notes

### 14h — Checkpoint

- [x] `cargo check --target wasm32-unknown-unknown` — zero warnings
- [ ] `dx serve` — app loads, search filters by name and ingredient, detail is read-only

---

## Phase 15 — End-to-End Verification

> Work through every user action and confirm it works.

- [ ] **Load**: all 50 recipes appear in the sidebar alphabetically
- [ ] **Search by name**: typing "castella" filters to matching recipes
- [ ] **Search by ingredient**: typing "burro" shows all recipes containing burro
- [ ] **Select**: clicking a recipe loads it in the right panel with ingredients
- [ ] **Read-only**: no edit controls visible — recipe detail and ingredients are
      display-only
- [ ] **Switch recipes fast**: click between several recipes — each loads correctly
- [ ] **Clear search**: clearing the search box shows all recipes again
- [ ] **WASM bundle size**: check `dx build --release` output — should be reasonable
- [ ] **No console errors**: open DevTools, confirm no JS/WASM errors

---

## Phase 16 — Future Extensions (optional)

> Do these after the MVP is solid.

- [ ] **Static hosting**: since there is no server, deploy the built WASM bundle
      to GitHub Pages, Netlify, Vercel, or any static file host
- [ ] **Scaling factor**: add a numeric input to the detail panel; multiply all
      ingredient quantities client-side using signal math — no DB write needed
- [ ] **Print view**: add `@media print` CSS that hides the sidebar and buttons,
      formats the ingredient table cleanly for A4
- [ ] **Categories/tags**: add a `category TEXT` column to `recipes`, rerun
      `diesel print-schema`, follow compiler errors to update all affected code
- [ ] **Export to Markdown**: add a function that queries all recipes via Diesel
      and regenerates `recipes.md` in the same format as the original
