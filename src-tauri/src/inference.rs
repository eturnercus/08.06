use parking_lot::RwLock;
#[cfg(feature = "embedded-llama")]
use parking_lot::Mutex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::gguf_runner::{generate_with_best_backend, GenerateParams, is_legacy_stub};
#[cfg(feature = "embedded-llama")]
use crate::gguf_runner::GgufRuntime;
use crate::memory::MemoryStore;
use crate::settings::AppSettings;
use crate::settings_engine::{
    self, check_user_input, cross_modal_user_note, effective_max_tokens, effective_temperature,
    enrich_system_prompt, filter_model_output, filter_stm, maybe_dream_consolidate, recall_ltm,
    synaptic_backend_pref, tune_generate_params,
};
use crate::stream_sink::{AgentStreamSink, StreamSink, TokenSink};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub path: String,
    pub format: String,
    pub backend: String,
    pub size_bytes: u64,
    pub loaded: bool,
    pub source: String,
    #[serde(default)]
    pub verified: bool,
    #[serde(default)]
    pub download_progress: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadResult {
    pub success: bool,
    pub model: Option<ModelInfo>,
    pub message: String,
    pub bytes_downloaded: u64,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatRequest {
    pub chat_id: String,
    pub model_id: String,
    pub message: String,
    pub attachments: Vec<AttachmentRef>,
    #[serde(default)]
    pub system_prompt: Option<String>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentRef {
    pub name: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub data_base64: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatResponse {
    pub content: String,
    pub tokens_used: u32,
    pub latency_ms: u64,
    pub memory_recalled: u32,
    pub injection_applied: bool,
    pub model_id: String,
}

pub struct InferenceEngine {
    loaded_models: RwLock<HashMap<String, ModelInfo>>,
    client: Client,
    #[cfg(feature = "embedded-llama")]
    gguf: Mutex<Option<GgufRuntime>>,
}

pub fn models_directory() -> PathBuf {
    let mut path = crate::app_paths::app_data_dir();
    path.push("models");
    fs::create_dir_all(&path).ok();
    path
}

fn sanitize_repo(repo: &str) -> String {
    repo.replace('/', "--")
}

impl InferenceEngine {
    pub fn new() -> Self {
        let engine = Self {
            loaded_models: RwLock::new(HashMap::new()),
            client: Client::builder()
                .user_agent("Silenium/1.0")
                .timeout(std::time::Duration::from_secs(600))
                .build()
                .unwrap_or_default(),
            #[cfg(feature = "embedded-llama")]
            gguf: Mutex::new(GgufRuntime::new().ok()),
        };
        engine.scan_directory();
        engine
    }

    pub fn detect_format(path: &str) -> Option<String> {
        let ext = Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())?;
        match ext.as_str() {
            "gguf" | "ggml" | "bin" => Some("gguf".into()),
            "onnx" => Some("onnx".into()),
            "safetensors" => Some("safetensors".into()),
            "pt" | "pth" => Some("pt".into()),
            _ => None,
        }
    }

    fn verify_file(path: &Path) -> bool {
        path.exists() && fs::metadata(path).map(|m| m.len() > 1024).unwrap_or(false)
    }

    pub fn scan_directory(&self) -> Vec<ModelInfo> {
        let root = models_directory();
        let mut found = Vec::new();
        Self::walk_models(&root, &root, &mut found);
        {
            let mut map = self.loaded_models.write();
            for m in &found {
                map.insert(m.id.clone(), m.clone());
            }
        }
        found
    }

    fn walk_models(root: &Path, dir: &Path, out: &mut Vec<ModelInfo>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                Self::walk_models(root, &path, out);
            } else if let Some(fmt) = Self::detect_format(path.to_str().unwrap_or("")) {
                let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                let stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if stem.contains("mmproj") {
                    continue;
                }
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("model")
                    .to_string();
                let id = format!(
                    "local-{}",
                    path.to_string_lossy().replace(['/', '\\', ':'], "-")
                );
                out.push(ModelInfo {
                    id,
                    name,
                    path: path.to_string_lossy().to_string(),
                    format: fmt,
                    backend: "local".into(),
                    size_bytes: size,
                    loaded: true,
                    source: "local".into(),
                    verified: Self::verify_file(&path),
                    download_progress: 100,
                });
            }
        }
    }

    pub fn load_model(&self, path: &str, name: &str, require_integrity: bool) -> Result<ModelInfo, String> {
        if !Path::new(path).exists() {
            return Err(format!("Файл модели не найден: {path}"));
        }
        let format = Self::detect_format(path).ok_or("Неподдерживаемый формат модели")?;
        let size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        if size < 1024 {
            return Err("Файл модели слишком мал или повреждён".into());
        }
        let id = format!("model-{}", path.replace(['/', '\\', '.'], "-"));
        if require_integrity && !Self::verify_file(Path::new(path)) {
            return Err("Проверка целостности модели не пройдена".into());
        }
        let info = ModelInfo {
            id: id.clone(),
            name: name.to_string(),
            path: path.to_string(),
            format,
            backend: "local".into(),
            size_bytes: size,
            loaded: true,
            source: "local".into(),
            verified: true,
            download_progress: 100,
        };
        self.loaded_models.write().insert(id.clone(), info.clone());
        Ok(info)
    }

    pub async fn download_huggingface(&self, repo: &str) -> Result<DownloadResult, String> {
        let tree_url = format!("https://huggingface.co/api/models/{repo}/tree/main?recursive=1");
        let resp = self
            .client
            .get(&tree_url)
            .send()
            .await
            .map_err(|e| format!("Ошибка API Hugging Face: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!(
                "Репозиторий не найден или недоступен: HTTP {}",
                resp.status()
            ));
        }

        let files: Vec<serde_json::Value> = resp
            .json()
            .await
            .map_err(|e| format!("Некорректный ответ API: {e}"))?;

        let gguf_path = files
            .iter()
            .filter_map(|f| f.get("path").and_then(|p| p.as_str()))
            .find(|p| p.ends_with(".gguf") || p.ends_with(".GGUF"))
            .ok_or_else(|| {
                "В репозитории нет GGUF файла. Скачайте GGUF-версию модели вручную.".to_string()
            })?;

        let download_url = format!("https://huggingface.co/{repo}/resolve/main/{gguf_path}");
        let dest_dir = models_directory().join(sanitize_repo(repo));
        fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
        let dest_path = dest_dir.join(
            Path::new(gguf_path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("model.gguf"),
        );

        let bytes = self
            .client
            .get(&download_url)
            .send()
            .await
            .map_err(|e| format!("Ошибка загрузки: {e}"))?
            .bytes()
            .await
            .map_err(|e| format!("Ошибка чтения данных: {e}"))?;

        let nbytes = bytes.len() as u64;
        if nbytes < 4096 {
            return Ok(DownloadResult {
                success: false,
                model: None,
                message: format!(
                    "Загрузка не удалась: получено только {nbytes} байт. Проверьте интернет и доступ к Hugging Face."
                ),
                bytes_downloaded: nbytes,
                verified: false,
            });
        }

        fs::write(&dest_path, &bytes).map_err(|e| format!("Не удалось сохранить файл: {e}"))?;

        let verified = Self::verify_file(&dest_path);
        if !verified {
            return Ok(DownloadResult {
                success: false,
                model: None,
                message: "Файл сохранён, но проверка целостности не пройдена.".into(),
                bytes_downloaded: nbytes,
                verified: false,
            });
        }

        let model = self.load_model(dest_path.to_str().unwrap(), repo, true)?;
        Ok(DownloadResult {
            success: true,
            model: Some(model),
            message: format!(
                "Модель {repo} скачана и проверена ({:.1} MB)",
                nbytes as f64 / 1_048_576.0
            ),
            bytes_downloaded: nbytes,
            verified: true,
        })
    }

    pub fn list_models(&self) -> Vec<ModelInfo> {
        self.scan_directory();
        let mut models: Vec<_> = self.loaded_models.read().values().cloned().collect();
        if !models.iter().any(|m| m.id == "default") {
            models.insert(
                0,
                ModelInfo {
                    id: "default".into(),
                    name: "Default (встроенная)".into(),
                    path: String::new(),
                    format: "gguf".into(),
                    backend: "builtin".into(),
                    size_bytes: 0,
                    loaded: true,
                    source: "builtin".into(),
                    verified: true,
                    download_progress: 100,
                },
            );
        }
        models.sort_by(|a, b| a.name.cmp(&b.name));
        models
    }

    pub fn verify_model(&self, model_id: &str, require_integrity: bool) -> Result<bool, String> {
        let map = self.loaded_models.read();
        let m = map.get(model_id).ok_or("Модель не найдена")?;
        if m.path.is_empty() {
            return Ok(m.id == "default");
        }
        let ok = Self::verify_file(Path::new(&m.path));
        if require_integrity && !ok {
            return Err("Проверка целостности модели не пройдена".into());
        }
        Ok(ok)
    }

    /// Inference without STM/LTM side effects (for agents).
    pub fn generate_reply(
        &self,
        settings: &AppSettings,
        model_id: &str,
        system: &str,
        user_message: &str,
        agent_stream: &mut Option<AgentStreamSink>,
    ) -> Result<String, String> {
        let model_info = self.loaded_models.read().get(model_id).cloned();
        let model_path = model_info
            .as_ref()
            .map(|m| m.path.clone())
            .unwrap_or_default();
        if model_path.is_empty() {
            return Err("Модель не выбрана для агента".into());
        }
        let model_size_bytes = model_info.as_ref().map(|m| m.size_bytes).unwrap_or(0);
        let backend = synaptic_backend_pref(settings);
        let mut messages = Vec::new();
        if !system.is_empty() {
            messages.push(("system".into(), system.to_string()));
        }
        messages.push(("user".into(), user_message.to_string()));
        let mut gen = GenerateParams {
            model_path,
            messages,
            temperature: effective_temperature(settings, settings.inference.temperature),
            max_tokens: effective_max_tokens(settings, 512),
            n_ctx: settings.inference.context_length.max(2048),
            threads: settings.system.thread_count.max(1),
            gpu_layers: settings.system.gpu_layers,
            gpu_memory_mb: settings.system.gpu_memory_mb,
            vram_reserve_mb: settings.system.vram_reserve_mb,
            ram_limit_mb: settings.system.ram_limit_mb,
            mmap_enabled: settings.system.mmap_enabled,
            mlock_enabled: settings.system.mlock_enabled,
            swap_usage: settings.system.swap_usage.clone(),
            oom_policy: settings.system.oom_policy.clone(),
            kv_offload: settings.performance.kv_cache_offload,
            model_size_bytes,
            prefer_embedded: backend.prefer_embedded,
            prefer_cli: backend.prefer_cli,
        };
        gen = tune_generate_params(settings, gen);

        #[cfg(feature = "embedded-llama")]
        {
            let gguf_guard = self.gguf.lock();
            return match agent_stream.as_mut() {
                Some(sink) => {
                    generate_with_best_backend(gguf_guard.as_ref(), gen, Some(sink as &mut dyn TokenSink), None)
                }
                None => generate_with_best_backend(gguf_guard.as_ref(), gen, None, None),
            };
        }
        #[cfg(not(feature = "embedded-llama"))]
        match agent_stream.as_mut() {
            Some(sink) => generate_with_best_backend(gen, Some(sink as &mut dyn TokenSink), None),
            None => generate_with_best_backend(gen, None, None),
        }
    }

    fn build_system_prompt(
        settings: &AppSettings,
        override_cfg: Option<&crate::settings::ChatOverride>,
        ltm_enabled: bool,
        memory: &MemoryStore,
        chat_id: &str,
        user_message: &str,
        chat_system: Option<&str>,
    ) -> (String, bool) {
        let mut parts = Vec::new();
        let mut injection_applied = false;

        if let Some(sp) = chat_system {
            if !sp.is_empty() {
                parts.push(sp.to_string());
            }
        }

        if settings.global_message_injection.enabled {
            injection_applied = true;
            let inj = &settings.global_message_injection;
            if !inj.system_prefix.is_empty() {
                parts.push(inj.system_prefix.clone());
            }
            if let Some(custom) = override_cfg.and_then(|o| o.custom_injection.as_ref()) {
                parts.push(custom.clone());
            }
            if !inj.hidden_context.is_empty() {
                parts.push(inj.hidden_context.clone());
            }
            if inj.inject_memory_summary && ltm_enabled {
                let recalled = recall_ltm(memory, settings, chat_id, user_message);
                if !recalled.is_empty() {
                    let summary: String = recalled
                        .iter()
                        .map(|e| e.content.clone())
                        .collect::<Vec<_>>()
                        .join(" | ");
                    parts.push(format!("Память: {summary}"));
                }
            }
        }

        (parts.join("\n"), injection_applied)
    }

    fn attachment_note(request: &ChatRequest) -> Option<String> {
        if request.attachments.is_empty() {
            return None;
        }
        let list: String = request
            .attachments
            .iter()
            .map(|a| {
                let kind = if a.mime_type.starts_with("audio/") {
                    "аудио"
                } else if a.mime_type.starts_with("image/") {
                    "изображение"
                } else if a.mime_type.starts_with("video/") {
                    "видео"
                } else {
                    "файл"
                };
                format!("{kind}: {}", a.name)
            })
            .collect::<Vec<_>>()
            .join(", ");
        Some(format!("Вложения пользователя: {list}"))
    }

    pub fn chat(
        &self,
        settings: &AppSettings,
        memory: &MemoryStore,
        request: &ChatRequest,
        stream: &mut Option<StreamSink>,
        cancel: Option<&std::sync::atomic::AtomicBool>,
    ) -> ChatResponse {
        settings_engine::audit_log(
            settings,
            "chat",
            &format!("chat_id={} model={}", request.chat_id, request.model_id),
        );
        let start = std::time::Instant::now();
        let turn_index = memory.get_stm(&request.chat_id).len() as u32 / 2 + 1;
        let override_cfg = settings.per_chat_overrides.get(&request.chat_id);

        let stm_enabled = override_cfg
            .and_then(|o| o.stm_enabled)
            .unwrap_or(settings.memory.stm_enabled);
        let ltm_enabled = override_cfg
            .and_then(|o| o.ltm_enabled)
            .unwrap_or(settings.memory.ltm_enabled);

        let model_info = self.loaded_models.read().get(&request.model_id).cloned();
        let model_name = model_info
            .as_ref()
            .map(|m| m.name.clone())
            .unwrap_or_else(|| request.model_id.clone());
        let model_path = model_info
            .as_ref()
            .map(|m| m.path.clone())
            .unwrap_or_default();
        let model_format = model_info
            .as_ref()
            .map(|m| m.format.clone())
            .unwrap_or_else(|| "gguf".into());
        let model_size_bytes = model_info.as_ref().map(|m| m.size_bytes).unwrap_or(0);

        let user_message = match check_user_input(settings, &request.message) {
            Ok(m) => m,
            Err(e) => {
                if let Some(sink) = stream.as_mut() {
                    sink.error(e.clone());
                }
                return ChatResponse {
                    content: e,
                    tokens_used: 0,
                    latency_ms: start.elapsed().as_millis() as u64,
                    memory_recalled: 0,
                    injection_applied: false,
                    model_id: request.model_id.clone(),
                };
            }
        };

        let temp = effective_temperature(
            settings,
            request.temperature.unwrap_or(settings.inference.temperature),
        );
        let max_tok = effective_max_tokens(
            settings,
            request
                .max_tokens
                .unwrap_or(settings.inference.context_length),
        );

        let (base_system, injection_applied) = Self::build_system_prompt(
            settings,
            override_cfg,
            ltm_enabled,
            memory,
            &request.chat_id,
            &user_message,
            request.system_prompt.as_deref(),
        );

        let stm_snapshot = if stm_enabled {
            filter_stm(settings, memory.get_stm(&request.chat_id))
        } else {
            Vec::new()
        };

        let system_prompt = enrich_system_prompt(
            settings,
            &base_system,
            &user_message,
            &stm_snapshot,
            turn_index,
        );

        let mut user_turn = user_message.clone();
        if let Some(note) = cross_modal_user_note(settings, Self::attachment_note(request)) {
            user_turn = format!("{user_turn}\n[{note}]");
        }

        let mut messages: Vec<(String, String)> = Vec::new();
        if !system_prompt.is_empty() {
            messages.push(("system".into(), system_prompt));
        }

        if stm_enabled {
            for entry in &stm_snapshot {
                if is_legacy_stub(&entry.content) {
                    continue;
                }
                let role = if entry.role == "assistant" {
                    "assistant"
                } else {
                    "user"
                };
                messages.push((role.into(), entry.content.clone()));
            }
        }

        messages.push(("user".into(), user_turn.clone()));

        let response_content = if model_path.is_empty() || request.model_id == "default" {
            "Выберите локальную GGUF-модель в свойствах чата (панель слева) или загрузите её в разделе «Модели».".into()
        } else if model_format != "gguf" && model_format != "ggml" {
            format!(
                "Формат «{model_format}» пока не поддерживается для вывода. Используйте файл .gguf."
            )
        } else {
            let backend = synaptic_backend_pref(settings);
            let mut gen = GenerateParams {
                model_path: model_path.clone(),
                messages,
                temperature: temp,
                max_tokens: max_tok,
                n_ctx: settings.inference.context_length.max(2048),
                threads: settings.system.thread_count.max(1),
                gpu_layers: settings.system.gpu_layers,
                gpu_memory_mb: settings.system.gpu_memory_mb,
                vram_reserve_mb: settings.system.vram_reserve_mb,
                ram_limit_mb: settings.system.ram_limit_mb,
                mmap_enabled: settings.system.mmap_enabled,
                mlock_enabled: settings.system.mlock_enabled,
                swap_usage: settings.system.swap_usage.clone(),
                oom_policy: settings.system.oom_policy.clone(),
                kv_offload: settings.performance.kv_cache_offload,
                model_size_bytes,
                prefer_embedded: backend.prefer_embedded,
                prefer_cli: backend.prefer_cli,
            };
            gen = tune_generate_params(settings, gen);
            #[cfg(feature = "embedded-llama")]
            let inference_result = {
                let gguf_guard = self.gguf.lock();
                match stream.as_mut() {
                    Some(sink) => generate_with_best_backend(
                        gguf_guard.as_ref(),
                        gen,
                        Some(sink as &mut dyn TokenSink),
                        cancel,
                    ),
                    None => generate_with_best_backend(gguf_guard.as_ref(), gen, None, cancel),
                }
            };
            #[cfg(not(feature = "embedded-llama"))]
            let inference_result = match stream.as_mut() {
                Some(sink) => generate_with_best_backend(gen, Some(sink as &mut dyn TokenSink)),
                None => generate_with_best_backend(gen, None),
            };
            match inference_result {
                Ok(text) => filter_model_output(settings, &text),
                Err(err) => format!("Ошибка инференса ({model_name}): {err}"),
            }
        };

        maybe_dream_consolidate(memory, settings, &request.chat_id, &request.model_id);

        if stm_enabled {
            memory.add_stm(
                &request.chat_id,
                "user",
                &user_message,
                settings.memory.stm_max_tokens,
            );
            memory.add_stm(
                &request.chat_id,
                "assistant",
                &response_content,
                settings.memory.stm_max_tokens,
            );
        }
        if ltm_enabled {
            let importance = settings_engine::neuroplastic_importance(settings, 0.5, turn_index);
            memory.add_ltm(
                &request.chat_id,
                &request.model_id,
                &user_message,
                importance,
                vec!["conversation".into()],
                true,
                None,
            );
        }

        tracing::info!(
            model = %model_name,
            path = %model_path,
            temp = temp,
            max_tokens = max_tok,
            latency_ms = start.elapsed().as_millis(),
            "chat inference"
        );

        let latency_ms = start.elapsed().as_millis() as u64;
        let tokens_used = (user_message.len() + response_content.len()) as u32 / 4;
        if let Some(sink) = stream.as_mut() {
            sink.finish(tokens_used, latency_ms);
        }
        ChatResponse {
            content: response_content,
            tokens_used,
            latency_ms,
            memory_recalled: if ltm_enabled {
                recall_ltm(memory, settings, &request.chat_id, &user_message).len() as u32
            } else {
                0
            },
            injection_applied,
            model_id: request.model_id.clone(),
        }
    }
}
