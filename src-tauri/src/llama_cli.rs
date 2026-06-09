use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;

pub struct LlamaCliConfig {
    pub model_path: String,
    pub prompt: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub top_p: f32,
    pub top_k: u32,
    pub repeat_penalty: f32,
    pub n_ctx: u32,
    pub threads: u32,
    pub gpu_layers: u32,
    pub mlock: bool,
    pub mmap: bool,
}

pub struct LlamaCliRunner;

impl LlamaCliRunner {
    pub fn find_binary() -> Option<PathBuf> {
        if let Some(p) = crate::llama_runtime::resolve_cli_binary() {
            return Some(p);
        }
        let mut candidates: Vec<PathBuf> = Vec::new();
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                candidates.push(dir.join("llama").join("llama-cli.exe"));
                candidates.push(dir.join("llama").join("llama-cli"));
                candidates.push(dir.join("llama-cli.exe"));
                candidates.push(dir.join("llama-cli"));
                candidates.push(dir.join("resources").join("llama").join("llama-cli.exe"));
                candidates.push(dir.join("resources").join("llama").join("llama-cli"));
                if let Some(parent) = dir.parent() {
                    candidates.push(parent.join("llama").join("llama-cli.exe"));
                    candidates.push(parent.join("resources").join("llama").join("llama-cli.exe"));
                }
            }
        }
        if let Ok(cwd) = std::env::current_dir() {
            candidates.push(cwd.join("src-tauri").join("bin").join("llama").join("llama-cli.exe"));
            candidates.push(cwd.join("bin").join("llama").join("llama-cli.exe"));
        }
        candidates.into_iter().find(|p| p.is_file())
    }

    fn build_command(bin: &Path, cfg: &LlamaCliConfig) -> Result<Command, String> {
        if cfg.model_path.to_lowercase().contains("mmproj") {
            return Err(
                "Файл mmproj — проектор для изображений, не языковая модель. Выберите основной .gguf."
                    .to_string(),
            );
        }
        if !Path::new(&cfg.model_path).exists() {
            return Err(format!("Файл модели не найден: {}", cfg.model_path));
        }

        let ngl = if cfg.gpu_layers > 0 {
            cfg.gpu_layers.to_string()
        } else {
            "0".into()
        };

        let mut cmd = Command::new(bin);
        cmd.arg("-m")
            .arg(&cfg.model_path)
            .arg("-p")
            .arg(&cfg.prompt)
            .arg("-n")
            .arg(cfg.max_tokens.to_string())
            .arg("-c")
            .arg(cfg.n_ctx.to_string())
            .arg("-t")
            .arg(cfg.threads.max(1).to_string())
            .arg("--temp")
            .arg(format!("{:.2}", cfg.temperature.clamp(0.0, 2.0)))
            .arg("--top-p")
            .arg(format!("{:.2}", cfg.top_p.clamp(0.0, 1.0)))
            .arg("--top-k")
            .arg(cfg.top_k.to_string())
            .arg("--repeat-penalty")
            .arg(format!("{:.2}", cfg.repeat_penalty.clamp(1.0, 2.0)))
            .arg("-ngl")
            .arg(ngl)
            .arg("--no-display-prompt")
            .arg("--simple-io")
            .arg("--log-disable");

        if cfg.mlock {
            cmd.arg("--mlock");
        } else {
            cmd.arg("--no-mlock");
        }
        if cfg.mmap {
            cmd.arg("--mmap");
        } else {
            cmd.arg("--no-mmap");
        }
        Ok(cmd)
    }

    pub fn generate(cfg: &LlamaCliConfig) -> Result<String, String> {
        Self::generate_with_callback(cfg, None, None)
    }

    pub fn generate_with_callback(
        cfg: &LlamaCliConfig,
        mut on_delta: Option<&mut dyn FnMut(&str)>,
        cancel: Option<&std::sync::atomic::AtomicBool>,
    ) -> Result<String, String> {
        let bin = Self::find_binary().ok_or_else(|| {
            "llama-cli не найден. Запустите scripts\\download-llama-win.ps1 или соберите с LLVM/MSVC."
                .to_string()
        })?;

        let mut cmd = Self::build_command(&bin, cfg)?;
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Не удалось запустить {}: {e}", bin.display()))?;

        let pid = child.id();

        let stderr = child.stderr.take();
        if let Some(err_pipe) = stderr {
            thread::spawn(move || {
                let mut reader = BufReader::new(err_pipe);
                let mut line = String::new();
                while reader.read_line(&mut line).unwrap_or(0) > 0 {
                    line.clear();
                }
            });
        }

        let mut full = String::new();
        let mut stream_sanitized = String::new();
        if let Some(stdout) = child.stdout.take() {
            let mut reader = BufReader::new(stdout);
            let mut buf = [0u8; 128];
            loop {
                if cancel.is_some_and(|f| f.load(std::sync::atomic::Ordering::SeqCst)) {
                    let _ = child.kill();
                    return Err("Генерация остановлена пользователем".into());
                }
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let chunk = String::from_utf8_lossy(&buf[..n]);
                        full.push_str(&chunk);
                        if crate::llm_sanitize::generation_should_stop(&full) {
                            break;
                        }
                        let (delta, sanitized) =
                            crate::llm_sanitize::stream_delta_since(&stream_sanitized, &full);
                        stream_sanitized = sanitized;
                        if let Some(cb) = on_delta.as_mut() {
                            if !delta.is_empty() {
                                cb(&delta);
                            }
                        }
                    }
                    Err(e) => return Err(format!("Ошибка чтения stdout llama-cli: {e}")),
                }
            }
        }

        if cancel.is_some_and(|f| f.load(std::sync::atomic::Ordering::SeqCst)) {
            let _ = child.kill();
            return Err("Генерация остановлена пользователем".into());
        }

        let status = child
            .wait()
            .map_err(|e| format!("Ошибка ожидания llama-cli (pid {pid}): {e}"))?;

        if !status.success() {
            return Err(format!(
                "llama-cli завершился с ошибкой (код {:?})",
                status.code()
            ));
        }

        let text = crate::llm_sanitize::sanitize_llm_output(full.trim());
        if text.is_empty() {
            return Err(
                "llama-cli не вернул текст. Увеличьте файл подкачки, уменьшите контекст или выберите Q4-квантизацию."
                    .to_string(),
            );
        }
        Ok(text)
    }
}

pub fn resolve_gpu_layers(
    compute_device: &str,
    configured: u32,
    gpu_memory_mb: u64,
    vram_reserve_mb: u64,
    model_size_bytes: u64,
) -> u32 {
    if compute_device == "cpu" {
        return 0;
    }
    if configured > 0 {
        return configured;
    }
    if compute_device == "auto" && gpu_memory_mb == 0 {
        return 0;
    }
    let avail = gpu_memory_mb.saturating_sub(vram_reserve_mb);
    let mut layers = if avail >= 12_000 {
        99
    } else if avail >= 8_000 {
        40
    } else if avail >= 6_000 {
        28
    } else if avail >= 4_000 {
        20
    } else if avail >= 2_000 {
        10
    } else {
        0
    };

    if model_size_bytes > 0 && avail > 0 {
        let model_mb = model_size_bytes / (1024 * 1024);
        let vram_mb = avail;
        if model_mb > vram_mb {
            let ratio = (vram_mb as f64 / model_mb as f64).clamp(0.12, 0.92);
            layers = ((layers as f64) * ratio).round() as u32;
            layers = layers.max(4).min(99);
        }
    }
    layers
}

pub fn resolve_mlock(mlock_setting: bool, swap_usage: &str, oom_policy: &str) -> bool {
    if swap_usage == "aggressive" || oom_policy == "swap" || oom_policy == "graceful_degrade" {
        false
    } else if swap_usage == "none" {
        mlock_setting
    } else {
        false
    }
}

pub fn resolve_mmap(mmap_setting: bool, swap_usage: &str) -> bool {
    if swap_usage == "none" && !mmap_setting {
        false
    } else {
        true
    }
}

pub fn resolve_context_len(requested: u32, ram_limit_mb: u64, swap_usage: &str) -> u32 {
    let base = requested.max(2048).min(131072);
    if swap_usage == "aggressive" || ram_limit_mb < 12_000 {
        base.min(8192)
    } else if ram_limit_mb < 16_000 {
        base.min(16_384)
    } else {
        base
    }
}
