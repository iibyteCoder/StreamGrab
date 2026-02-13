# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release

## [0.1.0] - 2024-01-01

### Added
- M3U8/MPD/MSS video stream download support
- Multi-threaded downloading with configurable thread count
- Real-time progress display with speed and ETA
- Batch URL import from text files
- Customizable download settings (threads, retry, timeout)
- Mux settings (format, muxer selection)
- Live stream recording support
- Proxy configuration
- Custom headers support
- Dark theme UI
- Task management (pause, resume, retry, delete)
- Download history tracking
- SQLite-based persistent storage

### Technical
- Built with Tauri 2.0 + Vue 3 + TypeScript
- Cross-platform support (Windows, macOS, Linux)

[Unreleased]: https://github.com/your-username/StreamGrab/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/your-username/StreamGrab/releases/tag/v0.1.0
