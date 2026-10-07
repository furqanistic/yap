# Yap

[![CI](https://github.com/furqanistic/yap/actions/workflows/ci.yml/badge.svg)](https://github.com/furqanistic/yap/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-bced09.svg)](LICENSE)

AI-powered voice dictation for the desktop, built with [Tauri 2](https://tauri.app) and React + TypeScript.

> **Status:** early development. Expect rapid changes.

## Prerequisites

- [Node.js](https://nodejs.org) 20+
- [Rust](https://www.rust-lang.org/tools/install), latest stable (`src-tauri/rust-toolchain.toml` selects it for you)
- [CMake](https://cmake.org/download/) 3.20+, a C++ compiler, and libclang, used to build the bundled speech engine (whisper.cpp) and generate its Rust bindings
- The [Tauri system dependencies](https://tauri.app/start/prerequisites/) for your OS

Per-OS notes:

- **Windows:** [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the "Desktop development with C++" workload (MSVC and the Windows SDK), CMake, and [LLVM](https://github.com/llvm/llvm-project/releases) for libclang (`winget install LLVM.LLVM`). If CMake or libclang aren't on your `PATH`, set the `CMAKE` and `LIBCLANG_PATH` environment variables (for example in your Cargo `config.toml` under `[env]`). **Also set** `CMAKE_C_FLAGS_RELEASE = "/MD /O2 /Ob2 /DNDEBUG"` and `CMAKE_CXX_FLAGS_RELEASE = "/MD /O2 /Ob2 /DNDEBUG /EHsc /utf-8"` there: without them the speech engine builds unoptimized and transcribes about 30 times slower. WebView2 is preinstalled on Windows 10 and 11.
- **macOS:** Xcode Command Line Tools (`xcode-select --install`, includes libclang) and CMake (`brew install cmake`).
- **Linux (Debian/Ubuntu):**

  ```bash
  sudo apt install build-essential cmake clang libclang-dev libwebkit2gtk-4.1-dev \
    libappindicator3-dev librsvg2-dev patchelf libssl-dev libxdo-dev libasound2-dev
  ```

  `libasound2-dev` is for microphone capture and `libxdo-dev` for typing text into other apps.

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

## How hold-to-talk works

Yap detects the push-to-talk shortcut, including modifier-only chords such as Ctrl+Win, with a low-level keyboard hook (`WH_KEYBOARD_LL` on Windows). The hook only watches for the chosen shortcut and never records or sends keystrokes anywhere. Some antivirus tools flag low-level keyboard hooks in general; if yours does, allow Yap.

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
│   │   ├── general/        General settings page
│   │   └── model/          Model page and the download modal
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
