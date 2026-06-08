import { useTranslation } from "react-i18next";
import { useModels } from "../../hooks/useModels";
import { Tooltip } from "../ui/Tooltip";

export function ModelSelect({
  value,
  onChange,
  label,
}: {
  value: string;
  onChange: (id: string) => void;
  label?: string;
}) {
  const { t } = useTranslation();
  const { models, loading } = useModels();

  return (
    <div className="form-row">
      <Tooltip text={t("models.selectTip")}>
        <label className="form-label">{label ?? t("models.select")} ⓘ</label>
      </Tooltip>
      <select
        className="m3-input"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        disabled={loading}
      >
        {models.map((m) => {
          const isMmproj = m.name.toLowerCase().includes("mmproj") || (m.path ?? "").toLowerCase().includes("mmproj");
          return (
            <option key={m.id} value={m.id} disabled={isMmproj}>
              {m.name}{isMmproj ? ` (${t("models.mmprojWarn")})` : ""} ({m.source}{m.verified === false ? " ⚠" : ""}{m.sizeBytes ? ` · ${(m.sizeBytes / 1048576).toFixed(0)}MB` : ""})
            </option>
          );
        })}
        {models.length === 0 && <option value="default">{t("models.builtin")}</option>}
      </select>
    </div>
  );
}
