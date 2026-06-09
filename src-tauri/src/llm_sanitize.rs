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

/// Bracketed innovation/injection tags that small models often echo from the system prompt.
const INNOVATION_LEAK_MARKERS: &[&str] = &[
    "[context DNA]",
    "[temporal anchor]",
    "[thought stream",
    "[holographic context]",
    "[quantum layers]",
    "[attention cascade]",
    "[emotion mirror",
    "[persona blend",
    "[meta-cognition]",
    "[whisper mode]",
    "[neural mesh]",
    "[ambient harvest]",
    "[resonance ",
];

pub fn truncate_at_template_leak(text: &str) -> String {
    let mut cut = text.len();
    for marker in TEMPLATE_MARKERS
        .iter()
        .chain(ROLE_LEAK_MARKERS.iter())
        .chain(INNOVATION_LEAK_MARKERS.iter())
    {
        if marker.is_empty() {
            continue;
        }
        if let Some(i) = text.find(marker) {
            cut = cut.min(i);
        }
    }
    text[..cut].trim().to_string()
}

/// Detect runaway loops where the model repeats the same phrase many times.
pub fn detect_repetition_loop(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() < 40 {
        return false;
    }
    let start = chars.len().saturating_sub(250);
    let tail: String = chars[start..].iter().collect();
    for len in (8usize..=36).rev() {
        if tail.chars().count() < len * 3 {
            continue;
        }
        let sample: String = tail.chars().rev().take(len).collect::<Vec<_>>().into_iter().rev().collect();
        let sample = sample.trim();
        if sample.chars().count() < 6 {
            continue;
        }
        if tail.matches(sample).count() >= 3 {
            return true;
        }
    }
    false
}

pub fn generation_should_stop(text: &str) -> bool {
    TEMPLATE_MARKERS
        .iter()
        .filter(|m| !m.is_empty())
        .any(|m| text.contains(m))
        || ROLE_LEAK_MARKERS.iter().any(|m| text.contains(m))
        || INNOVATION_LEAK_MARKERS.iter().any(|m| text.contains(m))
        || detect_repetition_loop(text)
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

    #[test]
    fn detects_repetition_loop() {
        let phrase = "Вариант 1: 1 яблоко = 1000 грамм. ";
        let raw = phrase.repeat(5);
        assert!(detect_repetition_loop(&raw));
    }

    #[test]
    fn strips_innovation_context_dna_leak() {
        let raw = "Test:[context DNA] 6af507f8cd7d77cc[temporal anchor] Учитывай контекст";
        let s = sanitize_llm_output(raw);
        assert!(!s.contains("[context DNA]"));
        assert!(!s.contains("[temporal anchor]"));
        assert_eq!(s, "Test:");
    }

    #[test]
    fn stops_on_innovation_marker_during_generation() {
        let raw = "Ответ.[context DNA] 6af507f8";
        assert!(generation_should_stop(raw));
    }
}
