import { createContext, useContext } from "react";
import type { SectionId } from "@/types/navigation";

/** Lets any page switch the sidebar section, e.g. "Manage models" → Model. */
export const NavigationContext = createContext<(id: SectionId) => void>(() => {});

export function useNavigation() {
  return useContext(NavigationContext);
}
