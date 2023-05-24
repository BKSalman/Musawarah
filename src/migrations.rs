use std::error::Error;

use diesel_async::{pooled_connection::deadpool::Object, AsyncPgConnection};
use diesel_migrations_async::{embed_migrations, EmbeddedMigrations, MigrationHarness};
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub async fn run_migrations(
    connection: &mut Object<AsyncPgConnection>,
) -> std::result::Result<(), Box<dyn Error + Send + Sync>> {
    connection.run_pending_migrations(MIGRATIONS).await?;

    Ok(())
}
