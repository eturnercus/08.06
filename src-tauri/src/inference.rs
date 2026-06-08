use parking_lot::RwLock;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::memory::MemoryStore;
use crate::settings::AppSettings;

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
}

pub fn models_directory() -> PathBuf {
    let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("neuroforge");
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
                .user_agent("NeuroForge/1.0")
                .timeout(std::time::Duration::from_secs(600))
                .build()
                .unwrap_or_default(),
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

    pub fn load_model(&self, path: &str, name: &str) -> Result<ModelInfo, String> {
        if !Path::new(path).exists() {
            return Err(format!("Файл модели не найден: {path}"));
        }
        let format = Self::detect_format(path).ok_or("Неподдерживаемый формат модели")?;
        let size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        if size < 1024 {
            return Err("Файл модели слишком мал или повреждён".into());
        }
        let id = format!("model-{}", path.replace(['/', '\\', '.'], "-"));
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

        let model = self.load_model(dest_path.to_str().unwrap(), repo)?;
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

    pub fn verify_model(&self, model_id: &str) -> Result<bool, String> {
        let map = self.loaded_models.read();
        let m = map.get(model_id).ok_or("Модель не найдена")?;
        if m.path.is_empty() {
            return Ok(m.id == "default");
        }
        Ok(Self::verify_file(Path::new(&m.path)))
    }

    pub fn chat(
        &self,
        settings: &AppSettings,
        memory: &MemoryStore,
        request: &ChatRequest,
    ) -> ChatResponse {
        let start = std::time::Instant::now();
        let override_cfg = settings.per_chat_overrides.get(&request.chat_id);

        let stm_enabled = override_cfg
            .and_then(|o| o.stm_enabled)
            .unwrap_or(settings.memory.stm_enabled);
        let ltm_enabled = override_cfg
            .and_then(|o| o.ltm_enabled)
            .unwrap_or(settings.memory.ltm_enabled);

        let model_name = self
            .loaded_models
            .read()
            .get(&request.model_id)
            .map(|m| m.name.clone())
            .unwrap_or_else(|| request.model_id.clone());

        let mut injection_applied = false;
        let mut full_message = request.message.clone();

        if let Some(sp) = &request.system_prompt {
            if !sp.is_empty() {
                full_message = format!("[system] {sp}\n{full_message}");
            }
        }

        if settings.global_message_injection.enabled {
            injection_applied = true;
            let inj = &settings.global_message_injection;
            if !inj.system_prefix.is_empty() {
                full_message = format!("{}\n{}", inj.system_prefix, full_message);
            }
            if let Some(custom) = override_cfg.and_then(|o| o.custom_injection.as_ref()) {
                full_message = format!("{}\n{}", custom, full_message);
            }
            if !inj.hidden_context.is_empty() {
                full_message = format!("[ctx] {}\n{}", inj.hidden_context, full_message);
            }
            if inj.inject_memory_summary && ltm_enabled {
                let recalled = memory.recall_ltm(
                    &request.chat_id,
                    &request.message,
                    settings.memory.recall_top_k,
                );
                if !recalled.is_empty() {
                    let summary: String = recalled
                        .iter()
                        .map(|e| e.content.clone())
                        .collect::<Vec<_>>()
                        .join(" | ");
                    full_message = format!("[memory] {}\n{}", summary, full_message);
                }
            }
        }

        if stm_enabled {
            memory.add_stm(
                &request.chat_id,
                "user",
                &full_message,
                settings.memory.stm_max_tokens,
            );
        }

        let attachment_info: String = request
            .attachments
            .iter()
            .map(|a| {
                let media = if a.mime_type.starts_with("audio/") {
                    "🎤 аудио"
                } else if a.mime_type.starts_with("image/") {
                    "📷 изображение"
                } else if a.mime_type.starts_with("video/") {
                    "🎬 видео"
                } else {
                    "📎 файл"
                };
                format!("{media}: {} ({}B)", a.name, a.size_bytes)
            })
            .collect::<Vec<_>>()
            .join(", ");

        let stm_context = if stm_enabled {
            memory
                .get_stm(&request.chat_id)
                .iter()
                .map(|e| format!("[{}] {}", e.role, e.content))
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            String::new()
        };

        let temp = request.temperature.unwrap_or(settings.inference.temperature);
        let max_tok = request.max_tokens.unwrap_or(settings.inference.context_length);

        let model_path = self
            .loaded_models
            .read()
            .get(&request.model_id)
            .map(|m| m.path.clone())
            .unwrap_or_default();

        let response_content = format!(
            "NeuroForge | Модель: {model_name}\n\
             Путь: {}\n\
             Backend: {} | temp: {temp:.2} | max_tokens: {max_tok}\n\
             RAM: {}MB | Потоки: {}\n\
             Вложения: {}\n\
             STM: {} | LTM: {}\n\
             ---\n\
             Ответ на: \"{}\"\n\
             Контекст STM ({} сообщ.): {}",
            if model_path.is_empty() {
                "встроенная".into()
            } else {
                model_path
            },
            settings.inference.default_backend,
            settings.system.ram_limit_mb,
            settings.system.thread_count,
            if attachment_info.is_empty() {
                "нет".into()
            } else {
                attachment_info
            },
            if stm_enabled { "вкл" } else { "выкл" },
            if ltm_enabled { "вкл" } else { "выкл" },
            request.message.chars().take(200).collect::<String>(),
            memory.get_stm(&request.chat_id).len(),
            stm_context.chars().take(300).collect::<String>()
        );

        if stm_enabled {
            memory.add_stm(
                &request.chat_id,
                "assistant",
                &response_content,
                settings.memory.stm_max_tokens,
            );
        }
        if ltm_enabled {
            memory.add_ltm(
                &request.chat_id,
                &request.model_id,
                &request.message,
                0.5,
                vec!["conversation".into()],
                true,
                None,
            );
        }

        let tokens_used = (full_message.len() + response_content.len()) as u32 / 4;
        ChatResponse {
            content: response_content,
            tokens_used,
            latency_ms: start.elapsed().as_millis() as u64,
            memory_recalled: if ltm_enabled {
                memory
                    .recall_ltm(&request.chat_id, &request.message, settings.memory.recall_top_k)
                    .len() as u32
            } else {
                0
            },
            injection_applied,
            model_id: request.model_id.clone(),
        }
    }
}
