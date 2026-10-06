import { useState, type ReactNode } from "react";
import { AppShell } from "@/components/layout";
import { SectionPlaceholder } from "@/components/section-placeholder";
import { Sidebar } from "@/components/sidebar";
import { AboutSettings } from "@/features/about";
import { GeneralSettings } from "@/features/general";
import { LanguageSettings } from "@/features/language";
import { ModelSettings } from "@/features/model";
import type { SectionId } from "@/types/navigation";
import { NAV_ITEMS, PRIMARY_NAV, SECONDARY_NAV } from "./navigation";

/** Sections that have a real page; the rest show a placeholder for now. */
const SECTION_PAGES: Partial<Record<SectionId, ReactNode>> = {
  general: <GeneralSettings />,
  model: <ModelSettings />,
  language: <LanguageSettings />,
  about: <AboutSettings />,
};

function App() {
  const [activeId, setActiveId] = useState<SectionId>("general");
  const activeItem = NAV_ITEMS.find((item) => item.id === activeId) ?? NAV_ITEMS[0];

  return (
    <AppShell
      sidebar={
        <Sidebar
          primaryItems={PRIMARY_NAV}
          secondaryItems={SECONDARY_NAV}
          activeId={activeId}
          onSelect={setActiveId}
        />
      }
    >
      {SECTION_PAGES[activeId] ?? <SectionPlaceholder item={activeItem} />}
    </AppShell>
  );
}

export default App;
