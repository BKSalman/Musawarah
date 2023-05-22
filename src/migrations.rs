use std::error::Error;

use diesel::pg::Pg;
use diesel_migrations_async::{embed_migrations, EmbeddedMigrations, MigrationHarness};
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub async fn run_migrations(
    connection: &mut impl MigrationHarness<Pg>,
) -> std::result::Result<(), Box<dyn Error + Send + Sync>> {
    connection.run_pending_migrations(MIGRATIONS).await?;

    Ok(())
}
