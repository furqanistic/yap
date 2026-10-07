import type { ReactNode } from "react";
import { LiquidGlassSurface } from "@/components/liquid-glass";
import { TitleBar } from "@/components/titlebar";
import "./AppShell.css";

interface AppShellProps {
  sidebar: ReactNode;
  children: ReactNode;
}

export function AppShell({ sidebar, children }: AppShellProps) {
  return (
    <div className="app-shell">
      {sidebar}
      <LiquidGlassSurface as="div" preset="clear" refraction={false} className="app-shell__main">
        <TitleBar />
        <main className="app-shell__content" tabIndex={0}>
          {children}
        </main>
      </LiquidGlassSurface>
    </div>
  );
}
