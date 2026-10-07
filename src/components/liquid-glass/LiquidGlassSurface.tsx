import {
  useCallback,
  createElement,
  useEffect,
  useId,
  useRef,
  useState,
  type ComponentPropsWithoutRef,
  type ElementType,
  type Ref,
  type ReactNode,
} from "react";
import { RoundedRectLensMap } from "./lensMap";
import "./LiquidGlass.css";

export type GlassPreset = "clear" | "regular" | "dense" | "interactive" | "floating" | "sidebar";

type LiquidGlassSurfaceProps<T extends ElementType> = {
  as?: T;
  preset?: GlassPreset;
  refraction?: boolean;
  children?: ReactNode;
  className?: string;
  ref?: Ref<HTMLElementFor<T>>;
} & Omit<ComponentPropsWithoutRef<T>, "as" | "children" | "className">;

type HTMLElementFor<T extends ElementType> = T extends keyof HTMLElementTagNameMap
  ? HTMLElementTagNameMap[T]
  : HTMLElement;

const RED_ONLY = "1 0 0 0 0  0 0 0 0 0  0 0 0 0 0  0 0 0 1 0";
const GREEN_ONLY = "0 0 0 0 0  0 1 0 0 0  0 0 0 0 0  0 0 0 1 0";
const BLUE_ONLY = "0 0 0 0 0  0 0 0 0 0  0 0 1 0 0  0 0 0 1 0";
const DISPERSION = [1, 0.97, 0.94] as const;

let svgBackdropFilterSupport: boolean | undefined;

/**
 * Reusable web implementation of a liquid-glass surface. Native Tauri window
 * effects still provide the desktop wallpaper material; this SVG filter bends
 * the rendered WebView backdrop where the renderer supports SVG backdrop URLs.
 */
export function LiquidGlassSurface<T extends ElementType = "div">({
  as,
  preset = "regular",
  refraction = true,
  className,
  children,
  ref: forwardedRef,
  ...props
}: LiquidGlassSurfaceProps<T>) {
  const tag = as ?? "div";
  const surfaceRef = useRef<HTMLElement | null>(null);
  const imageRef = useRef<SVGFEImageElement | null>(null);
  const filterRef = useRef<SVGFilterElement | null>(null);
  const generationRef = useRef(0);
  const activeUrlRef = useRef<string | null>(null);
  const [objectUrls] = useState(() => new Set<string>());
  const [revokeTimers] = useState(() => new Set<number>());
  const filterId = `liquid-glass-${useId().replace(/:/g, "")}`;
  const attachSurface = useCallback(
    (node: HTMLElementFor<T> | null) => {
      surfaceRef.current = node;
      if (typeof forwardedRef === "function") forwardedRef(node);
      else if (forwardedRef) {
        (forwardedRef as { current: HTMLElementFor<T> | null }).current = node;
      }
    },
    [forwardedRef],
  );

  useEffect(() => {
    const surface = surfaceRef.current;
    if (!surface) return;

    const reducedTransparency = window.matchMedia("(prefers-reduced-transparency: reduce)");
    const supportsRefraction = supportsSvgBackdropDisplacement();
    if (!refraction) {
      surface.dataset.glassRefraction = "static";
      return;
    }
    if (!supportsRefraction) {
      surface.dataset.glassRefraction = "fallback";
      return;
    }

    const map = new RoundedRectLensMap();
    let settleTimer = 0;
    let disposed = false;

    const updateFilterBounds = (width: number, height: number, radius: number) => {
      const bevel = Math.max(1, radius * 0.5);
      const shift = bevel * 0.6;
      const filter = filterRef.current;
      if (filter) {
        // Expand only by the maximum displaced edge, keeping SVG filter bounds tight.
        filter.setAttribute("x", String(-shift / width));
        filter.setAttribute("y", String(-shift / height));
        filter.setAttribute("width", String(1 + (shift * 2) / width));
        filter.setAttribute("height", String(1 + (shift * 2) / height));
      }
      imageRef.current?.setAttribute("x", "0");
      imageRef.current?.setAttribute("y", "0");
      imageRef.current?.setAttribute("width", String(width));
      imageRef.current?.setAttribute("height", String(height));
      filter?.querySelectorAll<SVGElement>("feDisplacementMap").forEach((node, index) => {
        node.setAttribute("scale", String(shift * 2 * DISPERSION[index % DISPERSION.length]));
      });
    };

    const generate = () => {
      if (disposed) return;
      if (reducedTransparency.matches) {
        surface.dataset.glassRefraction = "fallback";
        surface.dataset.glassMoving = "false";
        return;
      }
      const bounds = surface.getBoundingClientRect();
      if (bounds.width < 2 || bounds.height < 2) {
        surface.dataset.glassRefraction = "fallback";
        surface.dataset.glassMoving = "false";
        return;
      }

      const radius = Number.parseFloat(getComputedStyle(surface).borderTopLeftRadius) || 0;
      updateFilterBounds(bounds.width, bounds.height, radius);
      const generation = ++generationRef.current;
      surface.dataset.glassRefraction = "pending";

      try {
        map.generate({ width: bounds.width, height: bounds.height, radius }, (blob) => {
          if (disposed || generation !== generationRef.current) return;
          const nextUrl = URL.createObjectURL(blob);
          objectUrls.add(nextUrl);
          const probe = new Image();
          probe.onload = () => {
            if (disposed || generation !== generationRef.current) {
              URL.revokeObjectURL(nextUrl);
              objectUrls.delete(nextUrl);
              return;
            }

            const previousUrl = activeUrlRef.current;
            activeUrlRef.current = nextUrl;
            imageRef.current?.setAttribute("href", nextUrl);
            imageRef.current?.setAttributeNS("http://www.w3.org/1999/xlink", "xlink:href", nextUrl);
            surface.dataset.glassRefraction = "ready";
            surface.dataset.glassMoving = "false";

            if (previousUrl) {
              const timer = window.setTimeout(() => {
                URL.revokeObjectURL(previousUrl);
                objectUrls.delete(previousUrl);
                revokeTimers.delete(timer);
              }, 180);
              revokeTimers.add(timer);
            }
          };
          probe.onerror = () => {
            URL.revokeObjectURL(nextUrl);
            objectUrls.delete(nextUrl);
            if (!disposed && generation === generationRef.current) {
              surface.dataset.glassRefraction = "fallback";
              surface.dataset.glassMoving = "false";
            }
          };
          probe.src = nextUrl;
        });
      } catch {
        surface.dataset.glassRefraction = "fallback";
        surface.dataset.glassMoving = "false";
      }
    };

    const scheduleGeometry = () => {
      generationRef.current += 1;
      map.cancel();
      surface.dataset.glassMoving = "true";
      if (settleTimer) window.clearTimeout(settleTimer);
      settleTimer = window.setTimeout(generate, 125);
    };

    const observer = new ResizeObserver(scheduleGeometry);
    observer.observe(surface);
    const transitionStart = (event: TransitionEvent) => {
      if (["width", "height", "border-radius"].includes(event.propertyName)) scheduleGeometry();
    };
    surface.addEventListener("transitionrun", transitionStart);
    if (reducedTransparency.matches) {
      surface.dataset.glassRefraction = "fallback";
      surface.dataset.glassMoving = "false";
    } else {
      scheduleGeometry();
    }

    const handleTransparencyChange = () => {
      if (reducedTransparency.matches) {
        generationRef.current += 1;
        map.cancel();
        surface.dataset.glassRefraction = "fallback";
        surface.dataset.glassMoving = "false";
        const activeUrl = activeUrlRef.current;
        if (activeUrl) {
          URL.revokeObjectURL(activeUrl);
          objectUrls.delete(activeUrl);
          activeUrlRef.current = null;
          imageRef.current?.removeAttribute("href");
          imageRef.current?.removeAttributeNS("http://www.w3.org/1999/xlink", "href");
        }
      } else {
        scheduleGeometry();
      }
    };
    reducedTransparency.addEventListener("change", handleTransparencyChange);

    return () => {
      disposed = true;
      observer.disconnect();
      surface.removeEventListener("transitionrun", transitionStart);
      reducedTransparency.removeEventListener("change", handleTransparencyChange);
      window.clearTimeout(settleTimer);
      for (const timer of revokeTimers) window.clearTimeout(timer);
      revokeTimers.clear();
      map.cancel();
      for (const url of objectUrls) URL.revokeObjectURL(url);
      objectUrls.clear();
      activeUrlRef.current = null;
    };
  }, [refraction, objectUrls, revokeTimers]);

  const style = {
    ...props.style,
    "--glass-filter": `url("#${filterId}")`,
  } as ComponentPropsWithoutRef<T>["style"] & { [key: `--${string}`]: string };

  const surface = createElement(
    tag,
    {
      ...props,
      ref: attachSurface,
      className: ["liquid-glass", `liquid-glass--${preset}`, className].filter(Boolean).join(" "),
      style,
      "data-glass-preset": preset,
    } as never,
    children,
  );

  return (
    <>
      {refraction && (
        <svg className="liquid-glass__definitions" aria-hidden="true" focusable="false">
          <defs>
            <filter
              ref={filterRef}
              id={filterId}
              x="-0.02"
              y="-0.02"
              width="1.04"
              height="1.04"
              filterUnits="objectBoundingBox"
              primitiveUnits="userSpaceOnUse"
              colorInterpolationFilters="sRGB"
            >
              <feImage
                ref={imageRef}
                x="0"
                y="0"
                width="100%"
                height="100%"
                preserveAspectRatio="none"
                href="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='1' height='1'%3E%3Crect width='1' height='1' fill='%23808080'/%3E%3C/svg%3E"
                result="lens-map"
              />
              <feDisplacementMap
                in="SourceGraphic"
                in2="lens-map"
                scale="0"
                xChannelSelector="R"
                yChannelSelector="B"
                result="red-shift"
              />
              <feDisplacementMap
                in="SourceGraphic"
                in2="lens-map"
                scale="0"
                xChannelSelector="R"
                yChannelSelector="B"
                result="green-shift"
              />
              <feDisplacementMap
                in="SourceGraphic"
                in2="lens-map"
                scale="0"
                xChannelSelector="R"
                yChannelSelector="B"
                result="blue-shift"
              />
              <feColorMatrix in="red-shift" type="matrix" values={RED_ONLY} result="red-channel" />
              <feColorMatrix in="green-shift" type="matrix" values={GREEN_ONLY} result="green-channel" />
              <feColorMatrix in="blue-shift" type="matrix" values={BLUE_ONLY} result="blue-channel" />
              <feBlend in="red-channel" in2="green-channel" mode="screen" result="red-green" />
              <feBlend in="red-green" in2="blue-channel" mode="screen" />
            </filter>
          </defs>
        </svg>
      )}
      {surface}
    </>
  );
}

function supportsSvgBackdropDisplacement() {
  if (svgBackdropFilterSupport !== undefined) return svgBackdropFilterSupport;
  if (!CSS.supports("backdrop-filter", 'url("#liquid-glass-capability")')) {
    svgBackdropFilterSupport = false;
    return false;
  }

  const namespace = "http://www.w3.org/2000/svg";
  const svg = document.createElementNS(namespace, "svg");
  const filter = document.createElementNS(namespace, "filter");
  const displacement = document.createElementNS(namespace, "feDisplacementMap");
  const image = document.createElementNS(namespace, "feImage");
  const colorMatrix = document.createElementNS(namespace, "feColorMatrix");
  const blend = document.createElementNS(namespace, "feBlend");
  filter.append(displacement, image, colorMatrix, blend);
  svg.append(filter);
  svgBackdropFilterSupport =
    typeof SVGFEImageElement !== "undefined" &&
    typeof SVGFEDisplacementMapElement !== "undefined" &&
    typeof SVGFEColorMatrixElement !== "undefined" &&
    typeof SVGFEBlendElement !== "undefined" &&
    typeof displacement.setAttribute === "function" &&
    typeof image.setAttribute === "function";
  return svgBackdropFilterSupport;
}
