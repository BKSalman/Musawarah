use std::error::Error;

use diesel::Connection;
use diesel_async::{async_connection_wrapper::AsyncConnectionWrapper, AsyncPgConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub async fn run_migrations(
    connection_url: String,
) -> std::result::Result<(), Box<dyn Error + Send + Sync>> {
    tokio::task::spawn_blocking(move || {
        let mut migration_connection =
            AsyncConnectionWrapper::<AsyncPgConnection>::establish(&connection_url)?;

        migration_connection.run_pending_migrations(MIGRATIONS)?;
        Ok(())
    })
    .await?

    // Ok(())
}
