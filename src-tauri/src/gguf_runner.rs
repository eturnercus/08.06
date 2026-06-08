use llama_cpp_4::context::params::LlamaContextParams;
use llama_cpp_4::llama_backend::LlamaBackend;
use llama_cpp_4::llama_batch::LlamaBatch;
use llama_cpp_4::model::params::LlamaModelParams;
use llama_cpp_4::model::{AddBos, LlamaChatMessage, LlamaModel, Special};
use llama_cpp_4::sampling::LlamaSampler;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::Arc;

const BATCH_SIZE: usize = 512;

pub struct GgufRuntime {
    backend: LlamaBackend,
    models: Mutex<HashMap<String, Arc<LlamaModel>>>,
}

pub struct GenerateParams {
    pub model_path: String,
    pub messages: Vec<(String, String)>,
    pub temperature: f32,
    pub max_tokens: u32,
    pub n_ctx: u32,
    pub threads: u32,
    pub gpu_layers: u32,
}

impl GgufRuntime {
    pub fn new() -> Result<Self, String> {
        let mut backend = LlamaBackend::init().map_err(|e| format!("llama backend: {e}"))?;
        backend.void_logs();
        Ok(Self {
            backend,
            models: Mutex::new(HashMap::new()),
        })
    }

    fn load_model(&self, path: &str, gpu_layers: u32) -> Result<Arc<LlamaModel>, String> {
        let key = path.to_string();
        if let Some(m) = self.models.lock().get(&key) {
            return Ok(Arc::clone(m));
        }
        let params = LlamaModelParams::default().with_n_gpu_layers(gpu_layers);
        let model = LlamaModel::load_from_file(&self.backend, Path::new(path), &params)
            .map_err(|e| format!("Не удалось загрузить GGUF: {e}"))?;
        let arc = Arc::new(model);
        self.models.lock().insert(key, Arc::clone(&arc));
        Ok(arc)
    }

    pub fn generate(&self, p: GenerateParams) -> Result<String, String> {
        let path_lower = p.model_path.to_lowercase();
        if path_lower.contains("mmproj") {
            return Err(
                "Файл mmproj — это проектор для изображений, а не языковая модель. \
                 Выберите основной .gguf файл модели (без mmproj в имени)."
                    .into(),
            );
        }

        let model = self.load_model(&p.model_path, p.gpu_layers)?;

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

        let n_ctx = NonZeroU32::new(p.n_ctx.max(2048)).unwrap_or(NonZeroU32::new(2048).unwrap());
        let threads = p.threads.max(1) as i32;
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(Some(n_ctx))
            .with_n_threads(threads)
            .with_n_threads_batch(threads);

        let mut ctx = model
            .new_context(&self.backend, ctx_params)
            .map_err(|e| format!("Контекст модели: {e}"))?;

        let tokens = model
            .str_to_token(&prompt, AddBos::Always)
            .map_err(|e| format!("Токенизация: {e}"))?;

        let max_tokens = p.max_tokens.clamp(16, 4096) as i32;
        let n_ctx_i = ctx.n_ctx() as i32;
        if tokens.len() as i32 >= n_ctx_i {
            return Err("Промпт слишком длинный для контекста модели".into());
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
        let mut sampler = if temp < 0.05 {
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
                "Модель не сгенерировала текст. Проверьте файл модели, RAM и выберите не mmproj."
                    .into(),
            );
        }
        Ok(trimmed)
    }
}

pub fn is_legacy_stub(text: &str) -> bool {
    text.starts_with("NeuroForge | Модель:") || text.starts_with("NeuroForge | Model:")
}
