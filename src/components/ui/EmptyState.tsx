import type { IconProps } from "@devigner-ui/icons";
import type { ComponentType, ReactNode } from "react";
import "./EmptyState.css";

interface EmptyStateProps {
  icon: ComponentType<IconProps>;
  title: string;
  description?: string;
  /** Usually a single `Button`. */
  action?: ReactNode;
}

/** A card shown when a list or section has nothing in it yet. */
export function EmptyState({ icon: Icon, title, description, action }: EmptyStateProps) {
  return (
    <div className="empty-state">
      <Icon className="empty-state__icon" strokeWidth={1.5} />
      <p className="empty-state__title">{title}</p>
      {description && <p className="empty-state__description">{description}</p>}
      {action && <div className="empty-state__action">{action}</div>}
    </div>
  );
}
