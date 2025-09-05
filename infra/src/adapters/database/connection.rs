use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::path::PathBuf;
use domain::{Error, Result};

pub type DbConnection = SqliteConnection;
pub type DbPool = r2d2::Pool<ConnectionManager<DbConnection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn establish_connection(database_path: &PathBuf) -> Result<DbPool> {
    let database_url = format!("sqlite://{}", database_path.display());
    
    let manager = ConnectionManager::<DbConnection>::new(&database_url);
    let pool = r2d2::Pool::builder()
        .max_size(10) // Increase for tests to avoid timeouts
        .build(manager)
        .map_err(|e| Error::RepositoryError {
            message: format!("Failed to create connection pool: {}", e),
        })?;
    
    Ok(pool)
}

pub fn run_migrations(pool: &DbPool) -> Result<()> {
    let mut conn = pool.get().map_err(|e| Error::RepositoryError {
        message: format!("Failed to get connection from pool: {}", e),
    })?;
    
    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| Error::RepositoryError {
            message: format!("Failed to run migrations: {}", e),
        })?;
    
    Ok(())
}