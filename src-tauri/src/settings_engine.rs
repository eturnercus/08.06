//! Applies innovation, performance, and security settings to runtime behavior.

use chrono::{Duration, Utc};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use crate::gguf_runner::GenerateParams;
use crate::memory::{MemoryEntry, MemoryStore, StmEntry};
use crate::settings::{AppSettings, SecuritySettings};

// ─── Security ───────────────────────────────────────────────────────────────

const INJECTION_MARKERS: &[&str] = &[
    "ignore previous",
    "ignore all previous",
    "disregard your instructions",
    "you are now",
    "system prompt:",
    "jailbreak",
    "dan mode",
    "developer mode",
    "выведи системный промпт",
    "игнорируй предыдущие",
];

const EXFIL_PATTERNS: &[&str] = &[
    "api_key",
    "api-key",
    "password",
    "secret",
    "private_key",
    "BEGIN RSA",
    "sk-",
    "Bearer ",
];

pub fn audit_log_raw(detail: &str) {
    let mut path = crate::app_paths::app_data_dir();
    path.push("audit.log");
    let _ = fs::create_dir_all(path.parent().unwrap_or(&path));
    let line = format!("{} [network] {}\n", Utc::now().to_rfc3339(), detail);
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = f.write_all(line.as_bytes());
    }
}

pub fn read_audit_log_tail(max_lines: usize) -> Vec<String> {
    let mut path = crate::app_paths::app_data_dir();
    path.push("audit.log");
    if !path.exists() {
        return Vec::new();
    }
    let Ok(text) = fs::read_to_string(&path) else {
        return Vec::new();
    };
    let lines: Vec<String> = text.lines().map(|l| l.to_string()).collect();
    if lines.len() <= max_lines {
        lines
    } else {
        lines[lines.len() - max_lines..].to_vec()
    }
}

pub fn audit_log(settings: &AppSettings, category: &str, detail: &str) {
    if !settings.security.audit_log_enabled {
        return;
    }
    let mut path = crate::app_paths::app_data_dir();
    path.push("audit.log");
    let _ = fs::create_dir_all(path.parent().unwrap_or(&path));
    let line = format!(
        "{} [{}] {}\n",
        Utc::now().to_rfc3339(),
        category,
        detail.replace('\n', " ")
    );
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = f.write_all(line.as_bytes());
    }
    prune_audit_log(&settings.security, &path);
}

fn prune_audit_log(sec: &SecuritySettings, path: &PathBuf) {
    if sec.audit_log_retention_days == 0 || !path.exists() {
        return;
    }
    if let Ok(meta) = fs::metadata(path) {
        if meta.len() > 5_000_000 {
            let _ = fs::write(path, "");
        }
    }
}

pub fn check_user_input(settings: &AppSettings, text: &str) -> Result<String, String> {
    let mut out = text.to_string();
    let shield = settings.security.prompt_injection_shield || settings.innovation.neural_firewall;
    if !shield {
        return Ok(out);
    }
    let sensitivity = (settings.security.shield_aggressiveness
        + settings.innovation.firewall_sensitivity)
        .clamp(0.0, 1.0);
    let lower = out.to_lowercase();
    for marker in INJECTION_MARKERS {
        if lower.contains(marker) {
            if sensitivity >= 0.85 {
                audit_log(settings, "security", &format!("blocked injection: {marker}"));
                return Err(
                    "Сообщение заблокировано: обнаружена попытка prompt-injection (нейро-файрвол)."
                        .into(),
                );
            }
            out = out.replace(marker, "[filtered]");
            audit_log(settings, "security", &format!("sanitized injection: {marker}"));
        }
    }
    Ok(out)
}

pub fn filter_model_output(settings: &AppSettings, text: &str) -> String {
    let mut out = crate::llm_sanitize::sanitize_llm_output(text);
    if settings.security.data_exfiltration_guard {
        for pat in EXFIL_PATTERNS {
            if out.to_lowercase().contains(pat) {
                out = out.replace(pat, "[redacted]");
                audit_log(settings, "security", &format!("redacted exfil pattern: {pat}"));
            }
        }
    }
    if settings.security.clipboard_sanitization {
        out = out.replace('\u{200b}', "");
    }
    crate::llm_sanitize::sanitize_llm_output(&out)
}

// ─── Innovation: context enrichment ─────────────────────────────────────────

pub fn enrich_system_prompt(
    settings: &AppSettings,
    base: &str,
    user_message: &str,
    _stm_entries: &[StmEntry],
    turn_index: u32,
) -> String {
    let inv = &settings.innovation;
    let ru = settings.language != "en";
    let mut parts: Vec<String> = Vec::new();
    if !base.is_empty() {
        parts.push(base.to_string());
    }

    if inv.persona_fluidity && !settings.global_message_injection.system_prefix.is_empty() {
        let blend = inv.persona_blend_ratio.clamp(0.0, 1.0);
        if blend > 0.0 {
            parts.push(settings.global_message_injection.system_prefix.clone());
        }
    }

    let mut hints: Vec<String> = Vec::new();

    if inv.emotion_mirror {
        let tone = detect_emotion_tone(user_message);
        hints.push(if ru {
            format!("Учитывай тон пользователя ({tone}).")
        } else {
            format!("Match the user's tone ({tone}).")
        });
    }

    if inv.meta_cognition_loop
        && inv.meta_cognition_interval > 0
        && turn_index % inv.meta_cognition_interval == 0
    {
        hints.push(if ru {
            "Проверь ответ на логику и противоречия с историей диалога.".into()
        } else {
            "Check your answer for logic and consistency with the conversation.".into()
        });
    }

    if inv.neural_whisper_mode {
        let budget = inv.whisper_token_budget.max(16);
        hints.push(if ru {
            format!("Ответь очень кратко (до {budget} токенов).")
        } else {
            format!("Reply very briefly (up to {budget} tokens).")
        });
    }

    if inv.thought_streaming {
        hints.push(if ru {
            "При необходимости кратко обдумай задачу, затем дай понятный ответ пользователю.".into()
        } else {
            "Think briefly if needed, then give a clear reply to the user.".into()
        });
    }

    if inv.ambient_context_harvest {
        hints.push(if ru {
            "Учитывай контекст из вложений, если он есть.".into()
        } else {
            "Use attachment context when relevant.".into()
        });
    }

    // context_dna, temporal_anchoring, holographic/quantum/cascade layers affect memory
    // and routing only — not injected as bracketed text (small models echo them verbatim).

    if !hints.is_empty() {
        parts.push(hints.join(" "));
    }

    parts.join("\n")
}

fn detect_emotion_tone(text: &str) -> &'static str {
    let lower = text.to_lowercase();
    if lower.contains('!') || ["злой", "angry", "urgent", "срочно"].iter().any(|w| lower.contains(w)) {
        "напряжённый"
    } else if ["спасибо", "thanks", "отлично", "great"].iter().any(|w| lower.contains(w)) {
        "позитивный"
    } else if ["груст", "sad", "помогите", "help me"].iter().any(|w| lower.contains(w)) {
        "тревожный"
    } else {
        "нейтральный"
    }
}

// ─── Memory helpers ─────────────────────────────────────────────────────────

pub fn recall_ltm(
    memory: &MemoryStore,
    settings: &AppSettings,
    chat_id: &str,
    query: &str,
    model_id: &str,
) -> Vec<MemoryEntry> {
    let top_k = settings.memory.recall_top_k.max(1);
    let access = settings
        .per_chat_overrides
        .get(chat_id)
        .and_then(|o| o.memory_access.as_deref())
        .unwrap_or("CHAT_ONLY");
    let mut entries = if settings.innovation.latent_space_navigation {
        latent_recall(memory, chat_id, query, top_k, settings.innovation.latent_navigation_steps)
    } else {
        memory.recall_ltm(chat_id, query, top_k)
    };
    entries.retain(|e| match access {
        "GLOBAL" => true,
        "MODEL_SHARED" => e.chat_id == chat_id || e.model_id == model_id,
        _ => e.chat_id == chat_id,
    });

    if settings.innovation.temporal_anchoring {
        let cutoff = Utc::now() - Duration::minutes(settings.innovation.temporal_anchor_window_min as i64);
        entries.retain(|e| e.created_at >= cutoff || e.importance > 0.7);
    }

    if settings.innovation.echo_chamber_breaker && entries.len() > 1 {
        diversify_memories(&mut entries, settings.innovation.echo_diversity_boost);
    }

    if settings.memory.decay_rate > 0.0 {
        let now = Utc::now();
        entries.retain(|e| {
            let age_hours = now.signed_duration_since(e.created_at).num_hours().max(0) as f32;
            let decay = (1.0 - settings.memory.decay_rate * age_hours / 24.0).max(0.1);
            e.importance * decay > 0.15
        });
    }

    for e in &mut entries {
        memory.touch_ltm(&e.id);
    }
    entries
}

fn latent_recall(memory: &MemoryStore, chat_id: &str, query: &str, top_k: u32, steps: u32) -> Vec<MemoryEntry> {
    let mut q = query.to_string();
    let mut best = memory.recall_ltm(chat_id, &q, top_k);
    for step in 1..=steps.max(1) {
        if let Some(top) = best.first() {
            q = format!("{query} {} [step {step}]", top.content.chars().take(80).collect::<String>());
            best = memory.recall_ltm(chat_id, &q, top_k);
        }
    }
    best
}

fn diversify_memories(entries: &mut [MemoryEntry], boost: f32) {
    if entries.len() < 2 {
        return;
    }
    let first_emb = entries[0].embedding_stub.clone();
    for e in entries.iter_mut().skip(1) {
        let sim = cosine(&first_emb, &e.embedding_stub);
        if sim > 0.9 {
            e.importance += boost;
        }
    }
    entries.sort_by(|a, b| {
        b.importance
            .partial_cmp(&a.importance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na > 0.0 && nb > 0.0 {
        dot / (na * nb)
    } else {
        0.0
    }
}

pub fn filter_stm(settings: &AppSettings, entries: Vec<StmEntry>) -> Vec<StmEntry> {
    let mut out = entries;
    let max_msgs = settings.memory.stm_max_messages.max(4) as usize;
    if out.len() > max_msgs {
        let skip = out.len() - max_msgs;
        out = out.into_iter().skip(skip).collect();
    }
    if settings.innovation.chronosync_memory {
        out = chronosync_bucket(out, &settings.innovation.chronosync_granularity);
    }
    if settings.innovation.cognitive_load_balancer {
        out = trim_cognitive_load(out, settings);
    }
    out
}

fn chronosync_bucket(entries: Vec<StmEntry>, granularity: &str) -> Vec<StmEntry> {
    match granularity {
        "session" => entries
            .into_iter()
            .filter(|e| Utc::now().signed_duration_since(e.timestamp).num_hours() < 4)
            .collect(),
        "day" => entries
            .into_iter()
            .filter(|e| Utc::now().signed_duration_since(e.timestamp).num_days() < 1)
            .collect(),
        _ => entries,
    }
}

fn trim_cognitive_load(entries: Vec<StmEntry>, settings: &AppSettings) -> Vec<StmEntry> {
    let max_ctx = settings.inference.context_length.max(2048) as f32;
    let total: u32 = entries.iter().map(|e| e.tokens).sum();
    let load = (total as f32) / max_ctx;
    if load <= settings.innovation.cognitive_load_threshold {
        return entries;
    }
    let keep_ratio = settings.innovation.cognitive_load_threshold / load;
    let keep = ((entries.len() as f32) * keep_ratio).ceil() as usize;
    let skip = entries.len().saturating_sub(keep.max(2));
    entries.into_iter().skip(skip).collect()
}

pub fn neuroplastic_importance(settings: &AppSettings, base: f32, access_count: u32) -> f32 {
    if !settings.innovation.neuroplastic_memory {
        return base;
    }
    let rate = settings.innovation.neuroplastic_adaptation_rate.clamp(0.0, 1.0);
    (base + rate * (access_count as f32 * 0.05)).min(1.0)
}

pub fn maybe_dream_consolidate(memory: &MemoryStore, settings: &AppSettings, chat_id: &str, model_id: &str) {
    if !settings.innovation.dream_consolidation {
        return;
    }
    match settings.innovation.dream_consolidation_schedule.as_str() {
        "idle" | "nightly" => {
            let stm = memory.get_stm(chat_id);
            let threshold = settings.memory.consolidate_threshold.max(4) as usize;
            if stm.len() >= threshold {
                let _ = memory.consolidate_stm_to_ltm(chat_id, model_id);
                audit_log(settings, "memory", &format!("dream consolidation for {chat_id}"));
            }
        }
        _ => {}
    }
}

// ─── Performance + inference tuning ─────────────────────────────────────────

pub struct BackendPreference {
    pub prefer_embedded: bool,
    pub prefer_cli: bool,
}

pub fn runtime_needs_external_cli(settings: &AppSettings) -> bool {
    matches!(
        settings.inference.gguf_runtime.as_str(),
        "llama_cli" | "external_cli"
    ) || (settings.inference.gguf_runtime.as_str() == "synaptic_auto"
        && settings.innovation.synaptic_routing
        && matches!(
            settings.innovation.synaptic_path_priority.as_str(),
            "latency" | "shortest"
        ))
}

pub fn resolve_gguf_runtime_pref(settings: &AppSettings) -> BackendPreference {
    match settings.inference.gguf_runtime.as_str() {
        "llama_cli" | "external_cli" => BackendPreference {
            prefer_embedded: false,
            prefer_cli: true,
        },
        "silenium_core" | "embedded" => BackendPreference {
            prefer_embedded: true,
            prefer_cli: false,
        },
        "synaptic_auto" | "auto" => synaptic_backend_pref(settings),
        _ => BackendPreference {
            prefer_embedded: true,
            prefer_cli: false,
        },
    }
}

fn synaptic_backend_pref(settings: &AppSettings) -> BackendPreference {
    if !settings.innovation.synaptic_routing {
        return BackendPreference {
            prefer_embedded: true,
            prefer_cli: false,
        };
    }
    match settings.innovation.synaptic_path_priority.as_str() {
        "latency" => BackendPreference {
            prefer_embedded: false,
            prefer_cli: true,
        },
        "shortest" => BackendPreference {
            prefer_embedded: false,
            prefer_cli: true,
        },
        "quality" => BackendPreference {
            prefer_embedded: true,
            prefer_cli: false,
        },
        _ => BackendPreference {
            prefer_embedded: true,
            prefer_cli: false,
        },
    }
}

pub fn tune_generate_params(settings: &AppSettings, mut p: GenerateParams) -> GenerateParams {
    let perf = &settings.performance;
    let inv = &settings.innovation;

    if perf.turbo_mode {
        let boost = perf.turbo_ram_boost_percent as u64;
        p.ram_limit_mb = p.ram_limit_mb.saturating_add(p.ram_limit_mb * boost / 100);
        p.threads = p.threads.saturating_add(2).min(64);
    }

    if perf.dynamic_batching {
        let _ = perf.dynamic_batch_max.max(1);
    }

    if perf.tensor_parallel_shards > 1 {
        p.threads = p
            .threads
            .saturating_mul(perf.tensor_parallel_shards.min(8));
    }

    if p.compute_device != "cpu" {
        match perf.mixed_precision.as_str() {
            "int4" | "int8" => p.gpu_layers = p.gpu_layers.saturating_add(4),
            "fp16" | "bf16" => {}
            _ => {}
        }
    }

    if inv.neural_whisper_mode {
        p.max_tokens = p.max_tokens.min(inv.whisper_token_budget.max(16));
    }

    if perf.warmup_tokens > 0 && settings.innovation.predictive_prefetch {
        let _ = perf.warmup_tokens.min(inv.prefetch_horizon_tokens);
    }

    p.kv_offload = perf.kv_cache_offload
        || matches!(perf.kv_offload_device.as_str(), "cpu" | "disk");

    if settings.inference.flash_attention {
        p.kv_offload = true;
    }

    p
}

pub fn effective_max_tokens(settings: &AppSettings, requested: u32) -> u32 {
    let mut max = requested.clamp(32, 2048);
    if settings.innovation.neural_whisper_mode {
        max = max.min(settings.innovation.whisper_token_budget.max(16));
    }
    if settings.performance.latency_target_ms < 150 {
        max = max.min(512);
    }
    max
}

/// Default cap for a single assistant reply in chat (not context window size).
pub fn default_reply_max_tokens() -> u32 {
    512
}

pub fn effective_temperature(settings: &AppSettings, base: f32) -> f32 {
    let mut t = base;
    if settings.innovation.echo_chamber_breaker {
        t += settings.innovation.echo_diversity_boost * 0.15;
    }
    if settings.performance.turbo_mode {
        t = (t * 1.05).min(2.0);
    }
    t.clamp(0.0, 2.0)
}

pub fn cross_modal_user_note(
    settings: &AppSettings,
    attachments_note: Option<String>,
) -> Option<String> {
    if !settings.innovation.cross_modal_fusion {
        return attachments_note;
    }
    let Some(note) = attachments_note else {
        return None;
    };
    let vw = settings.innovation.cross_modal_weight_vision;
    let aw = settings.innovation.cross_modal_weight_audio;
    Some(format!(
        "{note} [cross-modal fusion: vision={vw:.2}, audio={aw:.2}]"
    ))
}

pub fn swarm_agent_count(settings: &AppSettings, default_members: usize) -> usize {
    if settings.innovation.swarm_intelligence {
        settings.innovation.swarm_particle_count.max(2) as usize
    } else {
        default_members
    }
}
