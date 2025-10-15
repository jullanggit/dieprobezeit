use sea_orm::{Database, DatabaseConnection};
use std::sync::OnceLock;

static DB: OnceLock<DatabaseConnection> = OnceLock::new();
pub async fn init_db() {
    let db = Database::connect("sqlite://mng.db?mode=rwc") // TODO: see what rwc means
        .await
        .expect("failed to connect to database");

    DB.set(db).expect("DB already set");
}

pub fn db() -> &'static DatabaseConnection {
    DB.get().expect("DB not set")
}
