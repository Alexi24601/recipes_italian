use std::cell::RefCell;

use diesel::connection::SimpleConnection;
use diesel::prelude::*;

const DB_SQL: &str = include_str!("recipes_dump.sql");

thread_local! {
    static CONN: RefCell<Option<SqliteConnection>> = RefCell::new(None);
}

/// Initialize the in-memory database by replaying the embedded SQL dump.
/// Must be called once before any `with_conn` calls.
pub fn init_db() {
    let mut conn = SqliteConnection::establish(":memory:")
        .expect("Failed to open in-memory SQLite");
    conn.batch_execute(DB_SQL)
        .expect("Failed to load embedded database");
    CONN.with(|c| c.borrow_mut().replace(conn));
}

/// Borrow the database connection and run a closure against it.
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
