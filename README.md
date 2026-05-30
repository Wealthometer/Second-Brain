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

