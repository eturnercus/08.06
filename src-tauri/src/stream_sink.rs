use serde::Serialize;
use std::time::Instant;
use tauri::{AppHandle, Emitter};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatStreamPayload {
    pub chat_id: String,
    pub delta: String,
    pub done: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_used: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_recalled: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub injection_applied: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens_limit: Option<u32>,
    #[serde(default)]
    pub cancelled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentStreamPayload {
    pub task_id: String,
    pub agent_id: String,
    pub agent_name: String,
    pub delta: String,
    pub done: bool,
}

pub trait TokenSink {
    fn push(&mut self, delta: &str);
    fn flush(&mut self);
    /// Characters already sent to the UI (streaming).
    fn emitted_chars(&self) -> usize {
        0
    }
    /// If the model produced no stream deltas, push the final text once.
    fn ensure_content(&mut self, text: &str) {
        if self.emitted_chars() == 0 && !text.trim().is_empty() {
            self.push(text);
            self.flush();
        }
    }
}

pub struct StreamSink {
    app: AppHandle,
    chat_id: String,
    buffer_ms: u64,
    pending: String,
    last_flush: Instant,
    total_emitted: usize,
}

impl StreamSink {
    pub fn for_chat(app: AppHandle, chat_id: String, buffer_ms: u64) -> Self {
        Self {
            app,
            chat_id,
            buffer_ms,
            pending: String::new(),
            last_flush: Instant::now(),
            total_emitted: 0,
        }
    }

    pub fn finish(
        &mut self,
        tokens_used: u32,
        prompt_tokens: u32,
        completion_tokens: u32,
        latency_ms: u64,
        model_id: &str,
        memory_recalled: u32,
        injection_applied: bool,
        max_tokens_limit: u32,
    ) {
        TokenSink::flush(self);
        let _ = self.app.emit(
            "chat-stream",
            ChatStreamPayload {
                chat_id: self.chat_id.clone(),
                delta: String::new(),
                done: true,
                tokens_used: Some(tokens_used),
                prompt_tokens: Some(prompt_tokens),
                completion_tokens: Some(completion_tokens),
                latency_ms: Some(latency_ms),
                model_id: Some(model_id.to_string()),
                memory_recalled: Some(memory_recalled),
                injection_applied: Some(injection_applied),
                max_tokens_limit: Some(max_tokens_limit),
                cancelled: false,
                error: None,
            },
        );
    }

    pub fn cancelled(&mut self, latency_ms: u64) {
        TokenSink::flush(self);
        let _ = self.app.emit(
            "chat-stream",
            ChatStreamPayload {
                chat_id: self.chat_id.clone(),
                delta: String::new(),
                done: true,
                tokens_used: None,
                prompt_tokens: None,
                completion_tokens: None,
                latency_ms: Some(latency_ms),
                model_id: None,
                memory_recalled: None,
                injection_applied: None,
                max_tokens_limit: None,
                cancelled: true,
                error: None,
            },
        );
    }

    pub fn error(&mut self, message: String) {
        let _ = self.app.emit(
            "chat-stream",
            ChatStreamPayload {
                chat_id: self.chat_id.clone(),
                delta: String::new(),
                done: true,
                tokens_used: None,
                prompt_tokens: None,
                completion_tokens: None,
                latency_ms: None,
                model_id: None,
                memory_recalled: None,
                injection_applied: None,
                max_tokens_limit: None,
                cancelled: false,
                error: Some(message),
            },
        );
    }

    pub fn emitted_chars(&self) -> usize {
        self.total_emitted
    }
}

impl TokenSink for StreamSink {
    fn emitted_chars(&self) -> usize {
        self.total_emitted
    }

    fn push(&mut self, delta: &str) {
        if delta.is_empty() {
            return;
        }
        self.pending.push_str(delta);
        let elapsed = self.last_flush.elapsed().as_millis() as u64;
        if self.buffer_ms == 0 || elapsed >= self.buffer_ms || delta.contains('\n') {
            TokenSink::flush(self);
        }
    }

    fn flush(&mut self) {
        if self.pending.is_empty() {
            return;
        }
        let delta = std::mem::take(&mut self.pending);
        self.total_emitted += delta.len();
        self.last_flush = Instant::now();
        let _ = self.app.emit(
            "chat-stream",
            ChatStreamPayload {
                chat_id: self.chat_id.clone(),
                delta,
                done: false,
                tokens_used: None,
                prompt_tokens: None,
                completion_tokens: None,
                latency_ms: None,
                model_id: None,
                memory_recalled: None,
                injection_applied: None,
                max_tokens_limit: None,
                cancelled: false,
                error: None,
            },
        );
    }
}

pub struct AgentStreamSink {
    app: AppHandle,
    task_id: String,
    agent_id: String,
    agent_name: String,
    buffer_ms: u64,
    pending: String,
    last_flush: Instant,
}

impl AgentStreamSink {
    pub fn new(
        app: AppHandle,
        task_id: String,
        agent_id: String,
        agent_name: String,
        buffer_ms: u64,
    ) -> Self {
        Self {
            app,
            task_id,
            agent_id,
            agent_name,
            buffer_ms,
            pending: String::new(),
            last_flush: Instant::now(),
        }
    }

    pub fn finish(&mut self) {
        TokenSink::flush(self);
        let _ = self.app.emit(
            "agent-stream",
            AgentStreamPayload {
                task_id: self.task_id.clone(),
                agent_id: self.agent_id.clone(),
                agent_name: self.agent_name.clone(),
                delta: String::new(),
                done: true,
            },
        );
    }
}

impl TokenSink for AgentStreamSink {
    fn push(&mut self, delta: &str) {
        if delta.is_empty() {
            return;
        }
        self.pending.push_str(delta);
        let elapsed = self.last_flush.elapsed().as_millis() as u64;
        if self.buffer_ms == 0 || elapsed >= self.buffer_ms {
            TokenSink::flush(self);
        }
    }

    fn flush(&mut self) {
        if self.pending.is_empty() {
            return;
        }
        let delta = std::mem::take(&mut self.pending);
        self.last_flush = Instant::now();
        let _ = self.app.emit(
            "agent-stream",
            AgentStreamPayload {
                task_id: self.task_id.clone(),
                agent_id: self.agent_id.clone(),
                agent_name: self.agent_name.clone(),
                delta,
                done: false,
            },
        );
    }
}

pub fn should_stream_chat(settings: &crate::settings::AppSettings) -> bool {
    settings.inference.streaming || settings.innovation.thought_streaming
}

pub const AGENT_ORCH_EVENT: &str = "agent-orchestration";

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentOrchestrationPayload {
    pub task_id: String,
    pub group_id: String,
    pub group_name: String,
    pub orchestration_mode: String,
    pub round: u32,
    pub phase: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_id: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

pub fn emit_orchestration(app: &AppHandle, payload: AgentOrchestrationPayload) {
    let _ = app.emit(AGENT_ORCH_EVENT, payload);
}

pub fn stream_buffer_ms(settings: &crate::settings::AppSettings) -> u64 {
    if settings.innovation.thought_streaming {
        settings.innovation.thought_stream_buffer_ms.max(0) as u64
    } else {
        0
    }
}
