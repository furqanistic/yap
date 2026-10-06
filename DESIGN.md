# Yap Design System

This is the single source of truth for how Yap looks and feels. Every UI change, whether written by a person or an AI tool, must follow it. If something you need isn't covered here, extend the system (and this file) rather than inventing a one-off style.

## Principles

1. **Flat and soft.** No gradients, gloss, bevels, or heavy shadows. Depth comes from soft tints, hairline borders, and rounded corners.
2. **Minimal.** Every element earns its place. Prefer whitespace over dividers, and fewer controls over more.
3. **Native-feeling.** Yap should feel like a well-made macOS app on every platform: system font, translucent glass window, arrow cursor, quiet motion.
4. **One accent.** Lime (`#bced09`) is the only brand color. Use it sparingly to show what's selected or on, and only as a fill or icon color. Never as an outline, border, or focus ring.
5. **Readable on any wallpaper.** The window is see-through, so every surface must stay legible over bright and busy backgrounds, in light and dark mode.

## Color

All colors live as CSS custom properties in [`src/styles/global.css`](src/styles/global.css). **Never hard-code a color in a component.** If you need a new one, add a token there with light, dark, and glass values.

### Brand

| Token | Light | Dark | Use |
|---|---|---|---|
| `--color-primary` | `#bced09` | `#bced09` | Fills only: switch "on", logo tile |
| `--color-on-primary` | `#111113` | `#111113` | Text/icons placed on a lime fill |
| `--color-primary-soft` | lime at 24% | lime at 14% | Selected sidebar item background |
| `--color-primary-strong` | `#5a7300` | `#bced09` | Lime-colored icons (e.g. the selected sidebar icon) |

> **Rules:**
> - Never put `--color-primary` text or icons on a light background. It's unreadable. Use `--color-primary-strong`, which becomes olive in light mode and lime in dark mode.
> - **No lime outlines, borders, or focus rings.** Focus is always neutral (see [Interaction](#interaction-and-accessibility)).

### Neutrals

| Token | Use |
|---|---|
| `--color-window` | Window background (transparent in glass mode) |
| `--color-surface` | Content pane background |
| `--color-card` | Settings cards, empty states |
| `--color-text` | Primary text, titles |
| `--color-text-secondary` | Unselected sidebar labels |
| `--color-text-muted` | Descriptions, hints, group titles, inactive icons |
| `--color-separator` | Hairline borders and dividers |
| `--color-hover` | Hover background for rows and buttons |
| `--color-control-off` | Switch track when off |
| `--color-field` / `--color-field-border` / `--color-field-border-hover` | Inputs and selects |
| `--color-key` | Keycap chips in the shortcut recorder |
| `--color-switch-thumb` | Switch knob |
| `--color-focus` / `--color-focus-ring` | Neutral keyboard-focus outline and field ring |
| `--color-menu` / `--color-menu-border` / `--shadow-menu` | Dropdown menus (always opaque, even in glass mode) |
| `--color-window-close` / `-pressed` / `--color-on-window-close` | Windows close button hover (red by platform convention) |
| `--color-primary-hover` | Primary button hover (a slightly deeper lime) |
| `--color-danger` / `-hover` / `--color-on-danger` | Destructive button fill and its text |
| `--color-danger-text` | Error messages and other red text (lighter in dark mode) |
| `--color-tag` | Neutral `Tag` background (translucent, works on glass cards) |
| `--color-backdrop` | Dimmed layer behind a `Modal` |
| `--color-toast` / `--color-on-toast` | Toast background and text (opaque in every mode) |

Floating surfaces (menus, modals, toasts) are opaque in every mode, glass included, so their tokens have no glass override.

## Glass (window translucency)

Inside the Tauri window, the OS blurs the desktop behind Yap (Acrylic on Windows, vibrancy on macOS). [`src/lib/platform.ts`](src/lib/platform.ts) then sets `data-vibrancy="native"` on `<html>`, and `global.css` swaps opaque tokens for translucent tints.

- **Sidebar** is tinted slightly grayer than the **content pane** so the two read as separate panes.
- **Cards and fields** carry a stronger tint than the pane behind them so text stays readable.
- **Don't add `backdrop-filter`** in glass mode. The OS already blurs, and stacking blurs looks muddy and costs performance.
- **Every token overridden in `:root[data-vibrancy="native"]` must also be overridden in its dark-mode block.** Otherwise the light value leaks into dark mode. This exact bug once made dark-mode fields unreadable.
- In a regular browser (no glass), the same tokens fall back to opaque colors and a soft decorative glow behind the sidebar.

## Typography

Use the system font stack defined on `:root` (SF Pro on macOS, Segoe UI on Windows). Don't load web fonts.

| Role | Size | Weight | Color |
|---|---|---|---|
| Page title | 20px | 650 | text, letter-spacing `-0.015em` |
| App name (sidebar) | 15px | 600 | text |
| Sidebar item | 13.5px | 500 (600 when selected) | text-secondary (text when selected) |
| Setting title | 13.5px | 500 | text |
| Body / control text | 13px | 400 | text |
| Page description | 13px | 400 | text-muted |
| Setting description | 12.5px | 400 | text-muted |
| Group title | 12px | 600 | text-muted |
| Keycap | 12px | 500 | text |
| Hint under a control | 11.5px | 400 | text-muted |

## Layout and spacing

| Element | Value |
|---|---|
| Window default / minimum | 960×640 / 760×520 |
| Sidebar width | 220px, padding `0 12px 12px` |
| Title bar height | `--titlebar-height` (36px) |
| Page | max-width 680px, padding `4px 32px 32px` |
| Gap between settings groups | 24px |
| Setting row | min-height 60px, padding `12px 16px`, 24px gap between text and control |
| Sidebar item | height 34px, padding `0 10px`, 10px icon gap, 2px between items |
| Input / select | 240px × 34px |
| Switch | 38px × 22px |

Spacing should come from this scale: **2, 4, 6, 8, 10, 12, 16, 20, 24, 32**.

## Corner radius

| Radius | Use |
|---|---|
| 6px | Keycaps |
| 7px | Logo tile |
| 8px | Menu options |
| 10px | Inputs, selects, buttons, sidebar items |
| 12px | Dropdown menus, toasts |
| 14px | Cards, empty states, modals |
| 999px | Switch track, tags, progress bar (pills) |

## Borders, shadows, and depth

- Separate things with **1px `--color-separator` hairlines**, or just whitespace.
- **No drop shadows** on cards, rows, or buttons. Only the switch thumb and floating surfaces get a shadow. Floating surfaces (menus, modals, toasts) use `--shadow-menu` so they read as sitting above the page.
- **No gradients or inner highlights.** The sidebar used to have glossy icon tiles and gradient pills, and they read as dated. Don't bring them back. The one exception is the soft radial glow behind the sidebar in browser mode (`--glow-*`), which stands in for the desktop wallpaper and disappears in the real glass window.

## Icons

We use [Devigner Icons](https://github.com/devigner-ui/icons) (`@devigner-ui/icons`). Credits are in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

- **Import each icon from its own path:** `import { IconRocket } from "@devigner-ui/icons/Rocket";`. Never import from the package root. Our bundler doesn't tree-shake it, and it adds ~6 MB.
- **Outline variant only** (the default). Don't mix in Bold, Bulk, or TwoTone.
- **Size with a CSS class**, not props. The components have no `size` prop.
- **Color comes from `currentColor`.** Set `color` on the icon or its parent.
- Stroke width: **1.75** for navigation and inline icons, **1.5** for large decorative icons, **2** for small chevrons.

| Context | Size |
|---|---|
| Sidebar item | 18px |
| Icon inside a field or button (chevron, keyboard, download) | 16px |
| Logo glyph | 15px |
| Empty-state illustration | 26px |

Icons next to a visible label are decorative. The library hides them from screen readers by default, so don't add labels to them.

## Motion

- Color and background changes: **150ms `ease`**.
- Switch thumb: **180ms `cubic-bezier(0.3, 0.7, 0.4, 1)`**.
- Keep transitions under 200ms. No bounce, no scale-up effects, no page-transition animations.
- Any new motion beyond color fades must be disabled under `@media (prefers-reduced-motion: reduce)`.

## Interaction and accessibility

- **Cursor:** `default` (arrow) on all controls, like a native app. Don't use `pointer`.
- **Text selection:** off for UI chrome (`user-select: none` on `body`). Turn it back on for content people may copy, such as transcripts.
- **Focus:** keyboard focus must be visible but **neutral, never lime**. Buttons, switches, and nav items get a 2px `--color-focus` outline. Fields get a `--color-field-border-hover` border plus a 3px `--color-focus-ring` ring. Use `:focus-visible`, so nothing shows after a mouse click.
- **Hover:** `--color-hover` background. Selected items keep their selected style on hover.
- **Dropdowns:** use the `Select` component, never a native `<select>`. Native popups are drawn by the OS, ignore our colors, and were unreadable in dark mode on Windows.
- **Semantics:** use real `<button>` elements, `role="switch"` + `aria-checked`, and `aria-current="page"` for the active nav item. Every control must be labelled. `SettingRow` passes `aria-labelledby` / `aria-describedby` to its control for you.
- **Contrast:** text must stay readable over any wallpaper. Check glass mode in both themes.

## Components

Reuse these before building anything new. They live in [`src/components/`](src/components).

| Component | Path | Use for |
|---|---|---|
| `AppShell` | `layout/` | Window layout: sidebar + title bar + scrolling content |
| `TitleBar`, `WindowControls` | `titlebar/` | Draggable top strip; Windows/Linux min/max/close buttons |
| `Sidebar` | `sidebar/` | Navigation. Add sections in [`src/app/navigation.ts`](src/app/navigation.ts) |
| `Page` | `page/` | Every section page: title, description, body |
| `SettingsGroup` | `settings/` | A titled card grouping related settings |
| `SettingRow` | `settings/` | One setting: title + description on the left, control on the right, optional hint |
| `Switch` | `ui/` | On/off settings |
| `Select` | `ui/` | Choosing one option from a list (custom menu with full keyboard support). Supports a `placeholder` and `disabled` options for things that aren't ready yet |
| `ShortcutInput` | `ui/` | Recording a keyboard shortcut |
| `Button` | `ui/` | Any action. See [Buttons](#buttons) |
| `TextField` | `ui/` | Free text. `variant="search"` adds a leading magnifier |
| `Tag` | `ui/` | Small pill label ("Multilingual"). `selected` for "In use" or "Recommended" |
| `ProgressBar` | `ui/` | Download or task progress. Omit `value` for an indeterminate bar |
| `Modal` | `ui/` | A focused task or decision on top of the page |
| `ConfirmDialog` | `ui/` | "Are you sure?" before a destructive or lossy action |
| `EmptyState` | `ui/` | A list or section with nothing in it yet |
| `ToastProvider`, `useToast` | `ui/` | A short, passing message ("Text copied") |

### Buttons

| Variant | Look | Use for |
|---|---|---|
| `primary` | Lime fill, `--color-on-primary` text | The one main action in a view or dialog |
| `secondary` (default) | Field style: tinted fill, hairline border | Most actions, and Cancel in dialogs |
| `quiet` | No fill until hover | Low-priority actions in rows ("Delete" next to "Use") |
| `destructive` | Red fill, white text | Confirming something that can't be undone |

- Default size is 34px high, matching inputs. `size="small"` is 28px, for buttons inside setting rows and lists.
- Radius 10px, 13px/500 text, optional 16px leading icon (stroke 1.75).
- Use `loading` while an action runs. It shows a spinner and disables the button.
- At most one `primary` button per view or dialog.

### Modals

- Max width 440px, padding 20px, radius 14px, opaque `--color-menu` surface with `--shadow-menu`. Title 15px/600, body 13px in `--color-text-secondary`, actions right-aligned with the main action last.
- The backdrop is `--color-backdrop` with **no** `backdrop-filter`.
- Focus moves into the dialog, is trapped there, and returns to the opener on close. Esc and backdrop clicks close it unless `dismissible={false}`. Modals can stack (a `ConfirmDialog` over a `Modal`); only the top one responds.
- `ConfirmDialog` with `destructive` focuses Cancel first, so Enter never deletes by accident.

### Feedback

- `ProgressBar`: 6px pill, `--color-control-off` track, lime fill. Always give it an `aria-label` or `aria-labelledby`.
- Toasts appear bottom-center, one at a time, and disappear after 4 seconds. Keep them to a few words. Use them for confirmations, not errors that need action.
- Error text uses `--color-danger-text`.

### Building a settings page

```tsx
<Page title="Model" description="Choose the AI model used for transcription.">
  <SettingsGroup title="Transcription">
    <SettingRow title="Model" description="Select the transcription model to use." hint={hint}>
      {(a11y) => <Select value={model} options={MODEL_OPTIONS} onChange={setModel} {...a11y} />}
    </SettingRow>
  </SettingsGroup>
</Page>
```

- Group related settings into cards of **2 to 4 rows**, each with a short noun title ("Recording", "Startup").
- Put the page in `src/features/<section>/` and register it in `SECTION_PAGES` in [`src/app/App.tsx`](src/app/App.tsx).
- Keep option lists and defaults in an `options.ts` next to the page, not inline in JSX.
- Option labels must fit the 240px field without truncating. Keep them under ~30 characters.
- For a read-only value in place of a control ("1.2 GB"), use `<span className="setting-row__value">`.
- Long descriptions such as folder paths wrap on their own; don't truncate them.
- Content announced to screen readers but not shown goes in an element with the `sr-only` class.

## Writing (UI copy)

- **Sentence case** everywhere: "Launch at login", not "Launch At Login".
- **Titles** are short and have no period. **Descriptions** are one sentence and end with a period.
- Say what the setting does, not how it works: "Automatically start Yap when you log in."
- Mark the suggested option with "(recommended)".
- Use platform-neutral words ("in the background", not "in the menu bar"), unless the text only appears on one platform.

## Do and don't

| Do | Don't |
|---|---|
| Use tokens from `global.css` | Hard-code hex values in components |
| Use flat fills and hairline borders | Add gradients, gloss, or drop shadows |
| Use lime for "selected" and "on" | Use lime for text, outlines, borders, or focus rings |
| Use the `Select` component | Use a native `<select>` |
| Use outline Devigner icons, imported per icon | Mix icon sets or import the package root |
| Reuse `Page`, `SettingsGroup`, `SettingRow`, `ui/` controls | Build one-off cards or controls for a single page |
| Check light, dark, and glass mode | Only check the theme you happen to use |

## Before you ship a UI change

- [ ] Uses existing tokens and components, and any new token has light, dark, and glass values
- [ ] Looks right in **light** and **dark** mode, both **in the app (glass)** and in the browser
- [ ] Nothing overflows or truncates at the **760px** minimum window width
- [ ] Keyboard focus is visible, and every control has a label
- [ ] `npm run build` passes
