pub mod entities;

#[cfg(feature = "server")]
mod init_and_get;
#[cfg(feature = "server")]
mod migrations;

#[cfg(feature = "server")]
pub use init_and_get::{db, init_db};
#[cfg(feature = "server")]
pub use migrations::Migrator;
