import { useTranslation } from "react-i18next";

const EXAMPLE_KEYS = ["mixtral", "deepseek", "phi"] as const;

export function MoeGuide() {
  const { t } = useTranslation();
  return (
    <article className="m3-card help-moe-guide">
      <header className="help-moe-header">
        <h3>{t("help.moe.title")}</h3>
        <p className="help-moe-sub">{t("help.moe.subtitle")}</p>
      </header>

      <section className="help-moe-section">
        <h4>{t("help.moe.whatTitle")}</h4>
        <p>{t("help.moe.whatBody")}</p>
        <div className="help-moe-diagram" aria-hidden>
          <div className="help-moe-flow">
            <span className="help-moe-node">{t("help.moe.diagInput")}</span>
            <span className="help-moe-arrow">→</span>
            <span className="help-moe-node help-moe-gate">{t("help.moe.diagGating")}</span>
            <span className="help-moe-arrow">→</span>
            <span className="help-moe-experts">
              {["E1", "E2", "E3", "E4"].map((e) => (
                <span key={e} className="help-moe-expert">{e}</span>
              ))}
            </span>
            <span className="help-moe-arrow">→</span>
            <span className="help-moe-node">{t("help.moe.diagOutput")}</span>
          </div>
          <p className="help-moe-diagram-caption">{t("help.moe.diagCaption")}</p>
        </div>
      </section>

      <section className="help-moe-section help-moe-warn">
        <h4>{t("help.moe.notAgentsTitle")}</h4>
        <p>{t("help.moe.notAgentsBody")}</p>
      </section>

      <section className="help-moe-section">
        <h4>{t("help.moe.sparseTitle")}</h4>
        <p>{t("help.moe.sparseBody")}</p>
        <ul className="help-moe-list">
          <li>{t("help.moe.sparsePoint1")}</li>
          <li>{t("help.moe.sparsePoint2")}</li>
          <li>{t("help.moe.sparsePoint3")}</li>
        </ul>
      </section>

      <section className="help-moe-section">
        <h4>{t("help.moe.examplesTitle")}</h4>
        <ul className="help-moe-list">
          {EXAMPLE_KEYS.map((key) => (
            <li key={key}>
              <strong>{t(`help.moe.examples.${key}.name`)}</strong>
              {" — "}
              {t(`help.moe.examples.${key}.desc`)}
            </li>
          ))}
        </ul>
      </section>

      <section className="help-moe-section help-moe-silenium">
        <h4>{t("help.moe.sileniumTitle")}</h4>
        <p>{t("help.moe.sileniumBody")}</p>
        <ul className="help-moe-list">
          <li>{t("help.moe.sileniumPoint1")}</li>
          <li>{t("help.moe.sileniumPoint2")}</li>
          <li>{t("help.moe.sileniumPoint3")}</li>
        </ul>
      </section>

      <footer className="help-moe-source">
        <p>{t("help.moe.sourceNote")}</p>
        <a
          href="https://habr.com/ru/articles/879494/"
          target="_blank"
          rel="noopener noreferrer"
        >
          {t("help.moe.sourceLink")}
        </a>
      </footer>
    </article>
  );
}
