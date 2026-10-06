# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Pages taller than the window couldn't scroll, and the bottom of the sidebar could be cut off
- Dropdown options were unreadable in dark mode on Windows; dropdowns now use a custom menu
- Removed the lime focus outline from fields; keyboard focus is now neutral

### Added

- Settings are saved by the Rust backend and survive restarts
- Shared UI components: Button, Modal, ConfirmDialog, ProgressBar, Toast, EmptyState, TextField and Tag
- Microphones are listed and recorded natively, so every device shows up without a permission prompt. "Test microphone" shows a live level meter
- Model page: download, resume, cancel, use and delete Whisper models, with a download window that shows progress, speed and time left. Downloads are checked for damage before use
- Initial Tauri 2 + React + TypeScript project setup
- macOS-style sidebar with General, Onboarding, Model, Language, Dictionary, History, Advanced, and About sections
- Native window translucency (Acrylic on Windows, vibrancy on macOS) with a custom title bar
- Devigner Icons for sidebar navigation
- DESIGN.md design system guide, plus AGENTS.md and CLAUDE.md so AI coding tools follow it
- General settings page: push-to-talk shortcut, microphone, recording indicator, auto-paste, language mode, model, launch at login, and start minimized
