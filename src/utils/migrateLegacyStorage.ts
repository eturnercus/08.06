/** Перенос ключей localStorage после переименования NeuroForge → Silenium. */
export function migrateLegacyStorage(): void {
  const pairs: [string, string][] = [
    ["neuroforge-lang", "silenium-lang"],
    ["neuroforge-chats", "silenium-chats"],
    ["neuroforge-active-chat", "silenium-active-chat"],
    ["neuroforge-browser-settings", "silenium-browser-settings"],
  ];
  for (const [oldKey, newKey] of pairs) {
    if (!localStorage.getItem(newKey)) {
      const value = localStorage.getItem(oldKey);
      if (value) localStorage.setItem(newKey, value);
    }
  }
}
