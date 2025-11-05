pub mod entities;

#[cfg(feature = "server")]
mod init_and_get;
#[cfg(feature = "server")]
mod migrations;

#[cfg(feature = "server")]
pub use init_and_get::{db, init_db};
#[cfg(feature = "server")]
pub use migrations::Migrator;

impl entities::edition::Model {
    // data - title?
    pub fn label(&self) -> String {
        self.title.as_ref().map_or(self.date.to_string(), |title| {
            format!("{} - {title}", self.date)
        })
    }
}
