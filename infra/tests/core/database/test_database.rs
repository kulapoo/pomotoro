use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;
use uuid::Uuid;

use domain::{Result, Error};
use infra::adapters::database::{DbPool, establish_connection, run_migrations};

/// Test database instance with automatic cleanup
pub struct TestDatabase {
    /// Temporary directory for the database
    temp_dir: TempDir,
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

    /// Create a new test database with a custom name prefix
    pub fn with_name(name: Option<&str>) -> Result<Self> {
        // Create temp directory in the project's tmp folder
        let temp_dir = TempDir::new_in("tmp").map_err(|e| Error::RepositoryError {
            message: format!("Failed to create temp directory: {}", e),
        })?;

        // Generate unique test ID
        let test_id = match name {
            Some(n) => format!("{}_{}", n, Uuid::new_v4()),
            None => Uuid::new_v4().to_string(),
        };

        // Create database path
        let db_path = temp_dir.path().join(format!("{}.db", test_id));
        
        // Establish connection pool
        let pool = establish_connection(&db_path)?;
        let pool = Arc::new(pool);

        // Run migrations
        run_migrations(&pool)?;

        Ok(Self {
            temp_dir,
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