// Per-icon imports: the package barrel is ~6 MB and isn't tree-shaken by our bundler.
import { IconBookMinimalistic } from "@devigner-ui/icons/BookMinimalistic";
import { IconHistory } from "@devigner-ui/icons/History";
import { IconInfoCircle } from "@devigner-ui/icons/InfoCircle";
import { IconLanguageCircle } from "@devigner-ui/icons/LanguageCircle";
import { IconRocket } from "@devigner-ui/icons/Rocket";
import { IconSettings } from "@devigner-ui/icons/Settings";
import { IconStarsMinimalistic } from "@devigner-ui/icons/StarsMinimalistic";
import { IconTuning2 } from "@devigner-ui/icons/Tuning2";
import type { NavItem } from "@/types/navigation";

/** Main sections, shown at the top of the sidebar. */
export const PRIMARY_NAV: NavItem[] = [
  {
    id: "general",
    label: "General",
    description: "Shortcuts, startup behavior, and everyday preferences.",
    icon: IconSettings,
  },
  {
    id: "onboarding",
    label: "Onboarding",
    description: "Set up permissions and learn the basics of Yap.",
    icon: IconRocket,
  },
  {
    id: "model",
    label: "Model",
    description: "Download and choose the speech model Yap uses.",
    icon: IconStarsMinimalistic,
  },
  {
    id: "language",
    label: "Language",
    description: "Pick the languages you dictate in.",
    icon: IconLanguageCircle,
  },
  {
    id: "dictionary",
    label: "Dictionary",
    description: "Teach Yap names, jargon, and custom words.",
    icon: IconBookMinimalistic,
  },
  {
    id: "history",
    label: "History",
    description: "Review and copy your past dictations.",
    icon: IconHistory,
  },
];

/** Secondary sections, pinned to the bottom of the sidebar. */
export const SECONDARY_NAV: NavItem[] = [
  {
    id: "advanced",
    label: "Advanced",
    description: "Power-user options and diagnostics.",
    icon: IconTuning2,
  },
  {
    id: "about",
    label: "About",
    description: "Version, license, and credits.",
    icon: IconInfoCircle,
  },
];

export const NAV_ITEMS: NavItem[] = [...PRIMARY_NAV, ...SECONDARY_NAV];
