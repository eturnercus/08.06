import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useAppStore } from "../store/appStore";
import { api, LtmEntry, MemoryOverview, StmEntry } from "../api/tauri";

export function MemoryView() {
  const { t } = useTranslation();
  const { chats, activeChatId } = useAppStore();
  const [stm, setStm] = useState<StmEntry[]>([]);
  const [ltm, setLtm] = useState<LtmEntry[]>([]);
  const [overview, setOverview] = useState<MemoryOverview | null>(null);
  const [consolidating, setConsolidating] = useState(false);
  const [consolidateMsg, setConsolidateMsg] = useState<string | null>(null);

  const chatId = activeChatId || chats[0]?.id || "";
  const chat = chats.find((c) => c.id === chatId);
  const modelId = chat?.modelId ?? "default";

  const reload = useCallback(() => {
    if (!chatId) return;
    api.getMemoryStm(chatId).then(setStm).catch(() => setStm([]));
    api.getMemoryLtm(chatId).then(setLtm).catch(() => setLtm([]));
    api.getMemoryOverview(chatId).then(setOverview).catch(() => setOverview(null));
  }, [chatId]);

  useEffect(() => {
    reload();
  }, [reload]);

  const consolidate = async () => {
    if (!chatId || consolidating) return;
    setConsolidating(true);
    setConsolidateMsg(null);
    try {
      await api.consolidateMemory(chatId, modelId);
      setConsolidateMsg(t("memory.consolidateDone"));
      reload();
    } catch (e) {
      setConsolidateMsg(String(e));
    } finally {
      setConsolidating(false);
    }
  };

  return (
    <div className="memory-view">
      <h2>{t("memory.title")}</h2>
      <p className="page-intro-desc memory-intro">{t("memory.pageIntro")}</p>
      <div className="memory-grid">
        <div className="card">
          <h3>{t("memory.stm")}</h3>
          <div className="scroll-y mem-scroll">
            {stm.length === 0 ? (
              <p className="empty">{t("network.noLogs")}</p>
            ) : (
              stm.map((e, i) => (
                <div key={i} className="mem-entry">
                  <span className="badge badge-blue">{e.role}</span>
                  <p>{e.content.slice(0, 200)}</p>
                  <small>{e.tokens} tokens</small>
                </div>
              ))
            )}
          </div>
          <button
            type="button"
            className="btn-secondary mem-action-btn"
            onClick={consolidate}
            disabled={consolidating || stm.length === 0}
          >
            {consolidating ? t("memory.consolidating") : t("memory.consolidate")}
          </button>
          {consolidateMsg && <p className="field-hint">{consolidateMsg}</p>}
        </div>

        <div className="card">
          <h3>{t("memory.ltm")}</h3>
          <div className="scroll-y mem-scroll">
            {ltm.length === 0 ? (
              <p className="empty">{t("network.noLogs")}</p>
            ) : (
              ltm.map((e) => (
                <div key={e.id} className="mem-entry">
                  <p>{e.content.slice(0, 200)}</p>
                  <small>
                    {t("memory.importance")}: {e.importance}{" "}
                    {e.transferable ? "↔" : "🔒"}
                  </small>
                </div>
              ))
            )}
          </div>
        </div>

        <div className="card memory-lane-card">
          <h3>{t("memory.synapticLane")}</h3>
          <p className="field-hint">{t("memory.synapticLaneDesc")}</p>
          {overview && (
            <ul className="memory-lane-stats">
              <li>
                <strong>{t("memory.stm")}:</strong> {overview.stmCount}
              </li>
              <li>
                <strong>{t("memory.ltm")}:</strong> {overview.ltmCount}
              </li>
              <li>
                <strong>{t("memory.bridgeCount")}:</strong> {overview.bridgeCount}
              </li>
            </ul>
          )}
          <p className="field-hint memory-lane-tip">{t("memory.autoBridgeHint")}</p>
        </div>
      </div>
      <style>{`
        .memory-view { padding: 16px; overflow-y: auto; height: 100%; }
        h2 { margin-bottom: 16px; }
        .memory-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 16px; }
        h3 { margin-bottom: 12px; font-size: 14px; }
        .mem-scroll { max-height: 300px; }
        .mem-entry { padding: 8px 0; border-bottom: 1px solid var(--border); }
        .mem-entry p { margin: 4px 0; font-size: 13px; line-height: 1.4; }
        .empty { color: var(--text2); font-size: 13px; }
        .mem-action-btn { margin-top: 8px; }
        .mem-action-btn:focus:not(:focus-visible) { outline: none; box-shadow: none; }
        .memory-lane-stats { margin: 12px 0; padding-left: 18px; font-size: 13px; line-height: 1.6; }
        .memory-lane-tip { margin-top: 8px; }
      `}</style>
    </div>
  );
}
