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

#[derive(Debug, Clone)]
pub struct GenerateResult {
    pub text: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

pub struct GenerateParams {
    pub model_path: String,
    pub messages: Vec<(String, String)>,
    pub temperature: f32,
    pub max_tokens: u32,
    pub top_p: f32,
    pub top_k: u32,
    pub repeat_penalty: f32,
    pub n_ctx: u32,
    pub threads: u32,
    pub gpu_layers: u32,
    pub compute_device: String,
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
        cancel: Option<&std::sync::atomic::AtomicBool>,
    ) -> Result<GenerateResult, String> {
        if p.model_path.to_lowercase().contains("mmproj") {
            return Err(
                "Файл mmproj — проектор для изображений, а не языковая модель. \
                 Выберите основной .gguf (без mmproj в имени)."
                    .into(),
            );
        }

        let gpu_layers = resolve_gpu_layers(
            &p.compute_device,
            p.gpu_layers,
            p.gpu_memory_mb,
            p.vram_reserve_mb,
            p.model_size_bytes,
        );
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

        let n_ctx_i = ctx.n_ctx() as i32;
        let prompt_tokens = tokens.len() as u32;
        if tokens.len() as i32 >= n_ctx_i {
            return Err(
                "Промпт слишком длинный для контекста. Уменьшите историю или max tokens в настройках чата."
                    .into(),
            );
        }
        let room = (n_ctx_i - tokens.len() as i32 - 4).max(16);
        let max_tokens = p.max_tokens.clamp(16, 2048).min(room as u32) as i32;

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
        let repeat = p.repeat_penalty.clamp(1.0, 2.0);
        let mut sampler_chain: Vec<LlamaSampler> =
            vec![LlamaSampler::penalties_simple(64, repeat)];
        if p.top_k > 0 {
            sampler_chain.push(LlamaSampler::top_k(p.top_k.min(200) as i32));
        }
        if p.top_p > 0.0 && p.top_p < 1.0 {
            sampler_chain.push(LlamaSampler::top_p(p.top_p.clamp(0.05, 1.0), 1));
        }
        if temp < 0.05 {
            sampler_chain.push(LlamaSampler::greedy());
        } else {
            sampler_chain.push(LlamaSampler::temp(temp));
            sampler_chain.push(LlamaSampler::dist(0xDEADBEEF));
        }
        let sampler = LlamaSampler::chain_simple(sampler_chain);

        let mut decoder = encoding_rs::UTF_8.new_decoder();
        let mut output = String::new();
        let mut n_cur = batch.n_tokens();
        let n_len = n_cur + max_tokens;
        let mut completion_tokens = 0u32;

        while n_cur < n_len {
            if cancel.is_some_and(|f| f.load(std::sync::atomic::Ordering::SeqCst)) {
                return Err("Генерация остановлена пользователем".into());
            }
            let token = sampler.sample(&ctx, batch.n_tokens() - 1);
            if model.is_eog_token(token) {
                break;
            }

            let bytes = model
                .token_to_bytes(token, Special::Tokenize)
                .map_err(|e| format!("token decode: {e}"))?;
            let mut piece = String::with_capacity(32);
            let _ = decoder.decode_to_string(&bytes, &mut piece, false);
            let tentative = format!("{output}{piece}");
            if crate::llm_sanitize::generation_should_stop(&tentative) {
                output = crate::llm_sanitize::truncate_at_template_leak(&tentative);
                break;
            }
            output.push_str(&piece);
            completion_tokens += 1;
            if let Some(s) = stream.as_mut() {
                let sanitized = crate::llm_sanitize::sanitize_llm_output(&output);
                let already = s.emitted_chars();
                if sanitized.len() > already {
                    s.push(&sanitized[already..]);
                }
            }

            batch.clear();
            batch
                .add(token, n_cur, &[0], true)
                .map_err(|e| format!("batch step: {e}"))?;
            n_cur += 1;
            ctx.decode(&mut batch)
                .map_err(|e| format!("decode step: {e}"))?;
        }

        let trimmed = crate::llm_sanitize::sanitize_llm_output(&output);
        if trimmed.is_empty() {
            return Err(
                "Модель не сгенерировала текст. Попробуйте Q4-квантизацию, включите подкачку (swap) \
                 в настройках или уменьшите контекст."
                    .into(),
            );
        }
        Ok(GenerateResult {
            text: trimmed,
            prompt_tokens,
            completion_tokens,
        })
    }
}

#[cfg(feature = "embedded-llama")]
pub fn generate_with_best_backend(
    embedded: Option<&GgufRuntime>,
    p: GenerateParams,
    stream: Option<&mut dyn TokenSink>,
    cancel: Option<&std::sync::atomic::AtomicBool>,
) -> Result<GenerateResult, String> {
    match stream {
        Some(sink) => generate_with_best_backend_streaming(embedded, p, sink, cancel),
        None => generate_with_best_backend_blocking(embedded, p, cancel),
    }
}

#[cfg(not(feature = "embedded-llama"))]
pub fn generate_with_best_backend(
    p: GenerateParams,
    stream: Option<&mut dyn TokenSink>,
    cancel: Option<&std::sync::atomic::AtomicBool>,
) -> Result<GenerateResult, String> {
    match stream {
        Some(sink) => generate_with_best_backend_streaming(None, p, sink, cancel),
        None => generate_with_best_backend_blocking(None, p, cancel),
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
    cancel: Option<&std::sync::atomic::AtomicBool>,
) -> Result<GenerateResult, String> {
    mmproj_guard(&p.model_path)?;
    let try_cli_first = p.prefer_cli && !p.prefer_embedded;

    if try_cli_first {
        if let Ok(result) = run_cli_backend(&p, Some(sink), cancel) {
            return Ok(result);
        }
        #[cfg(feature = "embedded-llama")]
        if let Some(rt) = embedded {
            if let Ok(result) = rt.generate(p.clone_for_embedded(), Some(sink), cancel) {
                return Ok(result);
            }
        }
    } else {
        #[cfg(feature = "embedded-llama")]
        if let Some(rt) = embedded {
            if let Ok(result) = rt.generate(p.clone_for_embedded(), Some(sink), cancel) {
                return Ok(result);
            }
        }
        if let Ok(result) = run_cli_backend(&p, Some(sink), cancel) {
            return Ok(result);
        }
    }

    Err("Не удалось выполнить инференс ни через embedded, ни через llama-cli.".into())
}

fn generate_with_best_backend_blocking(
    #[cfg(feature = "embedded-llama")] embedded: Option<&GgufRuntime>,
    #[cfg(not(feature = "embedded-llama"))] _embedded: Option<()>,
    p: GenerateParams,
    cancel: Option<&std::sync::atomic::AtomicBool>,
) -> Result<GenerateResult, String> {
    mmproj_guard(&p.model_path)?;
    let try_cli_first = p.prefer_cli && !p.prefer_embedded;

    if try_cli_first {
        if let Ok(result) = run_cli_backend(&p, None, cancel) {
            return Ok(result);
        }
        #[cfg(feature = "embedded-llama")]
        if let Some(rt) = embedded {
            if let Ok(result) = rt.generate(p.clone_for_embedded(), None, cancel) {
                return Ok(result);
            }
        }
    } else {
        #[cfg(feature = "embedded-llama")]
        if let Some(rt) = embedded {
            if let Ok(result) = rt.generate(p.clone_for_embedded(), None, cancel) {
                return Ok(result);
            }
        }
        if let Ok(result) = run_cli_backend(&p, None, cancel) {
            return Ok(result);
        }
    }

    Err("Не удалось выполнить инференс ни через embedded, ни через llama-cli.".into())
}

fn run_cli_backend(
    p: &GenerateParams,
    stream: Option<&mut dyn TokenSink>,
    cancel: Option<&std::sync::atomic::AtomicBool>,
) -> Result<GenerateResult, String> {
    let gpu_layers = resolve_gpu_layers(
        &p.compute_device,
        p.gpu_layers,
        p.gpu_memory_mb,
        p.vram_reserve_mb,
        p.model_size_bytes,
    );
    let mlock = resolve_mlock(p.mlock_enabled, &p.swap_usage, &p.oom_policy);
    let mmap = resolve_mmap(p.mmap_enabled, &p.swap_usage);
    let n_ctx = resolve_context_len(p.n_ctx, p.ram_limit_mb, &p.swap_usage);

    let messages = crate::chat_template::trim_messages_for_context(
        &p.model_path,
        p.messages.clone(),
        n_ctx,
        p.max_tokens.saturating_add(64),
    );
    let prompt = crate::chat_template::format_messages_prompt(&p.model_path, &messages)?;

    let cfg = LlamaCliConfig {
        model_path: p.model_path.clone(),
        prompt: prompt.clone(),
        temperature: p.temperature,
        max_tokens: p.max_tokens,
        top_p: p.top_p,
        top_k: p.top_k,
        repeat_penalty: p.repeat_penalty,
        n_ctx,
        threads: p.threads,
        gpu_layers,
        mlock,
        mmap,
    };

    let text = if let Some(s) = stream {
        let mut cb = |delta: &str| s.push(delta);
        LlamaCliRunner::generate_with_callback(&cfg, Some(&mut cb), cancel)?
    } else {
        LlamaCliRunner::generate_with_callback(&cfg, None, cancel)?
    };
    let trimmed = crate::llm_sanitize::sanitize_llm_output(&text);
    let prompt_tokens = ((prompt.len() as u32) + 3) / 4;
    let completion_tokens = ((trimmed.len() as u32) + 3) / 4;
    Ok(GenerateResult {
        text: trimmed,
        prompt_tokens,
        completion_tokens,
    })
}

impl GenerateParams {
    #[cfg(feature = "embedded-llama")]
    fn clone_for_embedded(&self) -> GenerateParams {
        GenerateParams {
            model_path: self.model_path.clone(),
            messages: self.messages.clone(),
            temperature: self.temperature,
            max_tokens: self.max_tokens,
            top_p: self.top_p,
            top_k: self.top_k,
            repeat_penalty: self.repeat_penalty,
            n_ctx: self.n_ctx,
            threads: self.threads,
            gpu_layers: self.gpu_layers,
            compute_device: self.compute_device.clone(),
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
            top_p: self.top_p,
            top_k: self.top_k,
            repeat_penalty: self.repeat_penalty,
            n_ctx: self.n_ctx,
            threads: self.threads,
            gpu_layers: self.gpu_layers,
            compute_device: self.compute_device.clone(),
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
    text.starts_with("Silenium | Модель:") || text.starts_with("Silenium | Model:")
}
