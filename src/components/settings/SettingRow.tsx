import { useId, type ReactNode } from "react";
import { LiquidGlassSurface } from "@/components/liquid-glass";

/** Ids a control uses to point screen readers at the row's title and description. */
export interface SettingControlA11y {
  "aria-labelledby": string;
  "aria-describedby": string;
}

interface SettingRowProps {
  title: string;
  description: string;
  /** Small note shown under the control, e.g. what the selected option means. */
  hint?: string;
  children: (a11y: SettingControlA11y) => ReactNode;
}

export function SettingRow({ title, description, hint, children }: SettingRowProps) {
  const titleId = useId();
  const descriptionId = useId();

  return (
    <LiquidGlassSurface
      as="div"
      preset="clear"
      refraction={false}
      className="setting-row"
    >
      <div className="setting-row__text">
        <div id={titleId} className="setting-row__title">
          {title}
        </div>
        <p id={descriptionId} className="setting-row__description">
          {description}
        </p>
      </div>

      <div className="setting-row__control">
        {children({ "aria-labelledby": titleId, "aria-describedby": descriptionId })}
        {hint && <p className="setting-row__hint">{hint}</p>}
      </div>
    </LiquidGlassSurface>
  );
}
