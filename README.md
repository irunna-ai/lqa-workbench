# LQA Workbench

**Professional Desktop Workbench for Game Localization QA**

AI-assisted desktop application for game localization quality assurance.

Built with Tauri v2 · Rust · React · TypeScript · SQLite · OCR · Image Analysis · Video Analysis · Knowledge Base · Enterprise Desktop UI

---

## Features

- **Image LQA** — Upload screenshots, run OCR, get AI-proposed issues matched against severity rules
- **Video LQA** — Import gameplay footage, extract frames, analyze through the same pipeline
- **OCR Engine** — Tesseract-powered text extraction with bounding geometry
- **Evidence Engine** — Visual red-box and red-bracket annotations over source images
- **Severity Rules** — Import severity matrix; issues matched deterministically
- **Glossary Authority** — Import terminology references; flag inconsistencies
- **Reference Docs** — Provide context documents for AI analysis
- **Knowledge Base** — Project-local semantic search across all imported data
- **Batch Analysis** — Queue multiple images for sequential analysis
- **BYOK** — Supports Ollama (local), OpenAI-compatible, Gemini, LM Studio
- **CSV Export** — Export reviewed issues for reporting
- **Audit Trail** — Append-only event log for every significant action
- **Command Palette** — Ctrl+K keyboard-driven navigation

---

## Architecture

```
┌──────────────────────────────────────────────┐
│              Tauri v2 Desktop Shell           │
├──────────────────┬───────────────────────────┤
│   Frontend       │       Backend             │
│   React 18       │       Rust                │
│   TypeScript     │       SQLite (rusqlite)   │
│   Vite           │       Image processing    │
│   Lucide Icons   │       Video extraction    │
│                  │       AI provider routing  │
│                  │       Evidence engine      │
│                  │       CSV export           │
├──────────────────┴───────────────────────────┤
│              Python Sidecar                   │
│              Glossary parser                   │
│              Severity rules parser             │
│              OCR adapter                       │
└──────────────────────────────────────────────┘
```

### Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop | Tauri 2 (Windows) |
| Backend | Rust |
| Frontend | React 18, TypeScript, Vite |
| Database | SQLite (rusqlite) |
| AI | BYOK — Ollama, OpenAI-compatible, Gemini, LM Studio |
| OCR | Tesseract (via Python sidecar) |
| Video | FFmpeg / FFprobe |
| Parsers | Python 3 + openpyxl |
| Icons | Lucide React |
| Testing | Vitest (frontend), cargo test (backend) |

---

## Screenshots

> Screenshots will be added in a future release.

<!-- ![Dashboard](docs/screenshots/dashboard.png) -->
<!-- ![Image Analysis](docs/screenshots/image-analysis.png) -->
<!-- ![Video Analysis](docs/screenshots/video-analysis.png) -->
<!-- ![Issues](docs/screenshots/issues.png) -->
<!-- ![Knowledge Base](docs/screenshots/knowledge.png) -->
<!-- ![Settings](docs/screenshots/settings.png) -->
<!-- ![About](docs/screenshots/about.png) -->

---

## Folder Structure

```
lqa-workbench/
├── src/                          # React frontend
│   ├── api/
│   │   └── tauri.ts              # Tauri IPC bridge
│   ├── components/               # 21 React components
│   ├── App.tsx                   # Root component + routing
│   ├── main.tsx                  # Entry point
│   ├── styles.css                # Design system (CSS custom properties)
│   └── types.ts                  # Shared TypeScript types
├── src-tauri/                    # Tauri / Rust backend
│   ├── src/                      # 11 Rust source files
│   ├── parsers/                  # Python sidecar parsers
│   ├── icons/                    # Application icons
│   ├── capabilities/             # Tauri security capabilities
│   ├── Cargo.toml                # Rust dependencies
│   └── tauri.conf.json           # Tauri configuration
├── package.json                  # NPM dependencies
├── vite.config.ts                # Vite + Vitest configuration
└── README.md
```

---

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs) — Latest stable (backend compilation)
- [Node.js](https://nodejs.org) — 18+ (frontend tooling)
- [Ollama](https://ollama.com) — Latest (local AI inference, optional)
- [Tesseract OCR](https://github.com/tesseract-ocr/tesseract) — 5+ (text extraction)
- [FFmpeg](https://ffmpeg.org) — Latest (video processing)
- [Python 3](https://python.org) — 3.10+ (sidecar parsers)
- `pip install openpyxl` — Excel parsing

### Install

```bash
# Clone
git clone https://github.com/irunna-ai/lqa-workbench.git
cd lqa-workbench

# Install frontend dependencies
npm install

# Install Python dependencies
pip install openpyxl
```

### Development

```bash
# Run in development mode
npm run tauri dev

# Run frontend tests
npm test

# Type check
npx tsc --noEmit

# Run backend tests
cargo test

# Build for production
npm run build
```

---

## Roadmap

- macOS support
- Linux support
- Plugin architecture for custom analyzers
- Multi-language OCR
- Collaborative review mode
- Custom report templates
- Tracker integrations (Jira, Azure DevOps)
- Automated regression detection
- Accessibility improvements
- Localization of the workbench itself

---

## License

Copyright (c) 2026 Irunna AI. All rights reserved.

See [LICENSE.md](LICENSE.md) for details.

---

## Contributing

Contributions are welcome. Please open an issue first to discuss proposed changes.

- Follow existing code conventions
- Keep changes focused and atomic
- Write tests for new functionality
- Never commit credentials or secrets
- Never store API keys in plaintext

---

## Security

- **Local-first** — All data stays on your machine
- **No telemetry** — No cloud upload, no analytics
- **Secure credentials** — API keys stored in OS keyring
- **Project isolation** — Each project's data strictly separated
- **Least privilege** — Tauri capabilities grant only what is needed

See [SECURITY.md](SECURITY.md) for the full policy.

---

## Acknowledgements

- [Tauri](https://tauri.app) — Desktop application framework
- [Ollama](https://ollama.com) — Local AI inference
- [Tesseract OCR](https://github.com/tesseract-ocr/tesseract) — Optical character recognition
- [FFmpeg](https://ffmpeg.org) — Video processing
- [React](https://react.dev) — UI library
- [Lucide](https://lucide.dev) — Icons
- [Vite](https://vitejs.dev) — Build tooling
