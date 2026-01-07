# Claude Dashboard

A native desktop dashboard for monitoring Claude Code usage in real-time. Built with Tauri 2.0, React 19, and Rust.

![Version](https://img.shields.io/badge/version-0.8.4-blue)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux-lightgrey)
![License](https://img.shields.io/badge/license-MIT-green)

## Features

- **Real-time Usage Tracking** — Monitor tokens, cost, and messages against your plan limits
- **5-Hour Session Blocks** — Matches Anthropic's rate limit window
- **Multi-Plan Support** — Pro, Max 5x, Max 20x plans with accurate limits
- **WSL Support** — Windows dashboard reads data from WSL Claude Code installations
- **Burn Rate & Predictions** — See when you'll hit limits at current usage
- **Model Distribution** — Track usage by tier (Opus, Sonnet, Haiku)
- **Configurable Settings** — Auto-refresh interval, animations, default plan
- **10+ Themes** — Cyberpunk, Matrix, Dracula, Nord, and more

## Installation

### Windows

Download from [Releases](https://github.com/Kikii95/claude-dashboard-custom/releases):
- `.msi` — Standard Windows installer
- `.exe` — NSIS installer

### Linux

Download from [Releases](https://github.com/Kikii95/claude-dashboard-custom/releases):
- `.AppImage` — Portable, no install required
- `.deb` — Debian/Ubuntu package

## How It Works

1. **Parses JSONL** — Reads all `.jsonl` files from `~/.claude/projects/`
2. **Session Blocks** — Groups entries into 5-hour blocks (rate limit window)
3. **Calculates Usage** — Compares tokens/cost vs plan limits
4. **Displays Metrics** — Shows percentages, burn rate, time until reset

## Supported Plans

| Plan | Output Tokens/5h | Cost/5h | Messages |
|------|------------------|---------|----------|
| Pro | 45,000 | $10.00 | 100 |
| Max 5x | 88,000 | $35.00 | 1,000 |
| Max 20x | 200,000 | $100.00 | 2,500 |

## Building from Source

### Prerequisites

- Node.js 20+
- pnpm 9+
- Rust (stable)
- Tauri 2.0 prerequisites ([see docs](https://v2.tauri.app/start/prerequisites/))

### Commands

```bash
# Install dependencies
pnpm install

# Development mode
pnpm tauri dev

# Build release
pnpm tauri build
```

## Tech Stack

- **Backend**: Rust + Tauri 2.0
- **Frontend**: React 19 + TypeScript + Tailwind CSS
- **Build**: Vite 7.x
- **CI/CD**: GitHub Actions (Windows + Linux builds)

## Screenshots

*Coming soon*

## Limitations

- **No Direct API Access** — Rate limit headers are only available during inference calls
- **Estimation** — Percentages are calculated vs known plan limits, not real-time API limits
- Precision similar to `claude-monitor`

## License

MIT
