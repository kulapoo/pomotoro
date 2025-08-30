use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use domain::{Result, Error};
use infra::adapters::database::{DbPool, DbConnection};

/// Provides isolated database operations for testing
pub struct IsolatedDb {
    pool: std::sync::Arc<DbPool>,
}

impl IsolatedDb {
    pub fn new(pool: std::sync::Arc<DbPool>) -> Self {
        Self { pool }
    }

    /// Get a database connection from the pool
    pub fn get_connection(&self) -> Result<r2d2::PooledConnection<ConnectionManager<DbConnection>>> {
        self.pool
            .get()
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to get connection: {}", e),
            })
    }

    /// Execute a function within a database transaction
    /// The transaction is rolled back after the function completes
    pub fn with_transaction<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&mut DbConnection) -> Result<R>,
    {
        let mut conn = self.get_connection()?;
        conn.test_transaction(|conn| f(conn))
    }

    /// Clear all data from the database (preserves schema)
    pub fn clear_all_tables(&self) -> Result<()> {
        let mut conn = self.get_connection()?;

        // Delete all data from tables in reverse dependency order
        diesel::sql_query("DELETE FROM session_history")
            .execute(&mut *conn)
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to clear session_history: {}", e),
            })?;

        diesel::sql_query("DELETE FROM timer_state")
            .execute(&mut *conn)
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to clear timer_state: {}", e),
            })?;

        diesel::sql_query("DELETE FROM tasks")
            .execute(&mut *conn)
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to clear tasks: {}", e),
            })?;

        diesel::sql_query("DELETE FROM config")
            .execute(&mut *conn)
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to clear config: {}", e),
            })?;

        Ok(())
    }

    /// Reset specific table
    pub fn clear_table(&self, table_name: &str) -> Result<()> {
        let mut conn = self.get_connection()?;
        let query = format!("DELETE FROM {}", table_name);

        diesel::sql_query(query)
            .execute(&mut *conn)
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to clear table {}: {}", table_name, e),
            })?;

        Ok(())
    }
}

