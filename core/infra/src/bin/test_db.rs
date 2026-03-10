use std::path::PathBuf;
use std::sync::Arc;

use infra::adapters::{
    SqliteTaskRepository, establish_connection, run_migrations,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let storage_path = PathBuf::from("/tmp/pomotoro_test");
    std::fs::create_dir_all(&storage_path)?;

    let db_path = storage_path.join("pomotoro.db");
    println!("Creating database at: {:?}", db_path);

    // Import Diesel and establish connection
    // use super::infra::adapters::{establish_connection, run_migrations, SqliteTaskRepository};
    use domain::{Task, TaskRepository};

    let pool = Arc::new(establish_connection(&db_path)?);
    println!("Database connection established");

    run_migrations(&pool)?;
    println!("Migrations completed");

    // Check if database file exists
    if db_path.exists() {
        println!("Database file created successfully!");
        let metadata = std::fs::metadata(&db_path)?;
        println!("Database size: {} bytes", metadata.len());

        // Test creating a task
        let task_repo = SqliteTaskRepository::new(pool);
        let runtime = tokio::runtime::Runtime::new()?;

        runtime.block_on(async {
            let task = Task::new("Test Task".to_string(), 4)
                .expect("Failed to create task");
            task_repo
                .create(task.clone())
                .await
                .expect("Failed to save task");
            println!("Task created with ID: {}", task.id());

            let loaded = task_repo
                .get_by_id(task.id())
                .await
                .expect("Failed to load task");
            if let Some(loaded_task) = loaded {
                println!("Task loaded successfully: {}", loaded_task.name());
            }
        });
    } else {
        println!("ERROR: Database file was not created");
    }

    Ok(())
}
