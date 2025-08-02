use tempfile::TempDir;

/// Core test utilities for managing temporary directories and basic test setup
pub struct TestContext {
    pub _temp_dir: TempDir,
}

impl TestContext {
    pub fn new() -> Self {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
        Self {
            _temp_dir: temp_dir,
        }
    }

    pub fn temp_path(&self) -> &std::path::Path {
        self._temp_dir.path()
    }
}