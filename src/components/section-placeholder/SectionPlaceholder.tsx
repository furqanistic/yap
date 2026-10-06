import { Page } from "@/components/page";
import { EmptyState } from "@/components/ui";
import type { NavItem } from "@/types/navigation";

interface SectionPlaceholderProps {
  item: NavItem;
}

export function SectionPlaceholder({ item }: SectionPlaceholderProps) {
  return (
    <Page title={item.label} description={item.description}>
      <EmptyState icon={item.icon} title="Coming soon" description={`${item.label} settings are on the way.`} />
    </Page>
  );
}
