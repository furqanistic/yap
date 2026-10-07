import { useId, type ReactNode } from "react";
import { LiquidGlassSurface } from "@/components/liquid-glass";
import "./Settings.css";

interface SettingsGroupProps {
  title: string;
  children: ReactNode;
}

/** A titled card that holds a list of setting rows. */
export function SettingsGroup({ title, children }: SettingsGroupProps) {
  const titleId = useId();

  return (
    <section className="settings-group" aria-labelledby={titleId}>
      <h2 id={titleId} className="settings-group__title">
        {title}
      </h2>
      <LiquidGlassSurface as="div" preset="regular" className="settings-group__card">
        {children}
      </LiquidGlassSurface>
    </section>
  );
}
