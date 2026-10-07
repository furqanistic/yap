import type { ReactNode } from "react";
import "./Tag.css";

interface TagProps {
  children: ReactNode;
  /** Lime-soft background, for "In use" or "Recommended". */
  selected?: boolean;
}

/** Small pill label. */
export function Tag({ children, selected }: TagProps) {
  return <span className={`tag${selected ? " tag--selected" : ""}`}>{children}</span>;
}
