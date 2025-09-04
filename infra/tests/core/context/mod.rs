// Application context for integration testing

mod app_context;
mod builder;

pub use app_context::AppContext;
pub use builder::AppContextBuilder;

pub async fn setup_test_context(name: &str) -> AppContext {
    let builder = AppContextBuilder::new()
        .with_name(name)
        .with_standard_fixtures();

    builder.build().await.expect("Failed to build test context")
}
