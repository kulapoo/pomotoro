use super::*;

#[tauri::command(rename_all = "snake_case")]
pub async fn get_timer_state(
    timer_repo: State<'_, TimerRepositoryArc>,
) -> Result<Timer, String> {
    let timer_repo_arc = timer_repo.inner().clone();

    timer_repo_arc
        .get()
        .await
        .context("infra::commands::timer_cmd::get_timer_state - Failed to retrieve timer state")
        .map_err(|e| e.to_string())
}
