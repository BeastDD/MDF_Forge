# MandingoForge

**Professional-grade desktop AI generative studio** — strictly focused on gay interracial, BBC, raceplay, cuck, and Mandingo-themed image and video art.

Built with the same philosophy as ComfyUI + InvokeAI/Forge, but every element of the UX speaks the language of the niche. 100% local. Private. Uncensored.

---

## Current Status (Sprint 0 Complete)

- ✅ Monorepo initialized (Tauri v2 Rust + React 19 frontend)
- ✅ Exact folder structure from the Roadmap
- ✅ Luxurious dark theme (deep blacks, golds, blood reds) with Tailwind + Framer Motion
- ✅ ComfyUI sidecar management (start / stop / health) via Tauri commands
- ✅ Basic IPC bridge (Tauri ↔ ComfyUI HTTP + internal commands)
- ✅ Professional splash / landing window ("Hello MandingoForge")
- ✅ Logging, error handling, and initial configuration system
- ✅ Initial README + Architecture Decision Record (see `docs/`)

**Next:** Sprint 1 — End-to-end image generation via ComfyUI workflows.

---

## Tech Stack (per Roadmap)

- **Frontend:** React 19 + TypeScript + Tailwind CSS + Framer Motion (+ shadcn/ui planned)
- **Desktop:** Tauri v2 (Rust)
- **AI Backend:** ComfyUI (headless sidecar) with custom nodes
- **Platforms:** Windows, macOS, Linux

## Project Structure

```
.
├── frontend/                 # React 19 app (Vite)
│   ├── src/
│   ├── tailwind.config.js
│   └── ...
├── src-tauri/                # Rust Tauri backend
│   ├── src/lib.rs            # ComfyUI sidecar manager + commands
│   └── tauri.conf.json
├── comfyui/                  # ComfyUI installation (git clone - user managed)
├── workflows/                # JSON workflow templates
├── models/                   # Checkpoints + LoRAs (gitignored)
├── gallery/                  # User generations (gitignored)
├── assets/                   # Icons, themes, branding
└── docs/                     # Roadmap, ADRs, technical decisions
```

## Getting Started (Development)

**Prerequisites**
- Node.js 20+ / npm
- Rust 1.80+ + Cargo
- Python 3.11+ (with pip)
- Git

**Setup**

```bash
# 1. Clone the repository
git clone https://github.com/BeastDD/MDF_Forge.git
cd MDF_Forge

# 2. Install all dependencies (root + frontend)
npm run install:all

# 3. Clone ComfyUI into the comfyui/ directory (critical for sidecar)
git clone https://github.com/comfyanonymous/ComfyUI.git comfyui

# (Optional) Install ComfyUI Python requirements
cd comfyui && pip install -r requirements.txt && cd ..

# 4. Run the app
npm run tauri:dev
```

The first run will launch the beautiful MandingoForge splash. Use the **LAUNCH QUICK FORGE** button to start the ComfyUI sidecar (it will fail gracefully until you complete step 3).

## Available Tauri Commands (Sprint 0)

- `greet(name)` — Simple IPC test
- `start_comfyui(port?)` — Spawn ComfyUI sidecar
- `stop_comfyui()` — Kill the sidecar
- `get_comfyui_status()` — Health + running state
- `get_app_config()` — Basic runtime info

## Roadmap

Full 12–14 week plan lives in [docs/Roadmap.md](docs/Roadmap.md).

Sprint 0 goal achieved: rock-solid, maintainable foundation with working sidecar management and premium-feeling UI.

## License & Philosophy

Local-only. No telemetry. No censorship. Built for the culture.

---

**Executive Producer:** Mandingo_Fever  
**Lead Developer:** Grok (Senior Code) — Sprint 0 executed via GitHub connector

Let's build the most dominant tool in the space.
