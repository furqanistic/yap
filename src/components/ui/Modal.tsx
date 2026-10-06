import { useCallback, useEffect, useId, useLayoutEffect, useRef, type ReactNode, type RefObject } from "react";
import { createPortal } from "react-dom";
import "./Modal.css";

interface ModalProps {
  open: boolean;
  /** Called on Esc or a backdrop click (when dismissible). */
  onClose: () => void;
  title: string;
  children?: ReactNode;
  /** Action buttons, right-aligned. Put the main action last. */
  footer?: ReactNode;
  /** When false, Esc and backdrop clicks do nothing. */
  dismissible?: boolean;
  /** Element to focus on open. Defaults to the first focusable element. */
  initialFocus?: RefObject<HTMLElement | null>;
}

const FOCUSABLE =
  'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

/*
 * Open modals, newest last. Only the top one reacts to keys, and everything
 * else (the app and lower modals) is made inert so focus can't leave it.
 */
// Layers are stable per-modal tokens rather than DOM nodes: a modal that
// mounts already open can run its effects while refs are still detached
// (React StrictMode re-runs mount effects), and the stack must not miss it.
interface Layer {
  element: HTMLElement | null;
}

const stack: Layer[] = [];

function syncInert() {
  const top = stack[stack.length - 1];
  const root = document.getElementById("root");
  if (root) root.inert = Boolean(top);
  stack.forEach((layer) => {
    if (layer.element) layer.element.inert = layer !== top;
  });
}

function focusableIn(container: HTMLElement): HTMLElement[] {
  return Array.from(container.querySelectorAll<HTMLElement>(FOCUSABLE));
}

export function Modal({ open, onClose, title, children, footer, dismissible = true, initialFocus }: ModalProps) {
  const titleId = useId();
  const layer = useRef<Layer>({ element: null }).current;
  const dialogRef = useRef<HTMLDivElement>(null);
  const onCloseRef = useRef(onClose);
  onCloseRef.current = onClose;

  const attachLayer = useCallback(
    (element: HTMLDivElement | null) => {
      layer.element = element;
      if (element) syncInert();
    },
    [layer],
  );

  // Register as the top layer, move focus in, and restore it on close.
  useLayoutEffect(() => {
    if (!open) return;

    const opener = document.activeElement as HTMLElement | null;
    stack.push(layer);
    syncInert();

    const focusIn = () => {
      const dialog = dialogRef.current;
      if (!dialog || dialog.contains(document.activeElement)) return;
      (initialFocus?.current ?? focusableIn(dialog)[0] ?? dialog).focus();
    };
    focusIn();
    const frame = requestAnimationFrame(focusIn);

    return () => {
      cancelAnimationFrame(frame);
      stack.splice(stack.indexOf(layer), 1);
      syncInert();
      opener?.focus?.();
    };
  }, [open, layer]);

  useEffect(() => {
    if (!open) return;
    const handleKeyDown = (event: KeyboardEvent) => {
      const dialog = dialogRef.current;
      if (!dialog || stack[stack.length - 1] !== layer) return;

      if (event.key === "Escape") {
        event.preventDefault();
        if (dismissible) onCloseRef.current();
        return;
      }

      if (event.key === "Tab") {
        const items = focusableIn(dialog);
        if (items.length === 0) {
          event.preventDefault();
          return;
        }
        const first = items[0];
        const last = items[items.length - 1];
        const active = document.activeElement;
        if (event.shiftKey && (active === first || !dialog.contains(active))) {
          event.preventDefault();
          last.focus();
        } else if (!event.shiftKey && (active === last || !dialog.contains(active))) {
          event.preventDefault();
          first.focus();
        }
      }
    };
    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [open, dismissible, layer]);

  if (!open) return null;

  return createPortal(
    <div
      ref={attachLayer}
      className="modal-backdrop"
      onPointerDown={(event) => {
        if (dismissible && event.target === event.currentTarget) onClose();
      }}
    >
      <div ref={dialogRef} className="modal" role="dialog" aria-modal="true" aria-labelledby={titleId} tabIndex={-1}>
        <h2 id={titleId} className="modal__title">
          {title}
        </h2>
        {children && <div className="modal__body">{children}</div>}
        {footer && <div className="modal__footer">{footer}</div>}
      </div>
    </div>,
    document.body,
  );
}
