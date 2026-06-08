mod agents;
mod devices;
mod gguf_runner;
mod inference;
mod llama_cli;
mod memory;
mod network;
mod settings;

use agents::AgentOrchestrator;
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
}

#[tauri::command]
fn get_settings(state: State<'_, AppState>) -> AppSettings {
    state.settings.lock().clone()
}

#[tauri::command]
fn update_settings(state: State<'_, AppState>, settings: AppSettings) -> Result<(), String> {
    save_settings(&settings)?;
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
    state: State<'_, AppState>,
    request: ChatRequest,
) -> Result<inference::ChatResponse, String> {
    let settings = state.settings.lock().clone();
    let inference = Arc::clone(&state.inference);
    let memory = Arc::clone(&state.memory);
    tauri::async_runtime::spawn_blocking(move || {
        inference.chat(&settings, &memory, &request)
    })
    .await
    .map_err(|e| format!("inference task: {e}"))
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
) -> Result<network::NetworkRequestLog, String> {
    state.network.web_search(&query, agent_id).await
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
    state: State<'_, AppState>,
    group_id: String,
    prompt: String,
) -> Result<agents::AgentTask, String> {
    let settings = state.settings.lock().clone();
    state
        .agents
        .run_team_task(&settings, &group_id, &prompt, &state.memory, &state.network)
        .await
}

#[tauri::command]
fn list_agent_tasks(state: State<'_, AppState>) -> Vec<agents::AgentTask> {
    state.agents.list_tasks()
}

#[tauri::command]
fn load_model(state: State<'_, AppState>, path: String, name: String) -> Result<inference::ModelInfo, String> {
    state.inference.load_model(&path, &name)
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
    state.inference.verify_model(&model_id)
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
    let app_state = AppState {
        settings: Mutex::new(settings),
        memory: Arc::new(MemoryStore::new()),
        network: Arc::new(NetworkManager::new()),
        inference: Arc::new(InferenceEngine::new()),
        agents: Arc::new(AgentOrchestrator::new()),
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
