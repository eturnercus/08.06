mod agents;
mod devices;
mod gguf_runner;
mod inference;
mod llama_cli;
mod memory;
mod network;
mod settings;
mod settings_engine;
mod storage_crypto;
mod inference_cancel;
mod stream_sink;

use agents::AgentOrchestrator;
use inference_cancel::CancelRegistry;
use inference::{ChatRequest, InferenceEngine};
use memory::MemoryStore;
use network::{FetchParams, NetworkManager};
use parking_lot::Mutex;
use settings::{load_settings, reset_settings, save_settings, AppSettings};
use std::sync::Arc;
use tauri::State;

pub struct AppState {
    pub settings: Mutex<AppSettings>,
    pub memory: Arc<MemoryStore>,
    pub network: Arc<NetworkManager>,
    pub inference: Arc<InferenceEngine>,
    pub agents: Arc<AgentOrchestrator>,
    pub cancel: Arc<CancelRegistry>,
}

#[tauri::command]
fn get_settings(state: State<'_, AppState>) -> AppSettings {
    state.settings.lock().clone()
}

#[tauri::command]
fn update_settings(state: State<'_, AppState>, settings: AppSettings) -> Result<(), String> {
    save_settings(&settings)?;
    state
        .memory
        .set_encrypt_at_rest(settings.security.encrypt_memory_at_rest);
    settings_engine::audit_log(&settings, "settings", "settings updated");
    *state.settings.lock() = settings;
    Ok(())
}

#[tauri::command]
fn reset_settings_cmd(state: State<'_, AppState>) -> AppSettings {
    let defaults = reset_settings();
    *state.settings.lock() = defaults.clone();
    defaults
}

#[tauri::command]
async fn send_chat(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    request: ChatRequest,
) -> Result<inference::ChatResponse, String> {
    let settings = state.settings.lock().clone();
    let inference = Arc::clone(&state.inference);
    let memory = Arc::clone(&state.memory);
    let cancel_reg = Arc::clone(&state.cancel);
    let stream_enabled = stream_sink::should_stream_chat(&settings);
    let buffer_ms = stream_sink::stream_buffer_ms(&settings);
    let chat_id = request.chat_id.clone();
    let cancel_flag = cancel_reg.begin(&chat_id);
    tauri::async_runtime::spawn_blocking(move || {
        let mut stream_sink = if stream_enabled {
            Some(stream_sink::StreamSink::for_chat(app, chat_id.clone(), buffer_ms))
        } else {
            None
        };
        let resp = inference.chat(
            &settings,
            &memory,
            &request,
            &mut stream_sink,
            Some(&cancel_flag),
        );
        cancel_reg.finish(&chat_id);
        resp
    })
    .await
    .map_err(|e| format!("inference task: {e}"))
}

#[tauri::command]
fn stop_chat(state: State<'_, AppState>, chat_id: String) {
    state.cancel.cancel(&chat_id);
}

#[tauri::command]
fn sync_chat_overrides(
    state: State<'_, AppState>,
    chat_id: String,
    allow_internet: bool,
    stm_enabled: bool,
    ltm_enabled: bool,
    agent_group_id: Option<String>,
) -> Result<(), String> {
    let mut settings = state.settings.lock();
    settings
        .per_chat_overrides
        .insert(chat_id, settings::ChatOverride {
            allow_internet: Some(allow_internet),
            stm_enabled: Some(stm_enabled),
            ltm_enabled: Some(ltm_enabled),
            agent_group_id,
            ..Default::default()
        });
    save_settings(&settings)?;
    Ok(())
}

#[tauri::command]
fn get_audit_logs(max_lines: Option<usize>) -> Vec<String> {
    settings_engine::read_audit_log_tail(max_lines.unwrap_or(200))
}

#[tauri::command]
async fn open_browser_url(
    state: State<'_, AppState>,
    url: String,
    chat_id: Option<String>,
    agent_id: Option<String>,
) -> Result<network::NetworkRequestLog, String> {
    let settings = state.settings.lock().clone();
    if !settings.devices.browser_automation_enabled {
        return Err("Управление браузером отключено в Настройки → Устройства".into());
    }
    let allow = chat_id
        .as_ref()
        .and_then(|id| settings.per_chat_overrides.get(id))
        .and_then(|o| o.allow_internet)
        .unwrap_or(settings.network.allow_internet);
    if !allow {
        return Err("Интернет отключён для этого чата".into());
    }
    open_system_url(&url)?;
    settings_engine::audit_log(
        &settings,
        "browser",
        &format!("open url={url} chat={chat_id:?} agent={agent_id:?}"),
    );
    let log = network::NetworkRequestLog {
        id: uuid::Uuid::new_v4().to_string(),
        agent_id,
        chat_id,
        method: "OPEN".into(),
        url: url.clone(),
        status: Some(200),
        request_headers: std::collections::HashMap::new(),
        response_preview: "Открыто во внешнем браузере".into(),
        duration_ms: 0,
        blocked: false,
        block_reason: None,
        timestamp: chrono::Utc::now(),
    };
    state.network.record_log(log.clone());
    Ok(log)
}

#[tauri::command]
async fn agent_fetch(
    state: State<'_, AppState>,
    url: String,
    chat_id: Option<String>,
    agent_id: Option<String>,
) -> Result<network::NetworkRequestLog, String> {
    let settings = state.settings.lock().clone();
    let allow = chat_id
        .as_ref()
        .and_then(|id| settings.per_chat_overrides.get(id))
        .and_then(|o| o.allow_internet)
        .unwrap_or(settings.network.allow_internet);
    state
        .network
        .fetch(FetchParams {
            url,
            method: "GET".into(),
            body: None,
            agent_id,
            chat_id,
            allow_internet: allow,
            isolation_mode: settings.network.isolation_mode.clone(),
            api_endpoints: settings.network.api_only_endpoints.clone(),
            data_exfiltration_guard: settings.security.data_exfiltration_guard,
            audit_enabled: settings.security.audit_log_enabled,
            block_private_ips: settings.network.block_private_ips,
            network_fingerprint_check: settings.security.network_fingerprint_check,
        })
        .await
}

#[tauri::command]
fn get_network_logs(state: State<'_, AppState>) -> Vec<network::NetworkRequestLog> {
    state.network.get_logs()
}

#[tauri::command]
async fn web_search(
    state: State<'_, AppState>,
    query: String,
    agent_id: Option<String>,
    chat_id: Option<String>,
) -> Result<network::NetworkRequestLog, String> {
    let settings = state.settings.lock().clone();
    let allow = chat_id
        .as_ref()
        .and_then(|id| settings.per_chat_overrides.get(id))
        .and_then(|o| o.allow_internet)
        .unwrap_or(settings.network.allow_internet);
    state
        .network
        .web_search(&query, agent_id, chat_id, allow, &settings)
        .await
}

fn open_system_url(url: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()
            .map_err(|e| format!("Не удалось открыть браузер: {e}"))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("Не удалось открыть браузер: {e}"))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("Не удалось открыть браузер: {e}"))?;
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        let _ = url;
        return Err("Платформа не поддерживает открытие браузера".into());
    }
    Ok(())
}

#[tauri::command]
fn get_memory_stm(state: State<'_, AppState>, chat_id: String) -> Vec<memory::StmEntry> {
    state.memory.get_stm(&chat_id)
}

#[tauri::command]
fn get_memory_ltm(
    state: State<'_, AppState>,
    chat_id: Option<String>,
) -> Vec<memory::MemoryEntry> {
    state.memory.get_all_ltm(chat_id.as_deref())
}

#[tauri::command]
fn transfer_memory(
    state: State<'_, AppState>,
    entry_ids: Vec<String>,
    from_chat: String,
    to_chat: String,
    from_model: String,
    to_model: String,
    memory_type: String,
) -> memory::MemoryTransferRequest {
    state.memory.transfer_memory(
        entry_ids,
        &from_chat,
        &to_chat,
        &from_model,
        &to_model,
        &memory_type,
    )
}

#[tauri::command]
fn consolidate_memory(
    state: State<'_, AppState>,
    chat_id: String,
    model_id: String,
) -> Option<memory::MemoryEntry> {
    state.memory.consolidate_stm_to_ltm(&chat_id, &model_id)
}

#[tauri::command]
async fn run_agent_team(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    group_id: String,
    prompt: String,
    chat_id: Option<String>,
) -> Result<agents::AgentTask, String> {
    let settings = state.settings.lock().clone();
    let inference = Arc::clone(&state.inference);
    state
        .agents
        .run_team_task(
            &settings,
            &group_id,
            &prompt,
            &state.memory,
            &state.network,
            &inference,
            app,
            chat_id,
        )
        .await
}

#[tauri::command]
fn list_agent_tasks(state: State<'_, AppState>) -> Vec<agents::AgentTask> {
    state.agents.list_tasks()
}

#[tauri::command]
fn load_model(state: State<'_, AppState>, path: String, name: String) -> Result<inference::ModelInfo, String> {
    let settings = state.settings.lock();
    state
        .inference
        .load_model(&path, &name, settings.security.model_integrity_verify)
}

#[tauri::command]
async fn download_huggingface_model(
    state: State<'_, AppState>,
    repo: String,
) -> Result<inference::DownloadResult, String> {
    state.inference.download_huggingface(&repo).await
}

#[tauri::command]
fn get_models_directory() -> String {
    inference::models_directory().to_string_lossy().to_string()
}

#[tauri::command]
fn scan_local_models(state: State<'_, AppState>) -> Vec<inference::ModelInfo> {
    state.inference.scan_directory()
}

#[tauri::command]
fn verify_model(state: State<'_, AppState>, model_id: String) -> Result<bool, String> {
    let settings = state.settings.lock();
    state
        .inference
        .verify_model(&model_id, settings.security.model_integrity_verify)
}

#[tauri::command]
fn list_models(state: State<'_, AppState>) -> Vec<inference::ModelInfo> {
    state.inference.list_models()
}

#[tauri::command]
fn get_device_status(state: State<'_, AppState>) -> devices::DeviceStatus {
    let settings = state.settings.lock().clone();
    devices::DeviceManager::get_status(&settings.devices)
}

#[tauri::command]
fn capture_screen(state: State<'_, AppState>) -> devices::CaptureResult {
    let settings = state.settings.lock().clone();
    devices::DeviceManager::capture_screen(&settings.devices)
}

#[tauri::command]
fn capture_audio(state: State<'_, AppState>) -> devices::CaptureResult {
    let settings = state.settings.lock().clone();
    devices::DeviceManager::capture_audio(&settings.devices)
}

#[tauri::command]
fn capture_camera(state: State<'_, AppState>) -> devices::CaptureResult {
    let settings = state.settings.lock().clone();
    devices::DeviceManager::capture_camera(&settings.devices)
}

#[tauri::command]
fn get_system_info(state: State<'_, AppState>) -> serde_json::Value {
    let settings = state.settings.lock().clone();
    serde_json::json!({
        "ramLimitMb": settings.system.ram_limit_mb,
        "cpuCores": settings.system.cpu_cores,
        "threadCount": settings.system.thread_count,
        "gpuLayers": settings.system.gpu_layers,
        "gpuMemoryMb": settings.system.gpu_memory_mb,
        "platform": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();
    let settings = load_settings();
    let memory = Arc::new(MemoryStore::new());
    memory.set_encrypt_at_rest(settings.security.encrypt_memory_at_rest);
    let app_state = AppState {
        settings: Mutex::new(settings),
        memory,
        network: Arc::new(NetworkManager::new()),
        inference: Arc::new(InferenceEngine::new()),
        agents: Arc::new(AgentOrchestrator::new()),
        cancel: Arc::new(CancelRegistry::new()),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_settings,
            update_settings,
            reset_settings_cmd,
            send_chat,
            stop_chat,
            sync_chat_overrides,
            get_audit_logs,
            open_browser_url,
            agent_fetch,
            get_network_logs,
            web_search,
            get_memory_stm,
            get_memory_ltm,
            transfer_memory,
            consolidate_memory,
            run_agent_team,
            list_agent_tasks,
            load_model,
            download_huggingface_model,
            get_models_directory,
            scan_local_models,
            verify_model,
            list_models,
            get_device_status,
            capture_screen,
            capture_audio,
            capture_camera,
            get_system_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running NeuroForge");
}
