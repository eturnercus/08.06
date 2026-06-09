import type { ReactNode } from "react";

export function PageIntro({ title, description, children }: { title?: string; description: string; children?: ReactNode }) {
  return (
    <div className="page-intro">
      {title && <h3 className="page-intro-title">{title}</h3>}
      <p className="page-intro-desc">{description}</p>
      {children}
    </div>
  );
}
