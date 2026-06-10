//! Mixture of Experts (MoE) detection and user-facing hints.
//! Concepts adapted from: https://habr.com/ru/articles/879494/

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoeInfo {
    pub is_moe: bool,
    /// `sparse_moe` | `dense`
    pub architecture: String,
    pub expert_count: Option<u32>,
    /// Experts activated per token (sparse routing).
    pub active_experts: Option<u32>,
    /// Approximate share of weights used per forward pass.
    pub active_ratio: Option<f32>,
    pub hint_ru: String,
    pub hint_en: String,
    pub family: Option<String>,
}

impl MoeInfo {
    pub fn dense() -> Self {
        Self {
            is_moe: false,
            architecture: "dense".into(),
            expert_count: None,
            active_experts: None,
            active_ratio: None,
            hint_ru: "Плотная (dense) модель: все параметры участвуют в каждом токене.".into(),
            hint_en: "Dense model: all parameters participate in every token.".into(),
            family: None,
        }
    }
}

/// Detect MoE architecture from model file name or path (GGUF catalog).
pub fn detect_moe(name_or_path: &str) -> MoeInfo {
    let lower = name_or_path.to_lowercase();
    let base = lower
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(&lower)
        .trim_end_matches(".gguf")
        .trim_end_matches(".ggml");

    if let Some(info) = detect_mixtral(base) {
        return info;
    }
    if let Some(info) = detect_deepseek_moe(base) {
        return info;
    }
    if let Some(info) = detect_phi_moe(base) {
        return info;
    }
    if let Some(info) = detect_generic_moe(base) {
        return info;
    }

    MoeInfo::dense()
}

fn sparse_moe(
    family: &str,
    expert_count: u32,
    active_experts: u32,
    note_ru: &str,
    note_en: &str,
) -> MoeInfo {
    let active_ratio = if expert_count > 0 {
        Some((active_experts as f32 / expert_count as f32).clamp(0.05, 1.0))
    } else {
        None
    };
    let ratio_pct = active_ratio
        .map(|r| (r * 100.0).round() as u32)
        .unwrap_or(0);
    MoeInfo {
        is_moe: true,
        architecture: "sparse_moe".into(),
        expert_count: Some(expert_count),
        active_experts: Some(active_experts),
        active_ratio,
        hint_ru: format!(
            "Sparse MoE ({family}): {expert_count} экспертов, ~{active_experts} активны на токен (~{ratio_pct}% весов). {note_ru}"
        ),
        hint_en: format!(
            "Sparse MoE ({family}): {expert_count} experts, ~{active_experts} active per token (~{ratio_pct}% weights). {note_en}"
        ),
        family: Some(family.into()),
    }
}

fn detect_mixtral(base: &str) -> Option<MoeInfo> {
    if !base.contains("mixtral") {
        return None;
    }
    let (experts, size) = parse_nx_m_size(base).unwrap_or((8, 7));
    let _ = size;
    Some(sparse_moe(
        "Mixtral",
        experts,
        2,
        "Как в статье на Habr: routing выбирает пару экспертов — инференс быстрее при сопоставимом качестве.",
        "Per Habr: routing picks a pair of experts — faster inference at similar quality.",
    ))
}

fn detect_deepseek_moe(base: &str) -> Option<MoeInfo> {
    if !base.contains("deepseek") {
        return None;
    }
    let is_moe = base.contains("v3")
        || base.contains("v2")
        || base.contains("moe")
        || base.contains("671b")
        || base.contains("236b");
    if !is_moe {
        return None;
    }
    Some(MoeInfo {
        is_moe: true,
        architecture: "sparse_moe".into(),
        expert_count: Some(256),
        active_experts: Some(8),
        active_ratio: Some(37.0 / 671.0),
        hint_ru: "DeepSeek MoE: сотни «экспертов» в архитектуре, на токен активна лишь малая доля параметров (~5% по Habr).".into(),
        hint_en: "DeepSeek MoE: hundreds of experts in architecture, only a small fraction active per token (~5% per Habr).".into(),
        family: Some("DeepSeek".into()),
    })
}

fn detect_phi_moe(base: &str) -> Option<MoeInfo> {
    if base.contains("phi") && base.contains("moe") {
        return Some(sparse_moe(
            "Phi-MoE",
            16,
            2,
            "Microsoft Phi MoE — sparse routing, как в обзоре Mixtral.",
            "Microsoft Phi MoE — sparse routing, as in the Mixtral overview.",
        ));
    }
    None
}

fn detect_generic_moe(base: &str) -> Option<MoeInfo> {
    if base.contains("moe")
        || base.contains("mixture-of-experts")
        || base.contains("mixture_of_experts")
    {
        let (experts, _) = parse_nx_m_size(base).unwrap_or((8, 7));
        return Some(sparse_moe(
            "MoE",
            experts,
            2.min(experts),
            "Модель с mixture-of-experts: gating/routing включает часть экспертов.",
            "Mixture-of-experts model: gating/routing activates a subset of experts.",
        ));
    }
    if let Some((experts, _)) = parse_nx_m_size(base) {
        if experts >= 2 {
            return Some(sparse_moe(
                "NxM",
                experts,
                2.min(experts),
                "Имя вида 8x7B часто указывает на MoE (несколько экспертов).",
                "Names like 8x7B often indicate MoE (multiple experts).",
            ));
        }
    }
    None
}

/// Parse `8x7b`, `8x22b` style expert×size notation.
fn parse_nx_m_size(s: &str) -> Option<(u32, u32)> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i + 2 < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            if i < bytes.len() && (bytes[i] == b'x' || bytes[i] == b'X') {
                let experts: u32 = s[start..i].parse().ok()?;
                i += 1;
                let size_start = i;
                while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
                    i += 1;
                }
                if i > size_start {
                    let size_str = &s[size_start..i];
                    let size: u32 = size_str
                        .trim_end_matches('b')
                        .parse()
                        .unwrap_or(7);
                    return Some((experts, size));
                }
            }
        } else {
            i += 1;
        }
    }
    None
}

pub fn attach_moe(info: &mut crate::inference::ModelInfo) {
    let moe = detect_moe(&format!("{} {}", info.name, info.path));
    if moe.is_moe {
        info.moe = Some(moe);
    } else {
        info.moe = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_mixtral() {
        let m = detect_moe("Mixtral-8x7B-Instruct-v0.1.Q4_K_M.gguf");
        assert!(m.is_moe);
        assert_eq!(m.expert_count, Some(8));
        assert_eq!(m.active_experts, Some(2));
    }

    #[test]
    fn detects_deepseek_v3() {
        let m = detect_moe("DeepSeek-V3-0324-Q4_K_M.gguf");
        assert!(m.is_moe);
        assert_eq!(m.family.as_deref(), Some("DeepSeek"));
    }

    #[test]
    fn dense_qwen_not_moe() {
        let m = detect_moe("Qwen2.5-1.5B-Instruct-Q4_K_M.gguf");
        assert!(!m.is_moe);
    }

    #[test]
    fn parses_8x7_notation() {
        assert_eq!(parse_nx_m_size("mixtral-8x7b-instruct"), Some((8, 7)));
    }
}
