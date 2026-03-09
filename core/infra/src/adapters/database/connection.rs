use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel_migrations::{
    EmbeddedMigrations, MigrationHarness, embed_migrations,
};
use domain::{Error, Result};
use std::path::Path;

pub type DbConnection = SqliteConnection;
pub type DbPool = r2d2::Pool<ConnectionManager<DbConnection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn establish_connection(database_path: &Path) -> Result<DbPool> {
    // Add SQLite pragmas for better concurrent access and test isolation
    let database_url = format!("sqlite://{}?mode=rwc", database_path.display());

    let manager = ConnectionManager::<DbConnection>::new(&database_url);
    let pool = r2d2::Pool::builder()
        .max_size(10) // Increase for tests to avoid timeouts
        .connection_customizer(Box::new(ConnectionCustomizer))
        .build(manager)
        .map_err(|e| Error::RepositoryError {
            message: format!("Failed to create connection pool: {}", e),
        })?;

    Ok(pool)
}

#[derive(Debug)]
struct ConnectionCustomizer;

impl r2d2::CustomizeConnection<DbConnection, r2d2::Error>
    for ConnectionCustomizer
{
    fn on_acquire(
        &self,
        conn: &mut DbConnection,
    ) -> std::result::Result<(), r2d2::Error> {
        use diesel::sql_query;

        // Set pragmas for better concurrency and performance
        sql_query("PRAGMA journal_mode = WAL")
            .execute(conn)
            .map_err(r2d2::Error::QueryError)?;
        sql_query("PRAGMA synchronous = NORMAL")
            .execute(conn)
            .map_err(r2d2::Error::QueryError)?;
        sql_query("PRAGMA busy_timeout = 5000")
            .execute(conn)
            .map_err(r2d2::Error::QueryError)?;
        sql_query("PRAGMA foreign_keys = ON")
            .execute(conn)
            .map_err(r2d2::Error::QueryError)?;
        Ok(())
    }
}

pub fn run_migrations(pool: &DbPool) -> Result<()> {
    let mut conn = pool.get().map_err(|e| Error::RepositoryError {
        message: format!("Failed to get connection from pool: {}", e),
    })?;

    conn.run_pending_migrations(MIGRATIONS).map_err(|e| {
        Error::RepositoryError {
            message: format!("Failed to run migrations: {}", e),
        }
    })?;

    Ok(())
}
