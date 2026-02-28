use super::*;

#[tauri::command(rename_all = "snake_case")]
pub async fn update_timer_secs(
    timer_repo: State<'_, TimerRepositoryArc>,
    remaining_seconds: u32,
) -> Result<(), String> {
    let timer_repo_arc = timer_repo.inner().clone();

    usecases::timer::update_timer_secs(timer_repo_arc.clone(), remaining_seconds)
        .await
        .context("infra::commands::timer_cmd::update_timer_secs - Failed to update timer seconds")
        .map_err(|e| e.to_string())?;

    Ok(())
}