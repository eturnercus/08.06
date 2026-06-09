import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useAppStore } from "../store/appStore";
import { api, LtmEntry, StmEntry } from "../api/tauri";

export function MemoryView() {
  const { t } = useTranslation();
  const { chats, activeChatId } = useAppStore();
  const [stm, setStm] = useState<StmEntry[]>([]);
  const [ltm, setLtm] = useState<LtmEntry[]>([]);
  const [fromChat, setFromChat] = useState("");
  const [toChat, setToChat] = useState("");

  const chatId = activeChatId || chats[0]?.id || "";

  useEffect(() => {
    if (chatId) {
      api.getMemoryStm(chatId).then(setStm).catch(() => {});
      api.getMemoryLtm(chatId).then(setLtm).catch(() => {});
    }
  }, [chatId]);

  const transfer = async () => {
    if (!fromChat || !toChat || ltm.length === 0) return;
    const ids = ltm.filter((e) => e.transferable).map((e) => e.id);
    await api.transferMemory({
      entryIds: ids.slice(0, 3),
      fromChat,
      toChat,
      fromModel: "default",
      toModel: "default",
      memoryType: "long_term",
    });
    api.getMemoryLtm(toChat).then(setLtm);
  };

  const consolidate = async () => {
    if (!chatId) return;
    await api.consolidateMemory(chatId, "default");
    api.getMemoryLtm(chatId).then(setLtm);
    api.getMemoryStm(chatId).then(setStm);
  };

  return (
    <div className="memory-view">
      <h2>{t("memory.title")}</h2>
      <div className="memory-grid">
        <div className="card">
          <h3>{t("memory.stm")}</h3>
          <div className="scroll-y" style={{ maxHeight: 300 }}>
            {stm.length === 0 ? <p className="empty">{t("network.noLogs")}</p> : stm.map((e, i) => (
              <div key={i} className="mem-entry">
                <span className="badge badge-blue">{e.role}</span>
                <p>{e.content.slice(0, 200)}</p>
                <small>{e.tokens} tokens</small>
              </div>
            ))}
          </div>
          <button className="btn-secondary" onClick={consolidate} style={{ marginTop: 8 }}>{t("memory.consolidate")}</button>
        </div>
        <div className="card">
          <h3>{t("memory.ltm")}</h3>
          <div className="scroll-y" style={{ maxHeight: 300 }}>
            {ltm.length === 0 ? <p className="empty">{t("network.noLogs")}</p> : ltm.map((e) => (
              <div key={e.id} className="mem-entry">
                <p>{e.content.slice(0, 200)}</p>
                <small>importance: {e.importance} {e.transferable ? "↔" : "🔒"}</small>
              </div>
            ))}
          </div>
        </div>
        <div className="card">
          <h3>{t("memory.transfer")}</h3>
          <TextSelect label={t("memory.fromChat")} value={fromChat} options={chats.map((c) => c.id)} onChange={setFromChat} />
          <TextSelect label={t("memory.toChat")} value={toChat} options={chats.map((c) => c.id)} onChange={setToChat} />
          <button className="btn-primary" onClick={transfer}>{t("memory.transfer")}</button>
        </div>
      </div>
      <style>{`
        .memory-view { padding: 16px; overflow-y: auto; height: 100%; }
        h2 { margin-bottom: 16px; }
        .memory-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 16px; }
        h3 { margin-bottom: 12px; font-size: 14px; }
        .mem-entry { padding: 8px 0; border-bottom: 1px solid var(--border); }
        .mem-entry p { margin: 4px 0; font-size: 13px; line-height: 1.4; }
        .empty { color: var(--text2); font-size: 13px; }
      `}</style>
    </div>
  );
}

function TextSelect({ label, value, options, onChange }: { label: string; value: string; options: string[]; onChange: (v: string) => void }) {
  return (
    <div className="field">
      <label className="label">{label}</label>
      <select value={value} onChange={(e) => onChange(e.target.value)} style={{ width: "100%" }}>
        <option value="">—</option>
        {options.map((o) => <option key={o} value={o}>{o}</option>)}
      </select>
    </div>
  );
}
