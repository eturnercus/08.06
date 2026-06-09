use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    tauri_build::build();
    #[cfg(target_os = "linux")]
    {
        link_linux_rpath();
        stage_native_llama_libs();
    }
    #[cfg(target_os = "windows")]
    {
        stage_native_llama_libs();
    }
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn target_profile_dir() -> PathBuf {
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    manifest_dir().join("target").join(profile)
}

fn bundle_native_dir() -> PathBuf {
    manifest_dir().join("llama-libs")
}

fn cmake_cache_bin_dir(target_dir: &Path) -> Option<PathBuf> {
    let cache_root = target_dir
        .parent()
        .unwrap_or(target_dir)
        .join("llama-cmake-cache");
    let entries = fs::read_dir(&cache_root).ok()?;
    for entry in entries.flatten() {
        let bin = entry.path().join("build").join("bin");
        if bin.is_dir() {
            return Some(bin);
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn link_linux_rpath() {
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../lib/Silenium");
}

#[cfg(target_os = "linux")]
fn resolve_llama_lib_dir(target_dir: &Path) -> Option<PathBuf> {
    let probe = target_dir.join("libllama.so");
    if probe.exists() {
        if let Ok(canonical) = fs::canonicalize(&probe) {
            return canonical.parent().map(Path::to_path_buf);
        }
    }
    cmake_cache_bin_dir(target_dir)
}

#[cfg(target_os = "windows")]
fn resolve_llama_lib_dir(target_dir: &Path) -> Option<PathBuf> {
    for name in ["llama.dll", "ggml.dll"] {
        let probe = target_dir.join(name);
        if probe.exists() {
            return Some(target_dir.to_path_buf());
        }
    }
    cmake_cache_bin_dir(target_dir).or_else(|| {
        let deps = target_dir.join("deps");
        if deps.join("llama.dll").exists() || deps.join("ggml.dll").exists() {
            Some(deps)
        } else {
            None
        }
    })
}

/// Copy native llama/ggml libraries into llama-libs for Tauri bundle (deb/msi).
fn stage_native_llama_libs() {
    let target_dir = target_profile_dir();
    let bundle_dir = bundle_native_dir();
    let _ = fs::create_dir_all(&bundle_dir);

    let Some(lib_dir) = resolve_llama_lib_dir(&target_dir) else {
        println!("cargo:warning=native llama libs not found for bundling (first build?)");
        return;
    };

    #[cfg(target_os = "linux")]
    let patterns = ["libllama.so", "libggml.so", "libggml-cpu.so", "libggml-base.so"];
    #[cfg(target_os = "windows")]
    let patterns = ["llama.dll", "ggml.dll", "ggml-cpu.dll", "ggml-base.dll"];

    let Ok(entries) = fs::read_dir(&lib_dir) else {
        return;
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let lower = name.to_lowercase();
        let copy = patterns.iter().any(|p| lower.starts_with(&p.to_lowercase()))
            || (cfg!(target_os = "windows") && lower.ends_with(".dll") && lower.contains("ggml"))
            || (cfg!(target_os = "windows") && lower.ends_with(".dll") && lower.contains("llama"));
        if copy {
            let _ = fs::copy(entry.path(), bundle_dir.join(&name));
        }
    }
}
