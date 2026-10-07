import { Page } from "@/components/page";
import { LiquidGlassSurface } from "@/components/liquid-glass";
import type { NavItem } from "@/types/navigation";
import "./SectionPlaceholder.css";

interface SectionPlaceholderProps {
  item: NavItem;
}

export function SectionPlaceholder({ item }: SectionPlaceholderProps) {
  const Icon = item.icon;

  return (
    <Page title={item.label} description={item.description}>
      <LiquidGlassSurface as="div" preset="regular" className="section-empty">
        <Icon className="section-empty__icon" strokeWidth={1.5} />
        <p>{item.label} settings are coming soon.</p>
      </LiquidGlassSurface>
    </Page>
  );
}
