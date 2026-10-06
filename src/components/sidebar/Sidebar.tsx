import { IconSoundwave } from "@devigner-ui/icons/Soundwave";
import type { NavItem, SectionId } from "@/types/navigation";
import { SidebarItem } from "./SidebarItem";
import "./Sidebar.css";

interface SidebarProps {
  primaryItems: NavItem[];
  secondaryItems: NavItem[];
  activeId: SectionId;
  onSelect: (id: SectionId) => void;
}

export function Sidebar({ primaryItems, secondaryItems, activeId, onSelect }: SidebarProps) {
  const renderItems = (items: NavItem[]) =>
    items.map((item) => (
      <li key={item.id}>
        <SidebarItem item={item} active={item.id === activeId} onSelect={onSelect} />
      </li>
    ));

  return (
    <aside className="sidebar">
      {/* Draggable top strip; on macOS the traffic lights sit here */}
      <div className="sidebar__titlebar" data-tauri-drag-region />

      <div className="sidebar__brand">
        <span className="sidebar__logo" aria-hidden="true">
          <IconSoundwave className="sidebar__logo-icon" strokeWidth={2} />
        </span>
        <span className="sidebar__name">Yap</span>
      </div>

      <nav className="sidebar__nav" aria-label="Settings">
        <ul className="sidebar__list sidebar__list--primary">{renderItems(primaryItems)}</ul>
        <ul className="sidebar__list sidebar__list--bottom">{renderItems(secondaryItems)}</ul>
      </nav>
    </aside>
  );
}
