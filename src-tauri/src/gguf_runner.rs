#[cfg(feature = "embedded-llama")]
use llama_cpp_4::context::params::LlamaContextParams;
#[cfg(feature = "embedded-llama")]
use llama_cpp_4::llama_backend::LlamaBackend;
#[cfg(feature = "embedded-llama")]
use llama_cpp_4::llama_batch::LlamaBatch;
#[cfg(feature = "embedded-llama")]
use llama_cpp_4::model::params::LlamaModelParams;
#[cfg(feature = "embedded-llama")]
use llama_cpp_4::model::{AddBos, LlamaChatMessage, LlamaModel, Special};
#[cfg(feature = "embedded-llama")]
use llama_cpp_4::sampling::LlamaSampler;
#[cfg(feature = "embedded-llama")]
use parking_lot::Mutex;
#[cfg(feature = "embedded-llama")]
use std::collections::HashMap;
#[cfg(feature = "embedded-llama")]
use std::num::NonZeroU32;
#[cfg(feature = "embedded-llama")]
use std::path::Path;
#[cfg(feature = "embedded-llama")]
use std::sync::Arc;

use crate::llama_cli::{
    resolve_context_len, resolve_gpu_layers, resolve_mmap, resolve_mlock, LlamaCliConfig,
    LlamaCliRunner,
};
use crate::stream_sink::TokenSink;

#[cfg(feature = "embedded-llama")]
const BATCH_SIZE: usize = 512;

pub struct GenerateParams {
    pub model_path: String,
    pub messages: Vec<(String, String)>,
    pub temperature: f32,
    pub max_tokens: u32,
    pub n_ctx: u32,
    pub threads: u32,
    pub gpu_layers: u32,
    pub gpu_memory_mb: u64,
    pub vram_reserve_mb: u64,
    pub ram_limit_mb: u64,
    pub mmap_enabled: bool,
    pub mlock_enabled: bool,
    pub swap_usage: String,
    pub oom_policy: String,
    pub kv_offload: bool,
    pub model_size_bytes: u64,
    pub prefer_embedded: bool,
    pub prefer_cli: bool,
}

#[cfg(feature = "embedded-llama")]
pub struct GgufRuntime {
    backend: LlamaBackend,
    models: Mutex<HashMap<String, Arc<LlamaModel>>>,
}

#[cfg(feature = "embedded-llama")]
impl GgufRuntime {
    pub fn new() -> Result<Self, String> {
        let mut backend = LlamaBackend::init().map_err(|e| format!("llama backend: {e}"))?;
        backend.void_logs();
        Ok(Self {
            backend,
            models: Mutex::new(HashMap::new()),
        })
    }

    fn load_model(
        &self,
        path: &str,
        gpu_layers: u32,
        mlock: bool,
    ) -> Result<Arc<LlamaModel>, String> {
        let key = format!("{path}|{gpu_layers}|{mlock}");
        if let Some(m) = self.models.lock().get(&key) {
            return Ok(Arc::clone(m));
        }
        let params = LlamaModelParams::default()
            .with_n_gpu_layers(gpu_layers)
            .with_use_mlock(mlock);
        let model = LlamaModel::load_from_file(&self.backend, Path::new(path), &params)
            .map_err(|e| format!("Не удалось загрузить GGUF: {e}"))?;
        let arc = Arc::new(model);
        self.models.lock().insert(key, Arc::clone(&arc));
        Ok(arc)
    }

    pub fn generate(
        &self,
        p: GenerateParams,
        mut stream: Option<&mut dyn TokenSink>,
    ) -> Result<String, String> {
        if p.model_path.to_lowercase().contains("mmproj") {
            return Err(
                "Файл mmproj — проектор для изображений, а не языковая модель. \
                 Выберите основной .gguf (без mmproj в имени)."
                    .into(),
            );
        }

        let gpu_layers =
            resolve_gpu_layers(p.gpu_layers, p.gpu_memory_mb, p.vram_reserve_mb, p.model_size_bytes);
        let mlock = resolve_mlock(p.mlock_enabled, &p.swap_usage, &p.oom_policy);
        let _mmap = resolve_mmap(p.mmap_enabled, &p.swap_usage);
        let n_ctx = resolve_context_len(p.n_ctx, p.ram_limit_mb, &p.swap_usage);

        let model = self.load_model(&p.model_path, gpu_layers, mlock)?;

        let chat: Vec<LlamaChatMessage> = p
            .messages
            .iter()
            .filter(|(_, c)| !c.trim().is_empty())
            .map(|(role, content)| {
                LlamaChatMessage::new(role.clone(), content.clone())
                    .map_err(|e| format!("сообщение чата: {e}"))
            })
            .collect::<Result<Vec<_>, _>>()?;

        if chat.is_empty() {
            return Err("Пустой запрос".into());
        }

        let prompt = model
            .apply_chat_template(None, &chat, true)
            .unwrap_or_else(|_| {
                let mut s = String::new();
                for (role, content) in &p.messages {
                    s.push_str(role);
                    s.push_str(": ");
                    s.push_str(content);
                    s.push('\n');
                }
                s.push_str("assistant: ");
                s
            });

        let n_ctx_nz =
            NonZeroU32::new(n_ctx.max(2048)).unwrap_or(NonZeroU32::new(2048).unwrap());
        let threads = p.threads.max(1) as i32;
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(Some(n_ctx_nz))
            .with_n_threads(threads)
            .with_n_threads_batch(threads)
            .with_offload_kqv(p.kv_offload);

        let mut ctx = model
            .new_context(&self.backend, ctx_params)
            .map_err(|e| format!("Контекст модели: {e}"))?;

        let tokens = model
            .str_to_token(&prompt, AddBos::Always)
            .map_err(|e| format!("Токенизация: {e}"))?;

        let max_tokens = p.max_tokens.clamp(16, 4096) as i32;
        let n_ctx_i = ctx.n_ctx() as i32;
        if tokens.len() as i32 >= n_ctx_i {
            return Err(
                "Промпт слишком длинный для контекста. Уменьшите историю или max tokens в настройках чата."
                    .into(),
            );
        }

        let mut batch = LlamaBatch::new(BATCH_SIZE, 1);
        let last = (tokens.len() - 1) as i32;
        for (i, token) in (0_i32..).zip(tokens) {
            batch
                .add(token, i, &[0], i == last)
                .map_err(|e| format!("batch: {e}"))?;
        }

        ctx.decode(&mut batch)
            .map_err(|e| format!("decode prompt: {e}"))?;

        let temp = p.temperature.clamp(0.0, 2.0);
        let sampler = if temp < 0.05 {
            LlamaSampler::chain_simple([LlamaSampler::greedy()])
        } else {
            LlamaSampler::chain_simple([LlamaSampler::temp(temp), LlamaSampler::dist(0xDEADBEEF)])
        };

        let mut decoder = encoding_rs::UTF_8.new_decoder();
        let mut output = String::new();
        let mut n_cur = batch.n_tokens();
        let n_len = n_cur + max_tokens;

        while n_cur < n_len {
            let token = sampler.sample(&ctx, batch.n_tokens() - 1);
            if model.is_eog_token(token) {
                break;
            }

            let bytes = model
                .token_to_bytes(token, Special::Tokenize)
                .map_err(|e| format!("token decode: {e}"))?;
            let mut piece = String::with_capacity(32);
            let _ = decoder.decode_to_string(&bytes, &mut piece, false);
            output.push_str(&piece);
            if let Some(s) = stream.as_mut() {
                s.push(&piece);
            }

            batch.clear();
            batch
                .add(token, n_cur, &[0], true)
                .map_err(|e| format!("batch step: {e}"))?;
            n_cur += 1;
            ctx.decode(&mut batch)
                .map_err(|e| format!("decode step: {e}"))?;
        }

        let trimmed = output.trim().to_string();
        if trimmed.is_empty() {
            return Err(
                "Модель не сгенерировала текст. Попробуйте Q4-квантизацию, включите подкачку (swap) \
                 в настройках или уменьшите контекст."
                    .into(),
            );
        }
        Ok(trimmed)
    }
}

#[cfg(feature = "embedded-llama")]
pub fn generate_with_best_backend(
    embedded: Option<&GgufRuntime>,
    p: GenerateParams,
    stream: Option<&mut dyn TokenSink>,
) -> Result<String, String> {
    match stream {
        Some(sink) => generate_with_best_backend_streaming(embedded, p, sink),
        None => generate_with_best_backend_blocking(embedded, p),
    }
}

#[cfg(not(feature = "embedded-llama"))]
pub fn generate_with_best_backend(
    p: GenerateParams,
    stream: Option<&mut dyn TokenSink>,
) -> Result<String, String> {
    match stream {
        Some(sink) => generate_with_best_backend_streaming(None, p, sink),
        None => generate_with_best_backend_blocking(None, p),
    }
}

fn mmproj_guard(path: &str) -> Result<(), String> {
    if path.to_lowercase().contains("mmproj") {
        return Err(
            "Файл mmproj — проектор для изображений, не языковая модель. \
             Скачайте основной .gguf без mmproj в имени."
                .into(),
        );
    }
    Ok(())
}

fn generate_with_best_backend_streaming(
    #[cfg(feature = "embedded-llama")] embedded: Option<&GgufRuntime>,
    #[cfg(not(feature = "embedded-llama"))] _embedded: Option<()>,
    p: GenerateParams,
    sink: &mut dyn TokenSink,
) -> Result<String, String> {
    mmproj_guard(&p.model_path)?;
    let try_cli_first = p.prefer_cli && !p.prefer_embedded;

    if try_cli_first {
        if let Ok(text) = run_cli_backend(&p, Some(sink)) {
            return Ok(text);
        }
        #[cfg(feature = "embedded-llama")]
        if let Some(rt) = embedded {
            if let Ok(text) = rt.generate(p.clone_for_embedded(), Some(sink)) {
                return Ok(text);
            }
        }
    } else {
        #[cfg(feature = "embedded-llama")]
        if let Some(rt) = embedded {
            if let Ok(text) = rt.generate(p.clone_for_embedded(), Some(sink)) {
                return Ok(text);
            }
        }
        if let Ok(text) = run_cli_backend(&p, Some(sink)) {
            return Ok(text);
        }
    }

    Err("Не удалось выполнить инференс ни через embedded, ни через llama-cli.".into())
}

fn generate_with_best_backend_blocking(
    #[cfg(feature = "embedded-llama")] embedded: Option<&GgufRuntime>,
    #[cfg(not(feature = "embedded-llama"))] _embedded: Option<()>,
    p: GenerateParams,
) -> Result<String, String> {
    mmproj_guard(&p.model_path)?;
    let try_cli_first = p.prefer_cli && !p.prefer_embedded;

    if try_cli_first {
        if let Ok(text) = run_cli_backend(&p, None) {
            return Ok(text);
        }
        #[cfg(feature = "embedded-llama")]
        if let Some(rt) = embedded {
            if let Ok(text) = rt.generate(p.clone_for_embedded(), None) {
                return Ok(text);
            }
        }
    } else {
        #[cfg(feature = "embedded-llama")]
        if let Some(rt) = embedded {
            if let Ok(text) = rt.generate(p.clone_for_embedded(), None) {
                return Ok(text);
            }
        }
        if let Ok(text) = run_cli_backend(&p, None) {
            return Ok(text);
        }
    }

    Err("Не удалось выполнить инференс ни через embedded, ни через llama-cli.".into())
}

fn run_cli_backend(
    p: &GenerateParams,
    stream: Option<&mut dyn TokenSink>,
) -> Result<String, String> {
    let gpu_layers =
        resolve_gpu_layers(p.gpu_layers, p.gpu_memory_mb, p.vram_reserve_mb, p.model_size_bytes);
    let mlock = resolve_mlock(p.mlock_enabled, &p.swap_usage, &p.oom_policy);
    let mmap = resolve_mmap(p.mmap_enabled, &p.swap_usage);
    let n_ctx = resolve_context_len(p.n_ctx, p.ram_limit_mb, &p.swap_usage);

    let prompt = p
        .messages
        .iter()
        .map(|(role, content)| format!("{role}: {content}"))
        .collect::<Vec<_>>()
        .join("\n")
        + "\nassistant:";

    let cfg = LlamaCliConfig {
        model_path: p.model_path.clone(),
        prompt,
        temperature: p.temperature,
        max_tokens: p.max_tokens,
        n_ctx,
        threads: p.threads,
        gpu_layers,
        mlock,
        mmap,
    };

    if let Some(s) = stream {
        let mut cb = |delta: &str| s.push(delta);
        LlamaCliRunner::generate_with_callback(&cfg, Some(&mut cb))
    } else {
        LlamaCliRunner::generate(&cfg)
    }
}

impl GenerateParams {
    #[cfg(feature = "embedded-llama")]
    fn clone_for_embedded(&self) -> GenerateParams {
        GenerateParams {
            model_path: self.model_path.clone(),
            messages: self.messages.clone(),
            temperature: self.temperature,
            max_tokens: self.max_tokens,
            n_ctx: self.n_ctx,
            threads: self.threads,
            gpu_layers: self.gpu_layers,
            gpu_memory_mb: self.gpu_memory_mb,
            vram_reserve_mb: self.vram_reserve_mb,
            ram_limit_mb: self.ram_limit_mb,
            mmap_enabled: self.mmap_enabled,
            mlock_enabled: self.mlock_enabled,
            swap_usage: self.swap_usage.clone(),
            oom_policy: self.oom_policy.clone(),
            kv_offload: self.kv_offload,
            model_size_bytes: self.model_size_bytes,
            prefer_embedded: self.prefer_embedded,
            prefer_cli: self.prefer_cli,
        }
    }
}

impl Clone for GenerateParams {
    fn clone(&self) -> Self {
        GenerateParams {
            model_path: self.model_path.clone(),
            messages: self.messages.clone(),
            temperature: self.temperature,
            max_tokens: self.max_tokens,
            n_ctx: self.n_ctx,
            threads: self.threads,
            gpu_layers: self.gpu_layers,
            gpu_memory_mb: self.gpu_memory_mb,
            vram_reserve_mb: self.vram_reserve_mb,
            ram_limit_mb: self.ram_limit_mb,
            mmap_enabled: self.mmap_enabled,
            mlock_enabled: self.mlock_enabled,
            swap_usage: self.swap_usage.clone(),
            oom_policy: self.oom_policy.clone(),
            kv_offload: self.kv_offload,
            model_size_bytes: self.model_size_bytes,
            prefer_embedded: self.prefer_embedded,
            prefer_cli: self.prefer_cli,
        }
    }
}

pub fn is_legacy_stub(text: &str) -> bool {
    text.starts_with("NeuroForge | Модель:") || text.starts_with("NeuroForge | Model:")
}
