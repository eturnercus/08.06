use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub language: String,
    pub first_run_completed: bool,
    pub onboarding_step: u32,
    pub system: SystemSettings,
    pub network: NetworkSettings,
    pub memory: MemorySettings,
    pub inference: InferenceSettings,
    pub devices: DeviceSettings,
    pub ui: UiSettings,
    pub advanced: AdvancedSettings,
    pub global_message_injection: GlobalMessageInjection,
    pub per_chat_overrides: HashMap<String, ChatOverride>,
    pub agent_groups: Vec<AgentGroupConfig>,
    pub custom_models: Vec<CustomModelEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemSettings {
    pub ram_limit_mb: u64,
    pub ram_soft_limit_percent: u8,
    pub cpu_cores: Vec<u32>,
    pub cpu_affinity_mode: String,
    pub gpu_layers: u32,
    pub gpu_memory_mb: u64,
    pub thread_count: u32,
    pub batch_size: u32,
    pub mmap_enabled: bool,
    pub mlock_enabled: bool,
    pub numa_node: i32,
    pub process_priority: String,
    pub swap_usage: String,
    pub disk_cache_mb: u64,
    pub temp_dir: String,
    pub auto_gc_interval_sec: u32,
    pub oom_policy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkSettings {
    pub isolation_mode: String,
    pub allow_internet: bool,
    pub api_only_endpoints: Vec<String>,
    pub proxy_url: String,
    pub dns_over_https: bool,
    pub request_timeout_sec: u32,
    pub max_concurrent_requests: u32,
    pub log_all_requests: bool,
    pub block_private_ips: bool,
    pub user_agent: String,
    pub tls_verify: bool,
    pub websocket_enabled: bool,
    pub huggingface_mirror: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemorySettings {
    pub stm_enabled: bool,
    pub stm_max_tokens: u32,
    pub stm_ttl_minutes: u32,
    pub ltm_enabled: bool,
    pub ltm_max_entries: u32,
    pub ltm_persistence: String,
    pub ltm_embedding_dims: u32,
    pub transfer_policy: String,
    pub auto_consolidate: bool,
    pub consolidate_threshold: u32,
    pub memory_compression: bool,
    pub cross_chat_transfer: bool,
    pub cross_model_transfer: bool,
    pub memory_encryption: bool,
    pub recall_top_k: u32,
    pub semantic_search: bool,
    pub decay_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InferenceSettings {
    pub default_backend: String,
    pub model_path: String,
    pub huggingface_repo: String,
    pub context_length: u32,
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: u32,
    pub repeat_penalty: f32,
    pub seed: i64,
    pub streaming: bool,
    pub flash_attention: bool,
    pub kv_cache_quant: String,
    pub rope_scaling: String,
    pub supported_formats: Vec<String>,
    pub auto_unload_idle_min: u32,
    pub speculative_decoding: bool,
    pub draft_model_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSettings {
    pub camera_enabled: bool,
    pub microphone_enabled: bool,
    pub screen_capture_enabled: bool,
    pub virtual_display_extend: bool,
    pub virtual_display_resolution: String,
    pub audio_input_device: String,
    pub audio_output_device: String,
    pub video_input_device: String,
    pub max_attachment_mb: u64,
    pub allowed_attachment_types: Vec<String>,
    pub ocr_on_images: bool,
    pub transcribe_audio: bool,
    pub frame_rate: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiSettings {
    pub theme: String,
    pub font_size: u32,
    pub compact_mode: bool,
    pub show_token_counter: bool,
    pub show_latency: bool,
    pub sidebar_width: u32,
    pub animations_enabled: bool,
    pub high_contrast: bool,
    pub reduce_motion: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvancedSettings {
    pub debug_mode: bool,
    pub telemetry: bool,
    pub crash_reports: bool,
    pub experimental_features: bool,
    pub custom_cuda_flags: String,
    pub log_level: String,
    pub max_log_files: u32,
    pub plugin_directory: String,
    pub sandbox_level: String,
    pub watchdog_enabled: bool,
    pub auto_restart_on_crash: bool,
    pub health_check_interval_sec: u32,
}

/// Инновационная настройка: вводные, подмешиваемые в КАЖДОЕ сообщение
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalMessageInjection {
    pub enabled: bool,
    pub system_prefix: String,
    pub user_suffix: String,
    pub hidden_context: String,
    pub inject_memory_summary: bool,
    pub inject_device_state: bool,
    pub inject_timestamp: bool,
    pub inject_locale: bool,
    pub per_agent_overrides: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatOverride {
    pub allow_internet: Option<bool>,
    pub stm_enabled: Option<bool>,
    pub ltm_enabled: Option<bool>,
    pub camera_enabled: Option<bool>,
    pub microphone_enabled: Option<bool>,
    pub screen_capture_enabled: Option<bool>,
    pub ram_limit_mb: Option<u64>,
    pub agent_group_id: Option<String>,
    pub custom_injection: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentGroupConfig {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub orchestration_mode: String,
    pub members: Vec<AgentMember>,
    pub shared_memory: bool,
    pub shared_ltm_pool_id: Option<String>,
    pub max_rounds: u32,
    pub consensus_threshold: f32,
    pub parallel_execution: bool,
    pub supervisor_agent_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentMember {
    pub id: String,
    pub name: String,
    pub role: String,
    pub model_id: String,
    pub permissions: AgentPermissions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentPermissions {
    pub internet: bool,
    pub camera: bool,
    pub microphone: bool,
    pub screen: bool,
    pub stm: bool,
    pub ltm: bool,
    pub can_delegate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomModelEntry {
    pub id: String,
    pub name: String,
    pub path: String,
    pub format: String,
    pub backend: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "ru".into(),
            first_run_completed: false,
            onboarding_step: 0,
            system: SystemSettings {
                ram_limit_mb: 8192,
                ram_soft_limit_percent: 85,
                cpu_cores: vec![0, 1, 2, 3],
                cpu_affinity_mode: "auto".into(),
                gpu_layers: 0,
                gpu_memory_mb: 4096,
                thread_count: 4,
                batch_size: 512,
                mmap_enabled: true,
                mlock_enabled: false,
                numa_node: -1,
                process_priority: "normal".into(),
                swap_usage: "minimal".into(),
                disk_cache_mb: 2048,
                temp_dir: String::new(),
                auto_gc_interval_sec: 300,
                oom_policy: "graceful_degrade".into(),
            },
            network: NetworkSettings {
                isolation_mode: "api_only".into(),
                allow_internet: false,
                api_only_endpoints: vec![
                    "https://huggingface.co".into(),
                    "https://*.huggingface.co".into(),
                ],
                proxy_url: String::new(),
                dns_over_https: true,
                request_timeout_sec: 30,
                max_concurrent_requests: 4,
                log_all_requests: true,
                block_private_ips: true,
                user_agent: "NeuroForge/1.0".into(),
                tls_verify: true,
                websocket_enabled: false,
                huggingface_mirror: String::new(),
            },
            memory: MemorySettings {
                stm_enabled: true,
                stm_max_tokens: 4096,
                stm_ttl_minutes: 120,
                ltm_enabled: true,
                ltm_max_entries: 10000,
                ltm_persistence: "sqlite".into(),
                ltm_embedding_dims: 384,
                transfer_policy: "explicit_approval".into(),
                auto_consolidate: true,
                consolidate_threshold: 50,
                memory_compression: true,
                cross_chat_transfer: true,
                cross_model_transfer: false,
                memory_encryption: false,
                recall_top_k: 8,
                semantic_search: true,
                decay_rate: 0.01,
            },
            inference: InferenceSettings {
                default_backend: "gguf".into(),
                model_path: String::new(),
                huggingface_repo: String::new(),
                context_length: 8192,
                temperature: 0.7,
                top_p: 0.9,
                top_k: 40,
                repeat_penalty: 1.1,
                seed: -1,
                streaming: true,
                flash_attention: true,
                kv_cache_quant: "q8_0".into(),
                rope_scaling: "linear".into(),
                supported_formats: vec![
                    "gguf".into(),
                    "onnx".into(),
                    "safetensors".into(),
                    "pt".into(),
                    "bin".into(),
                    "ggml".into(),
                ],
                auto_unload_idle_min: 30,
                speculative_decoding: false,
                draft_model_path: String::new(),
            },
            devices: DeviceSettings {
                camera_enabled: false,
                microphone_enabled: false,
                screen_capture_enabled: false,
                virtual_display_extend: false,
                virtual_display_resolution: "1920x1080".into(),
                audio_input_device: "default".into(),
                audio_output_device: "default".into(),
                video_input_device: "default".into(),
                max_attachment_mb: 100,
                allowed_attachment_types: vec![
                    "image/*".into(),
                    "audio/*".into(),
                    "video/*".into(),
                    "application/pdf".into(),
                ],
                ocr_on_images: true,
                transcribe_audio: true,
                frame_rate: 30,
            },
            ui: UiSettings {
                theme: "dark".into(),
                font_size: 14,
                compact_mode: false,
                show_token_counter: true,
                show_latency: true,
                sidebar_width: 280,
                animations_enabled: true,
                high_contrast: false,
                reduce_motion: false,
            },
            advanced: AdvancedSettings {
                debug_mode: false,
                telemetry: false,
                crash_reports: false,
                experimental_features: true,
                custom_cuda_flags: String::new(),
                log_level: "info".into(),
                max_log_files: 10,
                plugin_directory: String::new(),
                sandbox_level: "standard".into(),
                watchdog_enabled: true,
                auto_restart_on_crash: true,
                health_check_interval_sec: 60,
            },
            global_message_injection: GlobalMessageInjection {
                enabled: false,
                system_prefix: String::new(),
                user_suffix: String::new(),
                hidden_context: String::new(),
                inject_memory_summary: true,
                inject_device_state: false,
                inject_timestamp: true,
                inject_locale: true,
                per_agent_overrides: HashMap::new(),
            },
            per_chat_overrides: HashMap::new(),
            agent_groups: vec![],
            custom_models: vec![],
        }
    }
}

pub fn settings_path() -> PathBuf {
    let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("neuroforge");
    fs::create_dir_all(&path).ok();
    path.push("settings.json");
    path
}

pub fn load_settings() -> AppSettings {
    let path = settings_path();
    if path.exists() {
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(settings) = serde_json::from_str::<AppSettings>(&data) {
                return settings;
            }
        }
    }
    AppSettings::default()
}

pub fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let path = settings_path();
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

pub fn reset_settings() -> AppSettings {
    let defaults = AppSettings::default();
    let _ = save_settings(&defaults);
    defaults
}
