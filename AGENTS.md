# Agent Instructions

Guidance for AI coding tools (Claude Code, Codex, Cursor, Copilot, and others) working on Yap.

## Project

Yap is an open-source, AI-powered voice dictation app built with Tauri 2 (Rust) and React + TypeScript. See [README.md](README.md) for setup and the project structure.

## Design

**Read [DESIGN.md](DESIGN.md) before making any UI change, and follow it.** It defines Yap's colors, typography, spacing, icons, components, and copy style. Don't invent new styles: reuse the tokens in `src/styles/global.css` and the components in `src/components/`. If something isn't covered, extend the design system and update DESIGN.md in the same change.

## Conventions

- Frontend code lives in `src/`. Shared components go in `src/components/`, section pages in `src/features/<section>/`, hooks in `src/hooks/`, and helpers in `src/lib/`.
- Import from `src/` with the `@/` alias.
- Import Devigner icons per icon (`@devigner-ui/icons/<Name>`), never from the package root.
- Rust code lives in `src-tauri/`. Commands exposed to the frontend go in `src-tauri/src/commands/`.
- Follow the commit style and PR checklist in [CONTRIBUTING.md](CONTRIBUTING.md).

## Checks

Run these before finishing a change:

```bash
npm run build
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```
