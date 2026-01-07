# Changelog

All notable changes to Claude Dashboard.

## [0.8.4] - 2026-01-07

### Added
- Settings panel with configurable options
- Auto-refresh toggle (on/off)
- Refresh interval selector (30s, 1min, 2min, 5min)
- Animations toggle for reduced motion preference
- Default plan selector
- Settings persistence via localStorage
- Last refresh timestamp in header
- Loading overlay during data refresh

### Changed
- Default refresh interval increased from 30s to 1min

## [0.8.3] - 2026-01-06

### Added
- Loading overlay with spinner during refresh
- Visual feedback when parsing JSONL files

## [0.8.2] - 2026-01-06

### Fixed
- Token calculation now uses OUTPUT tokens only (matches Anthropic rate limits)
- Values now align with `claude-monitor` methodology

## [0.8.1] - 2026-01-06

### Added
- WSL path auto-detection for Windows users
- Automatic discovery of Claude data in WSL distros (Ubuntu, Debian, etc.)

### Fixed
- Windows dashboard now reads data from WSL Claude Code installations

## [0.8.0] - 2026-01-06

### Added
- Initial Tauri 2.0 release
- Real-time usage tracking (tokens, cost, messages)
- 5-hour session block system
- Multi-plan support (Pro, Max 5x, Max 20x)
- Burn rate and prediction calculations
- Model distribution by tier
- 10+ theme options
- GitHub Actions CI/CD for Windows and Linux builds
