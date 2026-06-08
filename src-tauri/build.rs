fn main() {
    tauri_build::build();
    #[cfg(all(target_os = "windows", feature = "embedded-llama"))]
    stage_embedded_llama_dlls();
}

#[cfg(all(target_os = "windows", feature = "embedded-llama"))]
fn stage_embedded_llama_dlls() {
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    let target_dir = env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| manifest_dir.join("target"));
    let build_dir = target_dir.join(&profile);
    let stage_dir = manifest_dir.join("bin").join("dlls");
    let _ = fs::create_dir_all(&stage_dir);

    let mut staged = 0u32;
    if let Ok(entries) = fs::read_dir(&build_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("dll") {
                continue;
            }
            let name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n,
                None => continue,
            };
            if !is_llama_runtime_dll(name) {
                continue;
            }
            let dst = stage_dir.join(name);
            if fs::copy(&path, &dst).is_ok() {
                staged += 1;
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }

    if staged > 0 {
        println!(
            "cargo:warning=Staged {staged} llama runtime DLL(s) in {}",
            stage_dir.display()
        );
    }
}

#[cfg(all(target_os = "windows", feature = "embedded-llama"))]
fn is_llama_runtime_dll(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.starts_with("llama")
        || lower.starts_with("ggml")
        || lower.starts_with("mtmd")
        || lower == "llava_shared.dll"
}
