use super::*;

#[command]
pub async fn validate_storage_path(path: String) -> Result<bool, String> {
    let path = PathBuf::from(path);

    if !path.is_absolute() {
        return Ok(false);
    }

    if let Some(parent) = path.parent() {
        if !parent.exists() {
            return Ok(false);
        }
    }

    Ok(true)
}
