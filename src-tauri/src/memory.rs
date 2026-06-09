use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryEntry {
    pub id: String,
    pub chat_id: String,
    pub model_id: String,
    pub memory_type: String,
    pub content: String,
    pub importance: f32,
    pub tags: Vec<String>,
    pub embedding_stub: Vec<f32>,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub access_count: u32,
    pub transferable: bool,
    pub source_agent_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryTransferRequest {
    pub id: String,
    pub entry_ids: Vec<String>,
    pub from_chat_id: String,
    pub to_chat_id: String,
    pub from_model_id: String,
    pub to_model_id: String,
    pub memory_type: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StmBuffer {
    pub chat_id: String,
    pub entries: Vec<StmEntry>,
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StmEntry {
    pub role: String,
    pub content: String,
    pub tokens: u32,
    pub timestamp: DateTime<Utc>,
}

pub struct MemoryStore {
    ltm: RwLock<HashMap<String, MemoryEntry>>,
    stm: RwLock<HashMap<String, StmBuffer>>,
    transfers: RwLock<Vec<MemoryTransferRequest>>,
    pools: RwLock<HashMap<String, Vec<String>>>,
    encrypt_at_rest: RwLock<bool>,
}

impl MemoryStore {
    pub fn new() -> Self {
        let store = Self {
            ltm: RwLock::new(HashMap::new()),
            stm: RwLock::new(HashMap::new()),
            transfers: RwLock::new(Vec::new()),
            pools: RwLock::new(HashMap::new()),
            encrypt_at_rest: RwLock::new(false),
        };
        store.load_from_disk();
        store
    }

    fn data_dir() -> PathBuf {
        let mut path = crate::app_paths::app_data_dir();
        path.push("memory");
        fs::create_dir_all(&path).ok();
        path
    }

    pub fn set_encrypt_at_rest(&self, enabled: bool) {
        *self.encrypt_at_rest.write() = enabled;
    }

    fn load_from_disk(&self) {
        let ltm_path = Self::data_dir().join("ltm.json");
        if ltm_path.exists() {
            if let Ok(raw) = fs::read_to_string(&ltm_path) {
                let data = if raw.trim_start().starts_with('{') {
                    raw
                } else {
                    crate::storage_crypto::decrypt_at_rest(&raw, true)
                };
                if let Ok(entries) = serde_json::from_str::<HashMap<String, MemoryEntry>>(&data) {
                    *self.ltm.write() = entries;
                }
            }
        }
    }

    pub fn persist(&self) -> Result<(), String> {
        let ltm_path = Self::data_dir().join("ltm.json");
        let json = serde_json::to_string_pretty(&*self.ltm.read()).map_err(|e| e.to_string())?;
        let encrypt = *self.encrypt_at_rest.read();
        let data = crate::storage_crypto::encrypt_at_rest(&json, encrypt);
        fs::write(ltm_path, data).map_err(|e| e.to_string())
    }

    pub fn touch_ltm(&self, id: &str) {
        if let Some(entry) = self.ltm.write().get_mut(id) {
            entry.last_accessed = Utc::now();
            entry.access_count = entry.access_count.saturating_add(1);
        }
    }

    pub fn add_stm(&self, chat_id: &str, role: &str, content: &str, max_tokens: u32) {
        let tokens = (content.len() as u32) / 4;
        let mut stm = self.stm.write();
        let buffer = stm.entry(chat_id.to_string()).or_insert_with(|| StmBuffer {
            chat_id: chat_id.to_string(),
            entries: Vec::new(),
            max_tokens,
        });
        buffer.max_tokens = max_tokens;
        buffer.entries.push(StmEntry {
            role: role.to_string(),
            content: content.to_string(),
            tokens,
            timestamp: Utc::now(),
        });
        let mut total: u32 = buffer.entries.iter().map(|e| e.tokens).sum();
        while total > max_tokens && !buffer.entries.is_empty() {
            let removed = buffer.entries.remove(0);
            total -= removed.tokens;
        }
    }

    pub fn get_stm(&self, chat_id: &str) -> Vec<StmEntry> {
        self.stm
            .read()
            .get(chat_id)
            .map(|b| b.entries.clone())
            .unwrap_or_default()
    }

    pub fn add_ltm(
        &self,
        chat_id: &str,
        model_id: &str,
        content: &str,
        importance: f32,
        tags: Vec<String>,
        transferable: bool,
        agent_id: Option<String>,
    ) -> MemoryEntry {
        let entry = MemoryEntry {
            id: Uuid::new_v4().to_string(),
            chat_id: chat_id.to_string(),
            model_id: model_id.to_string(),
            memory_type: "long_term".into(),
            content: content.to_string(),
            importance,
            tags,
            embedding_stub: simple_embedding(content),
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 0,
            transferable,
            source_agent_id: agent_id,
        };
        self.ltm.write().insert(entry.id.clone(), entry.clone());
        let _ = self.persist();
        entry
    }

    pub fn recall_ltm(&self, chat_id: &str, query: &str, top_k: u32) -> Vec<MemoryEntry> {
        let query_emb = simple_embedding(query);
        let mut entries: Vec<MemoryEntry> = self
            .ltm
            .read()
            .values()
            .filter(|e| e.chat_id == chat_id || e.transferable)
            .cloned()
            .collect();
        entries.sort_by(|a, b| {
            let sim_a = cosine_similarity(&query_emb, &a.embedding_stub);
            let sim_b = cosine_similarity(&query_emb, &b.embedding_stub);
            sim_b
                .partial_cmp(&sim_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        entries.truncate(top_k as usize);
        entries
    }

    pub fn transfer_memory(
        &self,
        entry_ids: Vec<String>,
        from_chat: &str,
        to_chat: &str,
        from_model: &str,
        to_model: &str,
        memory_type: &str,
    ) -> MemoryTransferRequest {
        let mut ltm = self.ltm.write();
        for id in &entry_ids {
            if let Some(entry) = ltm.get_mut(id) {
                if entry.transferable {
                    entry.chat_id = to_chat.to_string();
                    entry.model_id = to_model.to_string();
                    entry.last_accessed = Utc::now();
                }
            }
        }
        let _ = self.persist();

        let request = MemoryTransferRequest {
            id: Uuid::new_v4().to_string(),
            entry_ids,
            from_chat_id: from_chat.to_string(),
            to_chat_id: to_chat.to_string(),
            from_model_id: from_model.to_string(),
            to_model_id: to_model.to_string(),
            memory_type: memory_type.to_string(),
            status: "completed".into(),
            created_at: Utc::now(),
        };
        self.transfers.write().push(request.clone());
        request
    }

    pub fn create_shared_pool(&self, pool_id: &str, entry_ids: Vec<String>) {
        self.pools.write().insert(pool_id.to_string(), entry_ids);
    }

    pub fn get_transfers(&self) -> Vec<MemoryTransferRequest> {
        self.transfers.read().clone()
    }

    pub fn get_all_ltm(&self, chat_id: Option<&str>) -> Vec<MemoryEntry> {
        self.ltm
            .read()
            .values()
            .filter(|e| chat_id.map(|c| c == e.chat_id).unwrap_or(true))
            .cloned()
            .collect()
    }

    pub fn consolidate_stm_to_ltm(&self, chat_id: &str, model_id: &str) -> Option<MemoryEntry> {
        let stm_entries = self.get_stm(chat_id);
        if stm_entries.is_empty() {
            return None;
        }
        let summary: String = stm_entries
            .iter()
            .map(|e| format!("[{}] {}", e.role, e.content))
            .collect::<Vec<_>>()
            .join("\n");
        Some(self.add_ltm(
            chat_id,
            model_id,
            &summary,
            0.8,
            vec!["consolidated".into()],
            true,
            None,
        ))
    }
}

fn simple_embedding(text: &str) -> Vec<f32> {
    let mut emb = vec![0.0f32; 64];
    for (i, byte) in text.bytes().enumerate() {
        emb[i % 64] += (byte as f32) / 255.0;
    }
    let norm: f32 = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for v in &mut emb {
            *v /= norm;
        }
    }
    emb
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na > 0.0 && nb > 0.0 {
        dot / (na * nb)
    } else {
        0.0
    }
}
