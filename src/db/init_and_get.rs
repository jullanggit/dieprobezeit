use sea_orm::{Database, DatabaseConnection};
use std::sync::OnceLock;

static DB: OnceLock<DatabaseConnection> = OnceLock::new();

/// Panics if connecting to db fails or DB is already set
pub async fn init_db() {
    let db = Database::connect("sqlite://mng.db?mode=rwc") // TODO: see what rwc means
        .await
        .expect("failed to connect to database");

    DB.set(db).expect("DB already set");
}

/// Panics if DB is unset
pub fn db() -> &'static DatabaseConnection {
    DB.get().expect("DB not set")
}
