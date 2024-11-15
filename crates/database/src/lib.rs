pub mod models;
pub mod schema;

pub const MIGRATIONS: diesel_async_migrations::EmbeddedMigrations =
    diesel_async_migrations::embed_migrations!();
