import { IconCheck } from "@devigner-ui/icons/Check";
import { IconChevronDown } from "@devigner-ui/icons/ChevronDown";
import {
  useEffect,
  useId,
  useLayoutEffect,
  useRef,
  useState,
  type CSSProperties,
  type KeyboardEvent,
  type RefObject,
} from "react";
import { createPortal } from "react-dom";
import { LiquidGlassSurface } from "@/components/liquid-glass";
import "./Field.css";
import "./Menu.css";

export interface SelectOption<T extends string> {
  value: T;
  label: string;
}

interface SelectProps<T extends string> {
  value: T;
  options: readonly SelectOption<T>[];
  onChange: (value: T) => void;
  disabled?: boolean;
  "aria-labelledby"?: string;
  "aria-describedby"?: string;
}

const MENU_GAP = 6;
const VIEWPORT_MARGIN = 12;
const OPTION_HEIGHT = 32;
const MENU_PADDING = 10;

/**
 * Places the menu under the trigger, or above it when there isn't room below.
 * Uses fixed positioning in a portal so scrolling containers can't clip it.
 */
function useMenuPosition(
  triggerRef: RefObject<HTMLElement | null>,
  menuRef: RefObject<HTMLElement | null>,
  open: boolean,
  optionCount: number,
): CSSProperties {
  const [style, setStyle] = useState<CSSProperties>({});

  useLayoutEffect(() => {
    if (!open) return;

    const update = (writeToElement = false) => {
      const trigger = triggerRef.current;
      if (!trigger) return;
      const rect = trigger.getBoundingClientRect();
      const menuHeight = optionCount * OPTION_HEIGHT + MENU_PADDING;
      const spaceBelow = window.innerHeight - rect.bottom - VIEWPORT_MARGIN;
      const spaceAbove = rect.top - VIEWPORT_MARGIN;
      const placeAbove = spaceBelow < menuHeight && spaceAbove > spaceBelow;
      const maxHeight = Math.max(placeAbove ? spaceAbove : spaceBelow, OPTION_HEIGHT * 3);

      if (writeToElement) {
        const menu = menuRef.current;
        if (!menu) return;
        menu.style.left = `${rect.left}px`;
        menu.style.width = `${rect.width}px`;
        menu.style.maxHeight = `${maxHeight}px`;
        menu.style.top = placeAbove ? "" : `${rect.bottom + MENU_GAP}px`;
        menu.style.bottom = placeAbove ? `${window.innerHeight - rect.top + MENU_GAP}px` : "";
        return;
      }

      setStyle({
        left: rect.left,
        width: rect.width,
        maxHeight,
        ...(placeAbove
          ? { bottom: window.innerHeight - rect.top + MENU_GAP }
          : { top: rect.bottom + MENU_GAP }),
      });
    };

    let frame = 0;
    const scheduleUpdate = () => {
      if (frame) return;
      frame = window.requestAnimationFrame(() => {
        frame = 0;
        update(true);
      });
    };

    update();
    window.addEventListener("resize", scheduleUpdate);
    window.addEventListener("scroll", scheduleUpdate, { capture: true, passive: true });
    return () => {
      window.cancelAnimationFrame(frame);
      window.removeEventListener("resize", scheduleUpdate);
      window.removeEventListener("scroll", scheduleUpdate, true);
    };
  }, [open, optionCount, triggerRef, menuRef]);

  return style;
}

/** Dropdown with a custom menu, so it looks the same on every OS and theme. */
export function Select<T extends string>({
  value,
  options,
  onChange,
  disabled,
  "aria-labelledby": labelledBy,
  "aria-describedby": describedBy,
}: SelectProps<T>) {
  const [open, setOpen] = useState(false);
  const [activeIndex, setActiveIndex] = useState(0);
  const triggerRef = useRef<HTMLButtonElement>(null);
  const listRef = useRef<HTMLUListElement>(null);
  const triggerId = useId();
  const listId = useId();
  const menuStyle = useMenuPosition(triggerRef, listRef, open, options.length);

  const selectedIndex = options.findIndex((option) => option.value === value);
  const selected = options[selectedIndex];
  const optionId = (index: number) => `${listId}-option-${index}`;

  const openMenu = () => {
    setActiveIndex(Math.max(selectedIndex, 0));
    setOpen(true);
  };

  const closeMenu = (restoreFocus = true) => {
    setOpen(false);
    if (restoreFocus) triggerRef.current?.focus();
  };

  const choose = (index: number) => {
    onChange(options[index].value);
    closeMenu();
  };

  // Move focus into the menu when it opens
  useEffect(() => {
    if (open) listRef.current?.focus();
  }, [open]);

  // Keep the highlighted option visible while navigating with the keyboard
  useEffect(() => {
    if (open) document.getElementById(optionId(activeIndex))?.scrollIntoView({ block: "nearest" });
  }, [open, activeIndex]);

  // Close when clicking anywhere outside the trigger and menu
  useEffect(() => {
    if (!open) return;
    const handlePointerDown = (event: PointerEvent) => {
      const target = event.target as Node;
      if (!triggerRef.current?.contains(target) && !listRef.current?.contains(target)) {
        closeMenu(false);
      }
    };
    document.addEventListener("pointerdown", handlePointerDown);
    return () => document.removeEventListener("pointerdown", handlePointerDown);
  }, [open]);

  const handleTriggerKeyDown = (event: KeyboardEvent<HTMLButtonElement>) => {
    if (["ArrowDown", "ArrowUp", "Enter", " "].includes(event.key)) {
      event.preventDefault();
      openMenu();
    }
  };

  const handleListKeyDown = (event: KeyboardEvent<HTMLUListElement>) => {
    const last = options.length - 1;
    switch (event.key) {
      case "ArrowDown":
        setActiveIndex((index) => Math.min(index + 1, last));
        break;
      case "ArrowUp":
        setActiveIndex((index) => Math.max(index - 1, 0));
        break;
      case "Home":
        setActiveIndex(0);
        break;
      case "End":
        setActiveIndex(last);
        break;
      case "Enter":
      case " ":
        choose(activeIndex);
        break;
      case "Escape":
        closeMenu();
        break;
      case "Tab":
        closeMenu(false);
        return;
      default:
        return;
    }
    event.preventDefault();
  };

  return (
    <>
      <LiquidGlassSurface
        as="button"
        ref={triggerRef}
        id={triggerId}
        type="button"
        preset="dense"
        className={`field field--select${open ? " field--open" : ""}`}
        disabled={disabled}
        aria-haspopup="listbox"
        aria-expanded={open}
        aria-controls={open ? listId : undefined}
        aria-labelledby={labelledBy ? `${labelledBy} ${triggerId}` : undefined}
        aria-describedby={describedBy}
        onClick={() => (open ? closeMenu() : openMenu())}
        onKeyDown={handleTriggerKeyDown}
      >
        <span className="field__value">{selected?.label}</span>
        <IconChevronDown className="field__icon field__chevron" strokeWidth={2} />
      </LiquidGlassSurface>

      {open &&
        createPortal(
          <LiquidGlassSurface
            as="ul"
            ref={listRef}
            id={listId}
            className="menu"
            role="listbox"
            tabIndex={-1}
            preset="floating"
            aria-labelledby={labelledBy}
            aria-activedescendant={optionId(activeIndex)}
            style={menuStyle}
            onKeyDown={handleListKeyDown}
          >
            {options.map((option, index) => (
              <li
                key={option.value}
                id={optionId(index)}
                role="option"
                aria-selected={index === selectedIndex}
                className={`menu__option${index === activeIndex ? " menu__option--active" : ""}`}
                onPointerMove={() => setActiveIndex(index)}
                onClick={() => choose(index)}
              >
                <span className="menu__label">{option.label}</span>
                {index === selectedIndex && <IconCheck className="menu__check" strokeWidth={2} />}
              </li>
            ))}
          </LiquidGlassSurface>,
          document.body,
        )}
    </>
  );
}
