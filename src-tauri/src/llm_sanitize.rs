//! Strip ChatML / template leaks and runaway multi-turn generations from model text.

const IM_END: &str = concat!("<|", "im_end", "|>");

const TEMPLATE_MARKERS: &[&str] = &[
    "<|im_start|>",
    IM_END,
    "<|im_end|>",
    "<|redacted_im_start|>",
    "<|eot_id|>",
    "<|endoftext|>",
    "<|begin_of_text|>",
    "<|im",
    "<|redacted",
    "<s>",
    "</s>",
    "[INST]",
    "[/INST]",
    "<<SYS>>",
    "<</SYS>>",
];

const ROLE_LEAK_MARKERS: &[&str] = &[
    "\nuser:",
    "\nassistant:",
    "\nUser:",
    "\nAssistant:",
    "\nsystem:",
    "\nSystem:",
    "\nОтвет:",
    "\nВопрос:",
];

pub fn truncate_at_template_leak(text: &str) -> String {
    let mut cut = text.len();
    for marker in TEMPLATE_MARKERS.iter().chain(ROLE_LEAK_MARKERS.iter()) {
        if marker.is_empty() {
            continue;
        }
        if let Some(i) = text.find(marker) {
            cut = cut.min(i);
        }
    }
    text[..cut].trim().to_string()
}

pub fn generation_should_stop(text: &str) -> bool {
    TEMPLATE_MARKERS
        .iter()
        .filter(|m| !m.is_empty())
        .any(|m| text.contains(m))
        || ROLE_LEAK_MARKERS.iter().any(|m| text.contains(m))
}

pub fn sanitize_llm_output(text: &str) -> String {
    let mut out = truncate_at_template_leak(text);
    for marker in TEMPLATE_MARKERS {
        if !marker.is_empty() {
            out = out.replace(marker, "");
        }
    }
    while out.contains("\n\n\n") {
        out = out.replace("\n\n\n", "\n\n");
    }
    out.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_im_start_leak() {
        let raw = "Яблоко весит около 200 г.\n<|im_start|>user\nУ вас есть яблоко.";
        let s = sanitize_llm_output(raw);
        assert!(!s.contains("im_start"));
        assert!(s.starts_with("Яблоко"));
    }

    #[test]
    fn strips_redacted_template_leak() {
        let raw = "Ответ: одно яблоко.\n<|im_end|>\n<|im_start|>Ответ:\nЯблоко — единица массы.";
        let s = sanitize_llm_output(raw);
        assert!(!s.contains("redacted"));
        assert!(!s.contains("im_start"));
        assert!(s.contains("яблоко"));
    }

    #[test]
    fn strips_im_end_token() {
        let raw = format!("Нормальный ответ.{IM_END}user\nПродолжение");
        let s = sanitize_llm_output(&raw);
        assert_eq!(s, "Нормальный ответ.");
    }
}
