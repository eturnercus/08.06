use std::fs;
use std::path::PathBuf;

const APP_DIR: &str = "silenium";
const LEGACY_DIR: &str = "neuroforge";

/// Корневая папка данных приложения (`~/.local/share/silenium` на Linux).
pub fn app_data_dir() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    let new_dir = base.join(APP_DIR);
    let legacy = base.join(LEGACY_DIR);
    if !new_dir.exists() && legacy.exists() {
        let _ = fs::rename(&legacy, &new_dir);
    } else {
        fs::create_dir_all(&new_dir).ok();
    }
    new_dir
}
