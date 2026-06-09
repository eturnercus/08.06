/** Strip ChatML / template leaks from streamed or final model text (mirrors Rust llm_sanitize). */

const IM_END = "<|" + "im_end" + "|>";

const TEMPLATE_MARKERS = [
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

const ROLE_LEAK_MARKERS = [
  "\nuser:",
  "\nassistant:",
  "\nUser:",
  "\nAssistant:",
  "\nsystem:",
  "\nSystem:",
  "\nОтвет:",
  "\nВопрос:",
];

export function truncateAtTemplateLeak(text: string): string {
  let cut = text.length;
  for (const marker of [...TEMPLATE_MARKERS, ...ROLE_LEAK_MARKERS]) {
    if (!marker) continue;
    const i = text.indexOf(marker);
    if (i >= 0) cut = Math.min(cut, i);
  }
  return text.slice(0, cut).trim();
}

export function generationShouldStop(text: string): boolean {
  return (
    TEMPLATE_MARKERS.filter(Boolean).some((m) => text.includes(m)) ||
    ROLE_LEAK_MARKERS.some((m) => text.includes(m))
  );
}

export function sanitizeLlmOutput(text: string): string {
  let out = truncateAtTemplateLeak(text);
  for (const marker of TEMPLATE_MARKERS) {
    if (marker) out = out.split(marker).join("");
  }
  while (out.includes("\n\n\n")) {
    out = out.replace(/\n\n\n/g, "\n\n");
  }
  return out.trim();
}

/** Apply delta to accumulated text; returns sanitized text (stops growth at template leak). */
export function appendSanitizedDelta(accumulated: string, delta: string): string {
  const combined = accumulated + delta;
  if (generationShouldStop(combined)) {
    return sanitizeLlmOutput(truncateAtTemplateLeak(combined));
  }
  return sanitizeLlmOutput(combined);
}
