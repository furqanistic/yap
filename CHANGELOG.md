# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Dropdown options were unreadable in dark mode on Windows; dropdowns now use a custom menu
- Removed the lime focus outline from fields; keyboard focus is now neutral
- Pages now scroll when they are taller than the window, and the Advanced and About sidebar items stay visible at the default and minimum window sizes

### Added

- Initial Tauri 2 + React + TypeScript project setup
- macOS-style sidebar with General, Onboarding, Model, Language, Dictionary, History, Advanced, and About sections
- Native window translucency (Acrylic on Windows, vibrancy on macOS) with a custom title bar
- Devigner Icons for sidebar navigation
- DESIGN.md design system guide, plus AGENTS.md and CLAUDE.md so AI coding tools follow it
- General settings page: push-to-talk shortcut, microphone, recording indicator, auto-paste, language mode, model, launch at login, and start minimized
