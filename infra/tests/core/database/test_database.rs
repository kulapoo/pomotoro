use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;
use uuid::Uuid;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use domain::{Error, Result};
use infra::adapters::database::{DbConnection, DbPool, run_migrations};

/// Custom connection customizer for test databases
#[derive(Debug)]
struct TestConnectionCustomizer;

impl r2d2::CustomizeConnection<DbConnection, r2d2::Error>
    for TestConnectionCustomizer
{
    fn on_acquire(
        &self,
        conn: &mut DbConnection,
    ) -> std::result::Result<(), r2d2::Error> {
        use diesel::sql_query;

        // Use TRUNCATE journal mode - faster than DELETE but still avoids WAL issues
        sql_query("PRAGMA journal_mode = TRUNCATE")
            .execute(conn)
            .map_err(r2d2::Error::QueryError)?;

        // Use NORMAL locking mode to allow concurrent reads within same connection pool
        sql_query("PRAGMA locking_mode = NORMAL")
            .execute(conn)
            .map_err(r2d2::Error::QueryError)?;

        sql_query("PRAGMA synchronous = NORMAL")
            .execute(conn)
            .map_err(r2d2::Error::QueryError)?;

        sql_query("PRAGMA busy_timeout = 10000")
            .execute(conn)
            .map_err(r2d2::Error::QueryError)?;

        sql_query("PRAGMA foreign_keys = ON")
            .execute(conn)
            .map_err(r2d2::Error::QueryError)?;

        Ok(())
    }
}

/// Test database instance with automatic cleanup
pub struct TestDatabase {
    /// Temporary directory for the database (kept for RAII cleanup)
    _temp_dir: TempDir,
    /// Path to the test database
    pub db_path: PathBuf,
    /// Database connection pool
    pub pool: Arc<DbPool>,
    /// Unique test ID for isolation
    pub test_id: String,
}

impl TestDatabase {
    /// Create a new test database with a unique name
    pub fn new() -> Result<Self> {
        Self::with_name(None)
    }

    /// Reconnect to existing test database (simulates app restart)
    pub fn reconnect(&self) -> Result<Arc<DbPool>> {
        let pool = Self::establish_test_connection(&self.db_path)?;
        Ok(Arc::new(pool))
    }

    /// Establish a test-specific database connection
    /// Uses DELETE mode instead of WAL for better test isolation
    fn establish_test_connection(database_path: &PathBuf) -> Result<DbPool> {
        // Use DELETE journal mode for tests to avoid WAL issues
        let database_url = format!("sqlite://{}?mode=rwc", database_path.display());

        let manager = ConnectionManager::<DbConnection>::new(&database_url);
        let pool = r2d2::Pool::builder()
            .max_size(5) // Allow some concurrency within test, but less than production
            .min_idle(Some(1))
            .connection_customizer(Box::new(TestConnectionCustomizer))
            .build(manager)
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to create test connection pool: {}", e),
            })?;

        Ok(pool)
    }

    /// Create a new test database with a custom name prefix
    pub fn with_name(name: Option<&str>) -> Result<Self> {
        // Create temp directory in the system's temp folder for better isolation
        let temp_dir = TempDir::new().map_err(|e| Error::RepositoryError {
            message: format!("Failed to create temp directory: {}", e),
        })?;

        // Generate unique test ID with timestamp for extra uniqueness
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let test_id = match name {
            Some(n) => format!("{}_{}_{}", n, timestamp, Uuid::new_v4()),
            None => format!("{}_{}", timestamp, Uuid::new_v4()),
        };

        // Create database path
        let db_path = temp_dir.path().join(format!("{}.db", test_id));

        // Establish connection pool - use test-specific connection setup
        let pool = Self::establish_test_connection(&db_path)?;
        let pool = Arc::new(pool);

        // Run migrations
        run_migrations(&pool)?;

        Ok(Self {
            _temp_dir: temp_dir,
            db_path,
            pool,
            test_id,
        })
    }

    /// Get the database URL for this test instance
    pub fn database_url(&self) -> String {
        format!("sqlite://{}", self.db_path.display())
    }

    /// Check if the database file exists
    pub fn exists(&self) -> bool {
        self.db_path.exists()
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        // Close all connections in the pool before cleanup
        // This ensures SQLite releases all file locks
        if let Ok(pool) = Arc::try_unwrap(self.pool.clone()) {
            // Pool is not shared, we can drop it
            drop(pool);
        }

        // Since we're using DELETE journal mode, no WAL/SHM files to clean up
        // The TempDir will automatically clean up when dropped
        // This ensures test databases are removed after tests complete
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_creates_isolated_instance() {
        let db = TestDatabase::new().unwrap();
        assert!(db.exists());
        assert!(db.database_url().contains(&db.test_id));
    }

    #[test]
    fn test_database_cleanup_on_drop() {
        let db_path;
        {
            let db = TestDatabase::new().unwrap();
            db_path = db.db_path.clone();
            assert!(db_path.exists());
        } // database is dropped here

        // Database file should be cleaned up
        assert!(!db_path.exists());
    }
}
