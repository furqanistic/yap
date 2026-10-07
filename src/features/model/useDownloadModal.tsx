import { createContext, useCallback, useContext, useState, type ReactNode } from "react";
import type { ModelId } from "@/types/models";
import { DownloadModal } from "./DownloadModal";

const DownloadModalContext = createContext<((id: ModelId) => void) | null>(null);

/** Hosts the download modal so any screen can open it. Wrap the app once. */
export function DownloadModalProvider({ children }: { children: ReactNode }) {
  const [modelId, setModelId] = useState<ModelId | null>(null);
  const [open, setOpen] = useState(false);

  const show = useCallback((id: ModelId) => {
    setModelId(id);
    setOpen(true);
  }, []);

  return (
    <DownloadModalContext.Provider value={show}>
      {children}
      {modelId && <DownloadModal modelId={modelId} open={open} onClose={() => setOpen(false)} />}
    </DownloadModalContext.Provider>
  );
}

/** Returns a function that opens the download modal for a model. */
export function useDownloadModal(): (id: ModelId) => void {
  const show = useContext(DownloadModalContext);
  if (!show) throw new Error("useDownloadModal must be used inside <DownloadModalProvider>");
  return show;
}
