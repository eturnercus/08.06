use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    tauri_build::build();
    #[cfg(target_os = "linux")]
    {
        link_linux_rpath();
        stage_llama_shared_libs();
    }
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
    let cache_root = target_dir
        .parent()
        .unwrap_or(target_dir)
        .join("llama-cmake-cache");
    let Ok(entries) = fs::read_dir(&cache_root) else {
        return None;
    };
    for entry in entries.flatten() {
        let bin = entry.path().join("build").join("bin");
        if bin.join("libllama.so.0").exists() || bin.join("libllama.so").exists() {
            return Some(bin);
        }
    }
    None
}

/// Copy libllama / libggml into llama-libs for .deb resources.
#[cfg(target_os = "linux")]
fn stage_llama_shared_libs() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    let target_dir = manifest.join("target").join(&profile);
    let bundle_dir = manifest.join("llama-libs");
    let _ = fs::create_dir_all(&bundle_dir);

    let Some(lib_dir) = resolve_llama_lib_dir(&target_dir) else {
        println!("cargo:warning=llama shared libs not found for bundling (first build?)");
        return;
    };

    let libs = [
        "libllama.so.0",
        "libggml.so.0",
        "libggml-cpu.so.0",
        "libggml-base.so.0",
    ];

    for lib in libs {
        let src = lib_dir.join(lib);
        if src.exists() {
            let _ = fs::copy(&src, bundle_dir.join(lib));
        }
    }
}
