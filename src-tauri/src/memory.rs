use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
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
pub struct MemoryBridgeResult {
    pub bridge_id: String,
    pub stm_messages_bridged: u32,
    pub summary_chars: u32,
    pub from_model_id: String,
    pub to_model_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryOverview {
    pub stm_count: u32,
    pub ltm_count: u32,
    pub bridge_count: u32,
    pub cross_model_ready: bool,
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
    ltm_dirty: AtomicBool,
}

impl MemoryStore {
    pub fn new() -> Self {
        let store = Self {
            ltm: RwLock::new(HashMap::new()),
            stm: RwLock::new(HashMap::new()),
            transfers: RwLock::new(Vec::new()),
            pools: RwLock::new(HashMap::new()),
            encrypt_at_rest: RwLock::new(false),
            ltm_dirty: AtomicBool::new(false),
        };
        store.load_from_disk();
        store
    }

    fn data_dir() -> PathBuf {
        let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("neuroforge");
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

    fn mark_ltm_dirty(&self) {
        self.ltm_dirty.store(true, Ordering::Release);
    }

    /// Writes LTM to disk only when entries changed — avoids blocking the UI on every token.
    pub fn flush_ltm_if_dirty(&self) -> Result<(), String> {
        if !self.ltm_dirty.load(Ordering::Acquire) {
            return Ok(());
        }
        let ltm_path = Self::data_dir().join("ltm.json");
        let json = serde_json::to_string_pretty(&*self.ltm.read()).map_err(|e| e.to_string())?;
        let encrypt = *self.encrypt_at_rest.read();
        let data = crate::storage_crypto::encrypt_at_rest(&json, encrypt);
        fs::write(ltm_path, data).map_err(|e| e.to_string())?;
        self.ltm_dirty.store(false, Ordering::Release);
        Ok(())
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
        self.mark_ltm_dirty();
        entry
    }

    pub fn recall_ltm(
        &self,
        chat_id: &str,
        model_id: Option<&str>,
        query: &str,
        top_k: u32,
        cross_model: bool,
    ) -> Vec<MemoryEntry> {
        let query_emb = simple_embedding(query);
        let pool_ids: Vec<String> = self
            .pools
            .read()
            .get(&format!("chat:{chat_id}"))
            .cloned()
            .unwrap_or_default();

        let mut entries: Vec<MemoryEntry> = self
            .ltm
            .read()
            .values()
            .filter(|e| entry_visible(e, chat_id, model_id, cross_model, &pool_ids))
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

    /// Automatic handoff when the user picks another model in the same chat.
    /// Compresses recent STM into one LTM «bridge» card — no manual transfer button.
    pub fn synaptic_bridge_on_model_switch(
        &self,
        chat_id: &str,
        from_model: &str,
        to_model: &str,
    ) -> Option<MemoryBridgeResult> {
        if from_model == to_model {
            return None;
        }
        let stm = self.get_stm(chat_id);
        if stm.is_empty() {
            return None;
        }

        let recent: Vec<&StmEntry> = stm.iter().rev().take(10).collect();
        let summary: String = recent
            .iter()
            .rev()
            .map(|e| {
                let snippet: String = e.content.chars().take(140).collect();
                format!("[{}] {snippet}", e.role)
            })
            .collect::<Vec<_>>()
            .join("\n");
        let summary = truncate_chars(&summary, 900);

        let entry = self.add_ltm(
            chat_id,
            to_model,
            &format!("Синаптический мост ({from_model} → {to_model}): {summary}"),
            0.92,
            vec![
                "synaptic_bridge".into(),
                format!("from:{from_model}"),
                format!("to:{to_model}"),
            ],
            true,
            None,
        );

        let pool_id = format!("chat:{chat_id}");
        self.pools
            .write()
            .entry(pool_id)
            .or_default()
            .push(entry.id.clone());

        let _ = self.flush_ltm_if_dirty();

        Some(MemoryBridgeResult {
            bridge_id: entry.id,
            stm_messages_bridged: recent.len() as u32,
            summary_chars: summary.chars().count() as u32,
            from_model_id: from_model.to_string(),
            to_model_id: to_model.to_string(),
        })
    }

    pub fn memory_overview(&self, chat_id: &str) -> MemoryOverview {
        let stm_count = self.get_stm(chat_id).len() as u32;
        let ltm: Vec<MemoryEntry> = self.get_all_ltm(Some(chat_id));
        let bridge_count = ltm
            .iter()
            .filter(|e| e.tags.iter().any(|t| t == "synaptic_bridge"))
            .count() as u32;
        MemoryOverview {
            stm_count,
            ltm_count: ltm.len() as u32,
            bridge_count,
            cross_model_ready: bridge_count > 0 || ltm.iter().any(|e| e.transferable),
        }
    }

    /// Legacy explicit transfer — copies facts into target chat (does not move originals).
    pub fn transfer_memory(
        &self,
        entry_ids: Vec<String>,
        from_chat: &str,
        to_chat: &str,
        from_model: &str,
        to_model: &str,
        memory_type: &str,
    ) -> MemoryTransferRequest {
        let mut copied = Vec::new();
        {
            let ltm = self.ltm.read();
            for id in &entry_ids {
                let Some(src) = ltm.get(id) else {
                    continue;
                };
                if !src.transferable || src.chat_id != from_chat {
                    continue;
                }
                let clone = MemoryEntry {
                    id: Uuid::new_v4().to_string(),
                    chat_id: to_chat.to_string(),
                    model_id: to_model.to_string(),
                    memory_type: src.memory_type.clone(),
                    content: src.content.clone(),
                    importance: src.importance,
                    tags: {
                        let mut t = src.tags.clone();
                        t.push(format!("cloned_from:{id}"));
                        t
                    },
                    embedding_stub: src.embedding_stub.clone(),
                    created_at: Utc::now(),
                    last_accessed: Utc::now(),
                    access_count: 0,
                    transferable: true,
                    source_agent_id: src.source_agent_id.clone(),
                };
                copied.push(clone);
            }
        }
        if !copied.is_empty() {
            let mut ltm = self.ltm.write();
            for entry in copied {
                ltm.insert(entry.id.clone(), entry);
            }
            self.mark_ltm_dirty();
        }
        let _ = self.flush_ltm_if_dirty();

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
        let summary = truncate_chars(&summary, 4000);
        let entry = self.add_ltm(
            chat_id,
            model_id,
            &summary,
            0.8,
            vec!["consolidated".into()],
            true,
            None,
        );
        if let Some(buffer) = self.stm.write().get_mut(chat_id) {
            buffer.entries.clear();
        }
        let _ = self.flush_ltm_if_dirty();
        Some(entry)
    }
}

fn entry_visible(
    e: &MemoryEntry,
    chat_id: &str,
    model_id: Option<&str>,
    cross_model: bool,
    pool_ids: &[String],
) -> bool {
    if e.chat_id == chat_id || pool_ids.contains(&e.id) {
        return true;
    }
    if !e.transferable {
        return false;
    }
    if !cross_model {
        return false;
    }
    let Some(mid) = model_id else {
        return true;
    };
    if e.model_id == mid {
        return true;
    }
    e.tags
        .iter()
        .any(|t| t == &format!("to:{mid}") || t == "synaptic_bridge")
}

fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max).collect::<String>())
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

#[cfg(test)]
mod bridge_tests {
    use super::*;

    #[test]
    fn bridge_creates_ltm_entry() {
        let store = MemoryStore::new();
        store.add_stm("c1", "user", "hello world", 512);
        store.add_stm("c1", "assistant", "hi there", 512);
        let r = store
            .synaptic_bridge_on_model_switch("c1", "model-a", "model-b")
            .expect("bridge");
        assert_eq!(r.from_model_id, "model-a");
        assert!(r.stm_messages_bridged >= 2);
        let ltm = store.get_all_ltm(Some("c1"));
        assert!(ltm.iter().any(|e| e.tags.contains(&"synaptic_bridge".to_string())));
    }

    #[test]
    fn deferred_persist_skips_when_clean() {
        let store = MemoryStore::new();
        assert!(store.flush_ltm_if_dirty().is_ok());
    }
}
