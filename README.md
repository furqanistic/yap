# Yap

[![CI](https://github.com/furqanistic/yap/actions/workflows/ci.yml/badge.svg)](https://github.com/furqanistic/yap/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-bced09.svg)](LICENSE)

AI-powered voice dictation for the desktop, built with [Tauri 2](https://tauri.app) and React + TypeScript.

> **Status:** early development. Expect rapid changes.

## Prerequisites

- [Node.js](https://nodejs.org) 20+
- [Rust](https://www.rust-lang.org/tools/install) (stable)
- Tauri system dependencies: https://tauri.app/start/prerequisites/

## Getting started

```bash
git clone https://github.com/furqanistic/yap.git
cd yap
npm install
npm run tauri dev
```

Build a production bundle:

```bash
npm run tauri build
```

## Project structure

```
yap/
├── .github/                Issue/PR templates, CI workflow, Dependabot
├── public/                 Static assets served as-is
├── src/                    React frontend
│   ├── app/                App root and sidebar navigation config
│   ├── assets/             Images, fonts, and other imported assets
│   ├── components/         Shared, reusable UI components
│   │   ├── layout/         App shell (sidebar + content)
│   │   ├── page/           Page title + body wrapper
│   │   ├── section-placeholder/  Temporary "coming soon" page per section
│   │   ├── settings/       Settings groups and rows
│   │   ├── sidebar/        macOS-style navigation sidebar
│   │   ├── titlebar/       Custom title bar and window controls
│   │   └── ui/             Controls, buttons, modals, toasts and other building blocks
│   ├── features/           Feature modules (one folder per feature)
│   │   └── general/        General settings page
│   ├── hooks/              Shared React hooks
│   ├── lib/                Utilities and Tauri API wrappers
│   ├── styles/             Global styles and design tokens
│   ├── types/              Shared TypeScript types
│   └── main.tsx            Frontend entry point
└── src-tauri/              Rust backend
    ├── capabilities/       Tauri permission capabilities
    ├── icons/              App icons
    ├── src/
    │   ├── commands/       Commands exposed to the frontend via `invoke`
    │   ├── lib.rs          App builder and plugin setup
    │   └── main.rs         Binary entry point
    └── tauri.conf.json     Tauri configuration
```

Imports inside `src/` can use the `@/` alias, e.g. `import App from "@/app/App"`.

## Contributing

Contributions are welcome! Please read the [contributing guide](CONTRIBUTING.md) and our [code of conduct](CODE_OF_CONDUCT.md) before opening an issue or pull request. UI changes must follow the [design system](DESIGN.md).

To report a security vulnerability, see [SECURITY.md](SECURITY.md).

## License

Yap is licensed under the [MIT License](LICENSE).

Icons are from [Devigner Icons](https://github.com/devigner-ui/icons) (artwork under CC BY 4.0). See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for full credits.
