import type { NavItem, SectionId } from "@/types/navigation";
import { LiquidGlassSurface } from "@/components/liquid-glass";

interface SidebarItemProps {
  item: NavItem;
  active: boolean;
  onSelect: (id: SectionId) => void;
}

export function SidebarItem({ item, active, onSelect }: SidebarItemProps) {
  const Icon = item.icon;

  return (
    <LiquidGlassSurface
      as="button"
      type="button"
      preset="interactive"
      refraction={active}
      className={`sidebar-item${active ? " sidebar-item--active" : ""}`}
      aria-current={active ? "page" : undefined}
      onClick={() => onSelect(item.id)}
    >
      <span className="sidebar-item__icon" aria-hidden="true">
        <Icon className="sidebar-item__glyph" strokeWidth={1.75} />
      </span>
      <span className="sidebar-item__label">{item.label}</span>
    </LiquidGlassSurface>
  );
}
