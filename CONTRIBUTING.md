# Contributing to Yap

Thanks for your interest in contributing! This guide explains how to get set up and how to submit changes.

By participating in this project you agree to follow our [Code of Conduct](CODE_OF_CONDUCT.md).

## Ways to contribute

- **Report bugs**: open a [bug report](https://github.com/furqanistic/yap/issues/new?template=bug_report.yml).
- **Suggest features**: open a [feature request](https://github.com/furqanistic/yap/issues/new?template=feature_request.yml).
- **Improve docs**: typo fixes and clarifications are always welcome.
- **Write code**: pick an open issue, or open one first to discuss larger changes.

For anything beyond a small fix, please open an issue before starting work so we can agree on the approach.

## Development setup

1. Install the prerequisites:
   - [Node.js](https://nodejs.org) 20+
   - [Rust](https://www.rust-lang.org/tools/install) (stable)
   - CMake and a C++ compiler (see [Prerequisites](README.md#prerequisites) for per-OS details)
   - The [Tauri system dependencies](https://tauri.app/start/prerequisites/) for your OS
2. Fork and clone the repository:
   ```bash
   git clone https://github.com/<your-username>/yap.git
   cd yap
   ```
3. Install dependencies and start the app:
   ```bash
   npm install
   npm run tauri dev
   ```

## Making changes

1. Create a branch from `main`:
   ```bash
   git checkout -b feat/short-description
   ```
2. Make your changes, following the existing structure (see [Project structure](README.md#project-structure)).
3. Make sure everything passes locally before pushing:
   ```bash
   npm run build
   cd src-tauri
   cargo fmt --check
   cargo clippy --all-targets -- -D warnings
   cargo test
   ```
4. Commit using [Conventional Commits](https://www.conventionalcommits.org/):
   - `feat: add push-to-talk shortcut`
   - `fix: handle missing microphone permission`
   - `docs: clarify build steps`
   - `chore: bump dependencies`
5. Push your branch and open a pull request against `main`. Fill in the PR template and link any related issue.

## Pull request guidelines

- Keep PRs focused: one logical change per PR.
- Update documentation when behavior changes.
- Add an entry under `Unreleased` in [CHANGELOG.md](CHANGELOG.md) for user-facing changes.
- CI must pass before a PR can be merged.

## Code style

- **UI and design:** follow [DESIGN.md](DESIGN.md) for colors, typography, spacing, icons, components, and copy. AI coding tools pick this up automatically through [AGENTS.md](AGENTS.md) and [CLAUDE.md](CLAUDE.md).
- **TypeScript/React:** functional components, named exports for feature components, and the `@/` import alias for anything under `src/`.
- **Rust:** formatted with `rustfmt` and free of `clippy` warnings.
- Formatting basics (indentation, line endings) are defined in [.editorconfig](.editorconfig).

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
