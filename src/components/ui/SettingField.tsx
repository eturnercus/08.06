import { Toggle } from "./Toggle";
import { Slider } from "./Slider";

export function SettingToggle({
  title, desc, value, onChange, innovationHint,
}: {
  title: string;
  desc?: string;
  value: boolean;
  onChange: (v: boolean) => void;
  /** Show ! tooltip for experimental Silenium features */
  innovationHint?: boolean;
}) {
  const hint = desc || title;
  return (
    <div className="setting-row">
      <div className="setting-info">
        <div className="setting-title">
          {title}
          {innovationHint && (
            <span
              className="badge badge-innovation feature-hint-badge"
              style={{ marginLeft: 8 }}
              title={hint}
              aria-label={hint}
            >
              !
            </span>
          )}
        </div>
        {desc && <div className="setting-desc">{desc}</div>}
      </div>
      <div className="setting-control"><Toggle checked={value} onChange={onChange} /></div>
    </div>
  );
}

export function SettingNumber({
  title, desc, value, onChange, min, max, step,
}: { title: string; desc?: string; value: number; onChange: (v: number) => void; min?: number; max?: number; step?: number }) {
  return (
    <div className="setting-row">
      <div className="setting-info">
        <div className="setting-title">{title}</div>
        {desc && <div className="setting-desc">{desc}</div>}
      </div>
      <div className="setting-control">
        <input type="number" value={value} min={min} max={max} step={step}
          onChange={(e) => onChange(Number(e.target.value))}
          style={{ width: 100, textAlign: "right" }} className="mono" />
      </div>
    </div>
  );
}

export function SettingSlider({
  title, desc, value, onChange, min, max, step,
}: { title: string; desc?: string; value: number; onChange: (v: number) => void; min?: number; max?: number; step?: number }) {
  return (
    <div className="setting-row">
      <div className="setting-info">
        <div className="setting-title">{title}</div>
        {desc && <div className="setting-desc">{desc}</div>}
      </div>
      <div className="setting-control">
        <Slider value={value} onChange={onChange} min={min} max={max} step={step} showValue />
      </div>
    </div>
  );
}

export function SettingSelect({
  title, desc, value, options, onChange,
}: { title: string; desc?: string; value: string; options: string[] | { v: string; l: string }[]; onChange: (v: string) => void }) {
  const opts = options.map((o) => (typeof o === "string" ? { v: o, l: o } : o));
  return (
    <div className="setting-row">
      <div className="setting-info">
        <div className="setting-title">{title}</div>
        {desc && <div className="setting-desc">{desc}</div>}
      </div>
      <div className="setting-control">
        <select value={value} onChange={(e) => onChange(e.target.value)} style={{ minWidth: 160 }}>
          {opts.map((o) => <option key={o.v} value={o.v}>{o.l}</option>)}
        </select>
      </div>
    </div>
  );
}

export function SettingText({
  title, desc, value, onChange, multiline,
}: { title: string; desc?: string; value: string; onChange: (v: string) => void; multiline?: boolean }) {
  return (
    <div className="field">
      <label className="label">{title}</label>
      {desc && <div className="field-hint" style={{ marginBottom: 8 }}>{desc}</div>}
      {multiline ? (
        <textarea value={value} onChange={(e) => onChange(e.target.value)} rows={3} style={{ width: "100%" }} />
      ) : (
        <input type="text" value={value} onChange={(e) => onChange(e.target.value)} style={{ width: "100%" }} />
      )}
    </div>
  );
}

export function SectionTitle({ children }: { children: React.ReactNode }) {
  return <div className="setting-section-title">{children}</div>;
}
