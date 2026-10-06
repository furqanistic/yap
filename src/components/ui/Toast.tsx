import { createContext, useCallback, useContext, useEffect, useRef, useState, type ReactNode } from "react";
import "./Toast.css";

const DURATION_MS = 4000;

type ShowToast = (message: string) => void;

const ToastContext = createContext<ShowToast | null>(null);

/** Hosts the single toast slot. Wrap the app once, near the root. */
export function ToastProvider({ children }: { children: ReactNode }) {
  const [toast, setToast] = useState<{ id: number; message: string } | null>(null);
  const nextId = useRef(0);

  const show = useCallback<ShowToast>((message) => {
    nextId.current += 1;
    setToast({ id: nextId.current, message });
  }, []);

  useEffect(() => {
    if (!toast) return;
    const timer = window.setTimeout(() => setToast(null), DURATION_MS);
    return () => window.clearTimeout(timer);
  }, [toast]);

  return (
    <ToastContext.Provider value={show}>
      {children}
      {/* Always mounted, so screen readers pick up new messages */}
      <div className="toast-region" role="status" aria-live="polite">
        {toast && (
          <div key={toast.id} className="toast">
            {toast.message}
          </div>
        )}
      </div>
    </ToastContext.Provider>
  );
}

/** Returns a function that shows a short message at the bottom of the window. */
export function useToast(): ShowToast {
  const show = useContext(ToastContext);
  if (!show) throw new Error("useToast must be used inside <ToastProvider>");
  return show;
}
