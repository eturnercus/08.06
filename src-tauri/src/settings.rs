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
    #[serde(default)]
    pub innovation: InnovationSettings,
    #[serde(default)]
    pub security: SecuritySettings,
    #[serde(default)]
    pub performance: PerformanceSettings,
    pub per_chat_overrides: HashMap<String, ChatOverride>,
    pub agent_groups: Vec<AgentGroupConfig>,
    pub custom_models: Vec<CustomModelEntry>,
}

/// Передовые инновационные настройки Silenium
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InnovationSettings {
    pub cognitive_load_balancer: bool,
    pub cognitive_load_threshold: f32,
    pub neuroplastic_memory: bool,
    pub neuroplastic_adaptation_rate: f32,
    pub synaptic_routing: bool,
    pub synaptic_path_priority: String,
    pub context_dna: bool,
    pub context_dna_mutation_rate: f32,
    pub thought_streaming: bool,
    pub thought_stream_buffer_ms: u32,
    #[serde(default = "default_thought_max_tokens")]
    pub thought_max_tokens: u32,
    pub emotion_mirror: bool,
    pub emotion_mirror_intensity: f32,
    pub neural_mesh_sync: bool,
    pub neural_mesh_peers: Vec<String>,
    pub quantum_context_layers: u32,
    pub quantum_entanglement_strength: f32,
    pub attention_cascade: bool,
    pub attention_cascade_depth: u32,
    pub dream_consolidation: bool,
    pub dream_consolidation_schedule: String,
    pub persona_fluidity: bool,
    pub persona_blend_ratio: f32,
    pub cross_modal_fusion: bool,
    pub cross_modal_weight_vision: f32,
    pub cross_modal_weight_audio: f32,
    pub predictive_prefetch: bool,
    pub prefetch_horizon_tokens: u32,
    pub neural_firewall: bool,
    pub firewall_sensitivity: f32,
    pub ambient_context_harvest: bool,
    pub ambient_harvest_interval_sec: u32,
    pub temporal_anchoring: bool,
    pub temporal_anchor_window_min: u32,
    pub holographic_context: bool,
    pub holographic_projection_dims: u32,
    pub swarm_intelligence: bool,
    pub swarm_particle_count: u32,
    pub meta_cognition_loop: bool,
    pub meta_cognition_interval: u32,
    pub resonance_tuning: bool,
    pub resonance_frequency_hz: f32,
    pub latent_space_navigation: bool,
    pub latent_navigation_steps: u32,
    pub echo_chamber_breaker: bool,
    pub echo_diversity_boost: f32,
    pub neural_whisper_mode: bool,
    pub whisper_token_budget: u32,
    pub chronosync_memory: bool,
    pub chronosync_granularity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecuritySettings {
    pub encrypt_settings: bool,
    pub encrypt_memory_at_rest: bool,
    pub require_confirmation_internet: bool,
    pub require_confirmation_device: bool,
    pub audit_log_enabled: bool,
    pub audit_log_retention_days: u32,
    pub sandbox_process_isolation: bool,
    pub api_key_vault: bool,
    pub auto_lock_minutes: u32,
    pub biometric_unlock: bool,
    pub network_fingerprint_check: bool,
    pub model_integrity_verify: bool,
    pub prompt_injection_shield: bool,
    pub shield_aggressiveness: f32,
    pub data_exfiltration_guard: bool,
    pub clipboard_sanitization: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceSettings {
    pub turbo_mode: bool,
    pub turbo_ram_boost_percent: u8,
    pub dynamic_batching: bool,
    pub dynamic_batch_max: u32,
    pub pipeline_parallelism: bool,
    pub pipeline_stages: u32,
    pub kv_cache_offload: bool,
    pub kv_offload_device: String,
    pub continuous_batching: bool,
    pub prefix_caching: bool,
    pub prefix_cache_ttl_min: u32,
    pub tensor_parallel_shards: u32,
    pub mixed_precision: String,
    pub compile_graph: bool,
    pub warmup_tokens: u32,
    pub idle_power_save: bool,
    pub idle_power_threshold_min: u32,
    pub priority_queue_inference: bool,
    pub max_queue_depth: u32,
    pub latency_target_ms: u32,
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
    #[serde(default)]
    pub vram_reserve_mb: u64,
    #[serde(default)]
    pub cpu_boost_cores: Vec<u32>,
    #[serde(default)]
    pub thermal_throttle_c: u32,
    #[serde(default)]
    pub io_priority: String,
    #[serde(default)]
    pub huge_pages: bool,
    #[serde(default)]
    pub prefetch_models: bool,
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
    #[serde(default)]
    pub tor_enabled: bool,
    #[serde(default)]
    pub egress_filter_mode: String,
    #[serde(default)]
    pub rate_limit_rpm: u32,
    #[serde(default)]
    pub circuit_breaker_threshold: u32,
    #[serde(default)]
    pub offline_fallback: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemorySettings {
    pub stm_enabled: bool,
    pub stm_max_tokens: u32,
    #[serde(default = "default_stm_max_messages")]
    pub stm_max_messages: u32,
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
    #[serde(default)]
    pub episodic_memory: bool,
    #[serde(default)]
    pub procedural_memory: bool,
    #[serde(default)]
    pub working_memory_slots: u32,
    #[serde(default)]
    pub memory_graph_enabled: bool,
    #[serde(default)]
    pub graph_max_edges: u32,
    #[serde(default)]
    pub forgetting_curve: String,
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
    #[serde(default)]
    pub browser_automation_enabled: bool,
    #[serde(default)]
    pub desktop_control_enabled: bool,
}

fn default_thought_max_tokens() -> u32 {
    1024
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
#[derive(Default)]
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
    #[serde(default)]
    pub workspace_path: Option<String>,
    #[serde(default)]
    pub memory_access: Option<String>,
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
    #[serde(default)]
    pub conflict_mode: String,
    #[serde(default = "default_timeout")]
    pub timeout_sec: u32,
    #[serde(default)]
    pub feedback_loops: bool,
    #[serde(default)]
    pub task_decomposition: bool,
}

fn default_timeout() -> u32 {
    120
}

fn default_stm_max_messages() -> u32 {
    50
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentMember {
    pub id: String,
    pub name: String,
    pub role: String,
    pub model_id: String,
    pub permissions: AgentPermissions,
    #[serde(default)]
    pub resources: AgentResources,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default = "default_trigger")]
    pub trigger: String,
    #[serde(default)]
    pub trigger_keyword: String,
    #[serde(default)]
    pub system_prompt: String,
    #[serde(default)]
    pub system_prompt_customized: bool,
}

fn default_trigger() -> String {
    "always".into()
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
    #[serde(default)]
    pub files: bool,
    #[serde(default)]
    pub tools: bool,
    #[serde(default)]
    pub veto: bool,
    #[serde(default)]
    pub shared_memory: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentResources {
    pub ram_limit_mb: u64,
    pub cpu_cores: Vec<u32>,
    pub max_tokens: u32,
    pub temperature: f32,
    pub execution_order: u32,
}

impl Default for AgentResources {
    fn default() -> Self {
        Self {
            ram_limit_mb: 2048,
            cpu_cores: vec![0, 1],
            max_tokens: 2048,
            temperature: 0.7,
            execution_order: 0,
        }
    }
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

impl Default for InnovationSettings {
    fn default() -> Self {
        Self {
            cognitive_load_balancer: true,
            cognitive_load_threshold: 0.75,
            neuroplastic_memory: true,
            neuroplastic_adaptation_rate: 0.05,
            synaptic_routing: true,
            synaptic_path_priority: "adaptive".into(),
            context_dna: true,
            context_dna_mutation_rate: 0.02,
            thought_streaming: true,
            thought_stream_buffer_ms: 120,
            thought_max_tokens: 1024,
            emotion_mirror: false,
            emotion_mirror_intensity: 0.5,
            neural_mesh_sync: false,
            neural_mesh_peers: vec![],
            quantum_context_layers: 4,
            quantum_entanglement_strength: 0.3,
            attention_cascade: true,
            attention_cascade_depth: 6,
            dream_consolidation: true,
            dream_consolidation_schedule: "idle".into(),
            persona_fluidity: false,
            persona_blend_ratio: 0.3,
            cross_modal_fusion: true,
            cross_modal_weight_vision: 0.4,
            cross_modal_weight_audio: 0.35,
            predictive_prefetch: true,
            prefetch_horizon_tokens: 256,
            neural_firewall: true,
            firewall_sensitivity: 0.7,
            ambient_context_harvest: false,
            ambient_harvest_interval_sec: 300,
            temporal_anchoring: true,
            temporal_anchor_window_min: 60,
            holographic_context: true,
            holographic_projection_dims: 512,
            swarm_intelligence: false,
            swarm_particle_count: 8,
            meta_cognition_loop: true,
            meta_cognition_interval: 10,
            resonance_tuning: false,
            resonance_frequency_hz: 7.83,
            latent_space_navigation: true,
            latent_navigation_steps: 3,
            echo_chamber_breaker: true,
            echo_diversity_boost: 0.25,
            neural_whisper_mode: false,
            whisper_token_budget: 64,
            chronosync_memory: true,
            chronosync_granularity: "message".into(),
        }
    }
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            encrypt_settings: false,
            encrypt_memory_at_rest: false,
            require_confirmation_internet: true,
            require_confirmation_device: true,
            audit_log_enabled: true,
            audit_log_retention_days: 30,
            sandbox_process_isolation: true,
            api_key_vault: true,
            auto_lock_minutes: 0,
            biometric_unlock: false,
            network_fingerprint_check: true,
            model_integrity_verify: true,
            prompt_injection_shield: true,
            shield_aggressiveness: 0.6,
            data_exfiltration_guard: true,
            clipboard_sanitization: true,
        }
    }
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            turbo_mode: false,
            turbo_ram_boost_percent: 15,
            dynamic_batching: true,
            dynamic_batch_max: 32,
            pipeline_parallelism: true,
            pipeline_stages: 4,
            kv_cache_offload: false,
            kv_offload_device: "cpu".into(),
            continuous_batching: true,
            prefix_caching: true,
            prefix_cache_ttl_min: 60,
            tensor_parallel_shards: 1,
            mixed_precision: "bf16".into(),
            compile_graph: true,
            warmup_tokens: 128,
            idle_power_save: true,
            idle_power_threshold_min: 5,
            priority_queue_inference: true,
            max_queue_depth: 16,
            latency_target_ms: 200,
        }
    }
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
                vram_reserve_mb: 512,
                cpu_boost_cores: vec![],
                thermal_throttle_c: 85,
                io_priority: "normal".into(),
                huge_pages: false,
                prefetch_models: true,
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
                user_agent: "Silenium/1.0".into(),
                tls_verify: true,
                websocket_enabled: false,
                huggingface_mirror: String::new(),
                tor_enabled: false,
                egress_filter_mode: "strict".into(),
                rate_limit_rpm: 60,
                circuit_breaker_threshold: 10,
                offline_fallback: true,
            },
            memory: MemorySettings {
                stm_enabled: true,
                stm_max_tokens: 4096,
                stm_max_messages: 50,
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
                episodic_memory: true,
                procedural_memory: true,
                working_memory_slots: 7,
                memory_graph_enabled: true,
                graph_max_edges: 50000,
                forgetting_curve: "ebbinghaus".into(),
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
                browser_automation_enabled: false,
                desktop_control_enabled: false,
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
            innovation: InnovationSettings::default(),
            security: SecuritySettings::default(),
            performance: PerformanceSettings::default(),
            per_chat_overrides: HashMap::new(),
            agent_groups: vec![],
            custom_models: vec![],
        }
    }
}

pub fn settings_path() -> PathBuf {
    let mut path = crate::app_paths::app_data_dir();
    path.push("settings.json");
    path
}

pub fn load_settings() -> AppSettings {
    let path = settings_path();
    if path.exists() {
        if let Ok(raw) = fs::read_to_string(&path) {
            let try_plain = serde_json::from_str::<AppSettings>(&raw);
            if let Ok(settings) = try_plain {
                return settings;
            }
            let decrypted = crate::storage_crypto::decrypt_at_rest(&raw, true);
            if let Ok(settings) = serde_json::from_str::<AppSettings>(&decrypted) {
                return settings;
            }
        }
    }
    AppSettings::default()
}

pub fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let path = settings_path();
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    let data = crate::storage_crypto::encrypt_at_rest(&json, settings.security.encrypt_settings);
    fs::write(path, data).map_err(|e| e.to_string())
}

pub fn reset_settings() -> AppSettings {
    let defaults = AppSettings::default();
    let _ = save_settings(&defaults);
    defaults
}
