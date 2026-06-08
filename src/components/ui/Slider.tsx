import "./ui.css";

interface SliderProps {
  value: number;
  onChange: (v: number) => void;
  min?: number;
  max?: number;
  step?: number;
  showValue?: boolean;
}

export function Slider({ value, onChange, min = 0, max = 100, step = 1, showValue }: SliderProps) {
  const pct = ((value - min) / (max - min)) * 100;
  return (
    <div className="nf-slider-wrap">
      <input
        type="range"
        className="nf-slider"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
        style={{ "--pct": `${pct}%` } as React.CSSProperties}
      />
      {showValue && <span className="nf-slider-val mono">{value}</span>}
    </div>
  );
}
