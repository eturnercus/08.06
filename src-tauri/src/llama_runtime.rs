use reqwest::redirect::Policy;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlamaRuntimeStatus {
    pub embedded_available: bool,
    pub cli_path: Option<String>,
    pub cli_ready: bool,
    pub version: Option<String>,
    pub message: String,
}

pub fn llama_cli_dir() -> PathBuf {
    let mut dir = crate::app_paths::app_data_dir();
    dir.push("llama-cli");
    fs::create_dir_all(&dir).ok();
    dir
}

fn cli_binary_name() -> &'static str {
    if cfg!(windows) {
        "llama-cli.exe"
    } else {
        "llama-cli"
    }
}

pub fn bundled_cli_candidates() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            out.push(dir.join("llama").join(cli_binary_name()));
            out.push(dir.join("resources").join("llama").join(cli_binary_name()));
        }
    }
    out.push(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("bin")
            .join("llama")
            .join(cli_binary_name()),
    );
    out
}

pub fn managed_cli_path() -> PathBuf {
    llama_cli_dir().join(cli_binary_name())
}

pub fn resolve_cli_binary() -> Option<PathBuf> {
    let managed = managed_cli_path();
    if managed.is_file() {
        return Some(managed);
    }
    bundled_cli_candidates()
        .into_iter()
        .find(|p| p.is_file())
}

pub fn embedded_available() -> bool {
    cfg!(feature = "embedded-llama")
}

pub fn runtime_status() -> LlamaRuntimeStatus {
    let cli_path = resolve_cli_binary().map(|p| p.to_string_lossy().to_string());
    let cli_ready = cli_path.is_some();
    let embedded = embedded_available();
    let version = read_sidecar_version();
    let message = if embedded && cli_ready {
        "Встроенный llama.cpp и llama-cli готовы.".into()
    } else if embedded {
        "Встроенный движок активен; llama-cli можно докачать для резервного режима.".into()
    } else if cli_ready {
        "llama-cli готов (резервный бэкенд).".into()
    } else {
        "Движок не готов — нажмите «Установить llama.cpp» в настройках вывода.".into()
    };
    LlamaRuntimeStatus {
        embedded_available: embedded,
        cli_path,
        cli_ready,
        version,
        message,
    }
}

fn read_sidecar_version() -> Option<String> {
    let path = llama_cli_dir().join("version.txt");
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

fn write_sidecar_version(tag: &str) {
    let _ = fs::write(llama_cli_dir().join("version.txt"), tag);
}

fn pick_asset_name(assets: &[serde_json::Value], prefer_gpu: bool) -> Option<String> {
    let names: Vec<String> = assets
        .iter()
        .filter_map(|a| a.get("name").and_then(|n| n.as_str()).map(str::to_string))
        .collect();

    let pick = |patterns: &[&str]| -> Option<String> {
        for pat in patterns {
            if let Some(n) = names.iter().find(|n| n.contains(pat)).cloned() {
                return Some(n);
            }
        }
        None
    };

    if cfg!(windows) {
        return pick(&[
            "bin-win-cuda-12.4-x64.zip",
            "bin-win-vulkan-x64.zip",
            "bin-win-cpu-x64.zip",
        ]);
    }

    if cfg!(target_os = "linux") {
        if prefer_gpu {
            if let Some(n) = pick(&[
                "bin-ubuntu-vulkan-x64",
                "bin-ubuntu-rocm",
                "bin-ubuntu-x64",
            ]) {
                return Some(n);
            }
        }
        return pick(&["bin-ubuntu-x64", "bin-ubuntu-vulkan-x64"]);
    }

    if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            return pick(&["bin-macos-arm64"]);
        }
        return pick(&["bin-macos-x64"]);
    }

    None
}

pub async fn ensure_llama_cli(force: bool, prefer_gpu: bool) -> Result<LlamaRuntimeStatus, String> {
    let dest = managed_cli_path();
    if !force && dest.is_file() && fs::metadata(&dest).map(|m| m.len() > 1_000_000).unwrap_or(false) {
        return Ok(runtime_status());
    }

    let client = Client::builder()
        .user_agent("Silenium/1.0 (+https://github.com/eturnercus/Silenium)")
        .redirect(Policy::limited(10))
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .map_err(|e| e.to_string())?;

    let release: serde_json::Value = client
        .get("https://api.github.com/repos/ggml-org/llama.cpp/releases/latest")
        .send()
        .await
        .map_err(|e| format!("GitHub API: {e}"))?
        .json()
        .await
        .map_err(|e| format!("GitHub JSON: {e}"))?;

    let tag = release
        .get("tag_name")
        .and_then(|t| t.as_str())
        .unwrap_or("latest")
        .to_string();
    let assets = release
        .get("assets")
        .and_then(|a| a.as_array())
        .cloned()
        .unwrap_or_default();

    let asset_name = pick_asset_name(&assets, prefer_gpu)
        .ok_or_else(|| "Не найден подходящий бинарник llama.cpp для этой платформы.".to_string())?;

    let download_url = assets
        .iter()
        .find(|a| a.get("name").and_then(|n| n.as_str()) == Some(asset_name.as_str()))
        .and_then(|a| a.get("browser_download_url"))
        .and_then(|u| u.as_str())
        .ok_or_else(|| format!("URL для {asset_name} не найден"))?;

    let tmp_dir = llama_cli_dir().join("download-tmp");
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(&tmp_dir).map_err(|e| e.to_string())?;
    let archive_path = tmp_dir.join(&asset_name);

    let bytes = client
        .get(download_url)
        .send()
        .await
        .map_err(|e| format!("Загрузка {asset_name}: {e}"))?
        .bytes()
        .await
        .map_err(|e| format!("Чтение {asset_name}: {e}"))?;
    fs::write(&archive_path, &bytes).map_err(|e| e.to_string())?;

    let extract_dir = tmp_dir.join("extract");
    fs::create_dir_all(&extract_dir).map_err(|e| e.to_string())?;
    extract_archive(&archive_path, &extract_dir)?;

    let cli = find_cli_in_tree(&extract_dir).ok_or_else(|| {
        format!("llama-cli не найден внутри {asset_name}. Попробуйте переустановить.")
    })?;

    if dest.exists() {
        let _ = fs::remove_file(&dest);
    }
    fs::copy(&cli, &dest).map_err(|e| format!("Не удалось установить llama-cli: {e}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = fs::metadata(&dest) {
            let mut perms = meta.permissions();
            perms.set_mode(0o755);
            let _ = fs::set_permissions(&dest, perms);
        }
    }

    write_sidecar_version(&tag);
    let _ = fs::remove_dir_all(&tmp_dir);

    Ok(runtime_status())
}

fn extract_archive(archive: &Path, dest: &Path) -> Result<(), String> {
    let name = archive
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    if name.ends_with(".zip") {
        return extract_zip(archive, dest);
    }
    if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        let status = Command::new("tar")
            .arg("-xzf")
            .arg(archive)
            .arg("-C")
            .arg(dest)
            .status()
            .map_err(|e| format!("tar: {e}"))?;
        if status.success() {
            return Ok(());
        }
        return Err("tar завершился с ошибкой".into());
    }
    Err(format!("Неподдерживаемый архив: {}", archive.display()))
}

fn extract_zip(archive: &Path, dest: &Path) -> Result<(), String> {
    let file = fs::File::open(archive).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("zip: {e}"))?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let outpath = match file.enclosed_name() {
            Some(p) => dest.join(p),
            None => continue,
        };
        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath).ok();
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent).ok();
            }
            let mut outfile = fs::File::create(&outpath).map_err(|e| e.to_string())?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn find_cli_in_tree(dir: &Path) -> Option<PathBuf> {
    let target = cli_binary_name();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = fs::read_dir(&d) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.file_name().and_then(|s| s.to_str()) == Some(target) {
                return Some(path);
            }
        }
    }
    None
}
