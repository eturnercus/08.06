use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatRequest {
    pub chat_id: String,
    pub model_id: String,
    pub message: String,
    pub attachments: Vec<AttachmentRef>,
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
}

pub struct InferenceEngine {
    loaded_models: RwLock<HashMap<String, ModelInfo>>,
}

impl InferenceEngine {
    pub fn new() -> Self {
        Self {
            loaded_models: RwLock::new(HashMap::new()),
        }
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

    pub fn load_model(&self, path: &str, name: &str) -> Result<ModelInfo, String> {
        if !Path::new(path).exists() {
            return Err(format!("Файл модели не найден: {path}"));
        }
        let format = Self::detect_format(path).ok_or("Неподдерживаемый формат модели")?;
        let size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
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
        };
        self.loaded_models.write().insert(id.clone(), info.clone());
        Ok(info)
    }

    pub fn load_huggingface(&self, repo: &str) -> Result<ModelInfo, String> {
        let id = format!("hf-{}", repo.replace('/', "-"));
        let info = ModelInfo {
            id: id.clone(),
            name: repo.to_string(),
            path: format!("~/.cache/huggingface/hub/models--{}", repo.replace('/', "--")),
            format: "gguf".into(),
            backend: "huggingface".into(),
            size_bytes: 0,
            loaded: true,
            source: "huggingface".into(),
        };
        self.loaded_models.write().insert(id, info.clone());
        Ok(info)
    }

    pub fn list_models(&self) -> Vec<ModelInfo> {
        self.loaded_models.read().values().cloned().collect()
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

        let mut injection_applied = false;
        let mut full_message = request.message.clone();

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
            .map(|a| format!("{} ({}, {}B)", a.name, a.mime_type, a.size_bytes))
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

        let response_content = format!(
            "NeuroForge [{}] ({})\n\
             Формат: локальный inference\n\
             RAM: {}MB | Потоки: {} | Контекст: {}\n\
             Вложения: {}\n\
             STM: {} | LTM: {}\n\
             ---\n\
             Ответ на: \"{}\"\n\
             Контекст STM ({} сообщ.): {}",
            request.model_id,
            settings.inference.default_backend,
            settings.system.ram_limit_mb,
            settings.system.thread_count,
            settings.inference.context_length,
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
        }
    }
}
