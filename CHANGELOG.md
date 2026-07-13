# Changelog

All notable changes to QuotaLume will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Initial QuotaLume implementation: AI quota monitor for Linux status bar
- Support for OpenAI Codex (OAuth), Claude Code (OAuth), OpenRouter (API key), and Ollama Cloud (API key/cookie)
- Color-coded tray icon (green/yellow/red) based on lowest remaining quota
- Dropdown menu with per-provider usage bars, plan info, and reset timestamps
- Configurable refresh interval via `~/.config/quotalume/config.toml`
- `--once` CLI flag for terminal-only output
- 13 unit tests covering model clamping, credential parsing, and usage response parsing
- Autostart desktop entry for GNOME/KDE
- README with installation and configuration documentation