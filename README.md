# 🧠 Second Brain — Personal Intelligence OS

> A cross-platform desktop app that remembers **everything** you do on your computer — with a voice AI assistant that monitors your system and alerts you to critical events.

---

## ✨ Features

### 🗂️ Activity Memory
- **Clipboard history** — every text you copy, searchable instantly
- **App usage tracking** — time per app, categorized (dev, browser, productivity, etc.)
- **File events** — created, modified, deleted files in your home directories
- **Browser history** — reads Chrome, Firefox, Edge, Brave, Arc, Safari directly
- **Active window tracking** — which app/window is focused and for how long
- **Searchable timeline** — full-text search across everything via SQLite FTS5

### 🤖 Aria — Voice AI Assistant
- **Critical system alerts** spoken aloud:
  - CPU overload / overheating
  - RAM critical
  - Disk almost full
  - Battery low / critical
- **Two AI modes:**
  - 🏠 **Ollama (local)** — runs entirely on your machine, 100% private (llama3, mistral, phi3, gemma2...)
  - ☁️ **OpenRouter (cloud)** — GPT-4o-mini, Claude Haiku, Mistral, etc.
- **Conversational chat** — ask Aria anything about your day, system, productivity
- **Proactive summaries** — Aria speaks a productivity insight every N minutes
- **TTS engines supported:**
  - macOS: `say` (built-in, no install)
  - Windows: PowerShell SAPI (built-in, no install)
  - Linux: `espeak-ng`, `festival`, or `piper` (neural TTS)

### 📊 System Monitoring (live)
- CPU usage + top processes
- RAM / disk / network
- CPU temperature (where available)
- Battery status

### 🔐 Privacy
- **100% local** — all data in SQLite in your app data folder
- Configurable excluded apps and paths
- Data retention controls (auto-prune after N days)
- Export everything as JSON

---

## 🚀 Setup

### Prerequisites

```bash
# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Install Node.js (v18+)
# https://nodejs.org

# 3. Install Tauri CLI
cargo install tauri-cli --version "^2.0"

# 4. Platform-specific deps:

# macOS — nothing extra needed (say + Xcode tools)
xcode-select --install

# Ubuntu/Debian
sudo apt update && sudo apt install -y \
  libwebkit2gtk-4.1-dev libappindicator3-dev \
  librsvg2-dev patchelf espeak-ng xdotool xclip

# Windows — install WebView2 (usually pre-installed on Win11)
# https://developer.microsoft.com/en-us/microsoft-edge/webview2/
```

### Clone & Run

```bash
git clone https://github.com/you/second-brain
cd second-brain

npm install
cargo tauri dev        # development (hot reload)
cargo tauri build      # production build (creates platform-specific binary)
```

## 📦 Building for All Platforms

The app can be built for Windows, macOS, and Linux using the same commands. Tauri handles the platform-specific build process.

### Development Mode
```bash
npm run desktop:dev
```
Starts the app in development mode with hot reloading.

### Production Builds

#### Build for Current Platform
```bash
npm run desktop:build
```
Builds a production version for the platform you're currently running on.

#### Platform-Specific Build Targets
You can also build specific installers using the Tauri CLI directly:

**Windows:**
```bash
# NSIS installer
cargo tauri build --bundles nsis

# MSI installer
cargo tauri build --bundles msi
```

**macOS:**
```bash
# DMG installer
cargo tauri build --bundle --targets dmg

# PKG installer
cargo tauri build --bundle --targets pkg
```

**Linux:**
```bash
# AppImage
cargo tauri build --bundle --targets appimage

# DEB package
cargo tauri build --bundle --targets deb
```

### Output Locations
After building, distributables are placed in:
```text
src-tauri/target/release/bundle/
```

#### Windows Outputs:
- `src-tauri/target/release/bundle/nsis/*.exe` - NSIS installer
- `src-tauri/target/release/bundle/msi/*.msi` - MSI installer

#### macOS Outputs:
- `src-tauri/target/release/bundle/dmg/*.dmg` - DMG disk image
- `src-tauri/target/release/bundle/pkg/*.pkg` - macOS installer package

#### Linux Outputs:
- `src-tauri/target/release/bundle/appimage/*.AppImage` - AppImage
- `src-tauri/target/release/bundle/deb/*.deb` - Debian package

### Testing Builds Locally
To test the packaged app without creating installers:
```bash
npm run desktop:dev  # Development mode with hot reload
# OR
cargo tauri build    # Production binary (no installer)
```
The production binary will be in `src-tauri/target/release/` and can be run directly.

---

## 🤖 AI Setup

### Option A: Ollama (Local, Recommended)

```bash
# Install Ollama
curl -fsSL https://ollama.ai/install.sh | sh

# Pull a model (choose one):
ollama pull llama3          # Best quality (4.7 GB)
ollama pull mistral         # Great, faster (4.1 GB)
ollama pull phi3            # Lightweight (2.3 GB)
ollama pull gemma2          # Google, solid (5.4 GB)
ollama pull llama3.2:1b     # Tiny, very fast (1.3 GB)

# Start Ollama (usually auto-starts)
ollama serve
```

Then open Aria (bottom-right button) → Settings → Select **Ollama** → pick your model → Save.

### Option B: OpenRouter (Cloud)

1. Get a free API key at [openrouter.ai](https://openrouter.ai)
2. In Aria → Settings → Select **OpenRouter** → paste key → Save
3. GPT-4o-mini gives 1M free tokens/month on a free account

---

## 🎙️ Voice Setup (Linux)

```bash
# espeak-ng (basic, robotic but works everywhere)
sudo apt install espeak-ng

# Festival (better quality)
sudo apt install festival festvox-kallpc16k

# Piper (best quality, neural TTS, needs download)
pip install piper-tts
piper --download-dir ~/.local/share/piper-voices \
      --update-voices en_US-lessac-medium
```

---

## 📁 Project Structure

```
second-brain/
├── src/                        # React frontend
│   ├── components/
│   │   ├── assistant/
│   │   │   ├── VoiceAssistant.tsx     ← Aria chat + alert panel
│   │   │   └── VoiceAssistant.module.css
│   │   ├── dashboard/Dashboard.tsx    ← Live system overview
│   │   ├── timeline/Timeline.tsx      ← Activity timeline
│   │   ├── search/SearchView.tsx      ← Full-text search
│   │   ├── timeline/ClipboardView.tsx
│   │   ├── timeline/BrowserView.tsx
│   │   ├── timeline/FilesView.tsx
│   │   ├── insights/InsightsView.tsx  ← AI productivity insights
│   │   ├── settings/SettingsView.tsx
│   │   ├── Sidebar.tsx
│   │   └── Topbar.tsx
│   ├── store/index.ts           ← Zustand global state
│   └── utils/api.ts             ← Tauri invoke wrappers
│
└── src-tauri/src/              # Rust backend
    ├── lib.rs                   ← App setup + plugin registration
    ├── db/mod.rs                ← SQLite schema + all queries
    ├── monitors/
    │   ├── mod.rs               ← Monitor orchestrator
    │   ├── window_monitor.rs    ← Active window + app usage
    │   ├── file_monitor.rs      ← File system watcher (notify)
    │   └── browser_monitor.rs   ← Browser history reader
    ├── ai/
    │   ├── mod.rs               ← AI engine (OpenRouter)
    │   ├── voice_assistant.rs   ← Aria: TTS + LLM + alert engine
    │   └── assistant_commands.rs ← Tauri commands for assistant
    ├── commands/mod.rs          ← All Tauri IPC commands
    └── utils/mod.rs
```

---

## 🔧 Alert Thresholds (defaults)

| Alert | Warn | Critical |
|-------|------|----------|
| CPU | 75% | 92% |
| RAM | 78% | 90% |
| Disk | — | 90% |
| CPU Temp | — | 85°C |
| Battery | 20% | 10% |

Cooldown between same-type alerts: **5 minutes** (configurable).

All thresholds are configurable in Aria → Settings.

---

## 🗣️ Example Aria Conversations

> **You:** "What have I been working on today?"
> **Aria:** "You've spent about 3 hours in VS Code, 45 minutes in Chrome, and had 12 clipboard copies. Looks like a focused development day."

> **You:** "How's my system right now?"
> **Aria:** "CPU is at 34%, RAM at 62% — everything looks healthy. Disk is 71% full, nothing to worry about yet."

> **[Automatic]** "Warning. CPU usage is high at 87 percent. The top process is node.js." *(spoken aloud)*

> **You:** "Which apps am I wasting the most time in?"
> **Aria:** "Based on today, you spent 2 hours in Slack versus 3 hours in VS Code. Your communication-to-coding ratio looks reasonable, but Slack is eating 28% of your active time."

---

## 📊 Database Location

| OS | Path |
|----|------|
| macOS | `~/Library/Application Support/com.secondbrain.app/second_brain.db` |
| Windows | `%APPDATA%\com.secondbrain.app\second_brain.db` |
| Linux | `~/.local/share/com.secondbrain.app/second_brain.db` |

You can open it with any SQLite browser (e.g., DB Browser for SQLite).

---

## 🛣️ Roadmap

- [ ] STT (speech-to-text for voice commands via Whisper.cpp)
- [ ] Screenshot OCR (make screen content searchable)
- [ ] Coding session detection (git commits, file save patterns)
- [ ] Weekly email digest
- [ ] Plugin system for custom monitors
- [ ] Mobile companion app (view timeline remotely)
- [ ] RAG-based memory search (semantic similarity)

---

## License

MIT — do whatever you want with it.
