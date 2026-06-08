import { ReactNode } from "react";

export function Tooltip({ text, children }: { text: string; children: ReactNode }) {
  return (
    <span className="m3-tooltip-wrap">
      {children}
      <span className="m3-tip">{text}</span>
    </span>
  );
}
