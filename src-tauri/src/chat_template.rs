//! Build model-specific chat prompts (ChatML for Qwen, etc.).
//! The naive `role: text` format breaks multi-turn dialogs for instruction-tuned GGUF models.

/// Format multi-turn messages into a single prompt ending at the assistant turn.
pub fn format_messages_prompt(
    model_path: &str,
    messages: &[(String, String)],
) -> Result<String, String> {
    if messages.is_empty() {
        return Err("Пустой запрос".into());
    }

    #[cfg(feature = "embedded-llama")]
    if let Ok(prompt) = try_gguf_chat_template(model_path, messages) {
        return Ok(prompt);
    }

    Ok(format_heuristic_prompt(model_path, messages))
}

#[cfg(feature = "embedded-llama")]
fn try_gguf_chat_template(
    model_path: &str,
    messages: &[(String, String)],
) -> Result<String, String> {
    use llama_cpp_4::llama_backend::LlamaBackend;
    use llama_cpp_4::model::params::LlamaModelParams;
    use llama_cpp_4::model::{LlamaChatMessage, LlamaModel};
    use std::path::Path;

    if !Path::new(model_path).exists() {
        return Err("model missing".into());
    }

    let backend = LlamaBackend::init().map_err(|e| format!("llama backend: {e}"))?;
    let model = LlamaModel::load_from_file(
        &backend,
        Path::new(model_path),
        &LlamaModelParams::default(),
    )
    .map_err(|e| format!("template load: {e}"))?;

    let chat: Vec<LlamaChatMessage> = messages
        .iter()
        .filter(|(_, c)| !c.trim().is_empty())
        .map(|(role, content)| {
            LlamaChatMessage::new(role.clone(), content.clone())
                .map_err(|e| format!("chat message: {e}"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    if chat.is_empty() {
        return Err("empty chat".into());
    }

    model
        .apply_chat_template(None, &chat, true)
        .map_err(|e| format!("chat template: {e}"))
}

fn detect_model_family(model_path: &str) -> &'static str {
    let lower = model_path.to_lowercase();
    if lower.contains("qwen") {
        "qwen"
    } else if lower.contains("llama") || lower.contains("meta-llama") {
        "llama3"
    } else if lower.contains("mistral") || lower.contains("mixtral") {
        "mistral"
    } else if lower.contains("phi") {
        "phi"
    } else if lower.contains("gemma") {
        "gemma"
    } else {
        "chatml"
    }
}

fn format_heuristic_prompt(model_path: &str, messages: &[(String, String)]) -> String {
    match detect_model_family(model_path) {
        "llama3" => format_llama3(messages),
        "mistral" => format_mistral(messages),
        "gemma" => format_gemma(messages),
        _ => format_chatml(messages),
    }
}

/// Qwen / ChatML: `<|im_start|>role\ncontent`
fn format_chatml(messages: &[(String, String)]) -> String {
    let mut out = String::new();
    for (role, content) in messages {
        if content.trim().is_empty() {
            continue;
        }
        out.push_str("<|im_start|>");
        out.push_str(role);
        out.push('\n');
        out.push_str(content);
        out.push_str("\n");
    }
    out.push_str("<|im_start|>assistant\n");
    out
}

fn format_llama3(messages: &[(String, String)]) -> String {
    let bos = "<|begin_of_text|>";
    let mut out = String::from(bos);
    for (role, content) in messages {
        if content.trim().is_empty() {
            continue;
        }
        out.push_str("<|start_header_id|>");
        out.push_str(role);
        out.push_str("<|end_header_id|>\n\n");
        out.push_str(content);
        out.push_str("<|eot_id|>");
    }
    out.push_str("<|start_header_id|>assistant<|end_header_id|>\n\n");
    out
}

fn format_mistral(messages: &[(String, String)]) -> String {
    let mut out = String::new();
    for (role, content) in messages {
        if content.trim().is_empty() {
            continue;
        }
        if role == "system" {
            out.push_str("[INST] ");
            out.push_str(content);
            out.push_str(" [/INST]");
        } else if role == "user" {
            if !out.is_empty() {
                out.push(' ');
            }
            out.push_str("[INST] ");
            out.push_str(content);
            out.push_str(" [/INST]");
        } else if role == "assistant" {
            out.push_str(content);
        }
    }
    if !out.ends_with("[/INST]") {
        out.push_str(" [INST] [/INST]");
    }
    out
}

fn format_gemma(messages: &[(String, String)]) -> String {
    let mut out = String::new();
    for (role, content) in messages {
        if content.trim().is_empty() {
            continue;
        }
        out.push_str("<start_of_turn>");
        out.push_str(role);
        out.push('\n');
        out.push_str(content);
        out.push_str("<end_of_turn>\n");
    }
    out.push_str("<start_of_turn>model\n");
    out
}

/// Drop oldest turns until the prompt fits the context budget (char heuristic).
pub fn trim_messages_for_context(
    model_path: &str,
    mut messages: Vec<(String, String)>,
    n_ctx: u32,
    reserve_completion: u32,
) -> Vec<(String, String)> {
    let budget_chars = (n_ctx.saturating_sub(reserve_completion).max(256) as usize) * 3;
    if messages.is_empty() {
        return messages;
    }

    loop {
        let prompt_len = format_messages_prompt(model_path, &messages)
            .map(|s| s.len())
            .unwrap_or_else(|_| estimate_len(&messages));
        if prompt_len <= budget_chars || messages.len() <= 2 {
            break;
        }
        // Keep system message at index 0 if present; drop oldest user/assistant pair after it.
        let start = if messages.first().is_some_and(|(r, _)| r == "system") {
            1
        } else {
            0
        };
        if messages.len() <= start + 1 {
            break;
        }
        messages.remove(start);
        if messages.len() > start && messages[start].0 != "system" {
            messages.remove(start);
        }
    }
    messages
}

fn estimate_len(messages: &[(String, String)]) -> usize {
    messages.iter().map(|(r, c)| r.len() + c.len() + 16).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qwen_chatml_has_assistant_turn() {
        let msgs = vec![
            ("system".into(), "You are helpful.".into()),
            ("user".into(), "Привет".into()),
        ];
        let p = format_heuristic_prompt("model-Qwen2-1.5B.gguf", &msgs);
        assert!(p.contains("<|im_start|>user"));
        assert!(p.ends_with("<|im_start|>assistant\n"));
    }

    #[test]
    fn trim_drops_old_turns() {
        let msgs: Vec<(String, String)> = (0..40)
            .map(|i| {
                if i % 2 == 0 {
                    ("user".into(), format!("msg {i} {}", "x".repeat(200)))
                } else {
                    ("assistant".into(), format!("reply {i}"))
                }
            })
            .collect();
        let trimmed = trim_messages_for_context("qwen.gguf", msgs, 2048, 512);
        assert!(trimmed.len() < 40);
    }
}
