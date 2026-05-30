# ADR-001: Initial Architecture — Tauri v2 + React 19 + ComfyUI Sidecar

**Date:** 2026-05-30  
**Status:** Accepted  
**Sprint:** 0

## Context

MandingoForge requires a professional desktop application that feels like high-end creative software (ComfyUI + InvokeAI/Forge class) while being completely fetish-native and 100% local/private.

Key constraints from the Executive Producer / Roadmap:
- Tauri v2 (Rust) for the desktop shell (lightweight, secure, small bundle, excellent cross-platform)
- React 19 + TypeScript frontend
- ComfyUI as the headless AI engine (sidecar process)
- Strict folder structure separating concerns (frontend/, src-tauri/, comfyui/, workflows/, models/, gallery/, etc.)
- Dark luxurious Mandingo-inspired aesthetics from day one

## Decision

We adopted the following architecture for Sprint 0 and the foundation of v1.0:

### 1. Monorepo Layout (exactly as specified in Roadmap)
- `frontend/` — All React/Vite/TS code, Tailwind, future shadcn/ui components
- `src-tauri/` — Rust crate containing the Tauri application, sidecar orchestration, and IPC commands
- `comfyui/` — Git-cloned ComfyUI (user-managed, large, mostly gitignored)
- `workflows/`, `models/`, `gallery/`, `assets/`, `docs/` at root for clear ownership

### 2. ComfyUI as Managed Sidecar (not embedded)
- Python process spawned and lifecycle-controlled from Rust (`std::process::Child`)
- Tauri commands: `start_comfyui`, `stop_comfyui`, `get_comfyui_status`
- Health checks performed via HTTP (reqwest) against the ComfyUI REST API (port 8188 by default)
- This gives us full control, logging, restart capability, and future multi-instance support

### 3. IPC Strategy
- Primary: Tauri `invoke()` for all control-plane operations (sidecar mgmt, config, queue)
- Secondary: Direct HTTP + WebSocket from frontend to ComfyUI (once running) for prompt submission and progress (standard ComfyUI pattern)
- This hybrid approach is the industry standard for Tauri + ComfyUI wrappers and gives maximum performance + flexibility.

### 4. Theming & UX Foundation
- Tailwind CSS v3 + custom design tokens for deep blacks (#0A0A0A), golds (#C5A46E), blood reds (#8B0000)
- Framer Motion for premium micro-interactions and entrance animations
- shadcn/ui + Radix planned for Sprints 2–3 (component library)
- All UI text and interactions designed to feel native to the gay interracial fetish space from the splash screen onward

### 5. Configuration & State
- Initial app config exposed via `get_app_config`
- Sidecar port and paths stored in Rust state (expandable to file-based config in Sprint 1+)
- No external databases in v1 — everything local filesystem + JSON

## Consequences

**Positive:**
- Excellent separation of concerns and future scalability
- True professional desktop experience (native menus, window chrome, performance)
- Full control over ComfyUI lifecycle (critical for reliability on consumer hardware)
- Theme and branding established early — no "generic AI app" smell

**Negative / Trade-offs:**
- Slightly more complex initial dev setup (users must clone ComfyUI themselves in Sprint 0)
- Python process management on Windows requires careful stdio/creation flag handling (implemented)
- Future updates to ComfyUI may require pinning + migration scripts (documented in Roadmap risks)

## Alternatives Considered

- **Electron + ComfyUI**: Rejected — too heavy, worse security model, larger bundles. Tauri v2 was explicitly required.
- **ComfyUI as Rust port / embedded**: Rejected — ComfyUI's Python ecosystem (custom nodes, IPAdapter, ControlNet, AnimateDiff, training) is irreplaceable for the niche in the 2025-2026 timeframe.
- **Fully bundled sidecar binary**: Deferred. Python + torch is massive; current approach (user clones + manages their own ComfyUI) is the pragmatic industry pattern used by SwarmUI, Pinokio, etc.

## Next Steps (Future ADRs)
- ADR-002: Workflow execution & prompt queuing strategy (Sprint 1)
- ADR-003: Model management & automatic folder synchronization
- ADR-004: Consistency engine (IPAdapter + reference images) architecture

---

**Decision Makers:** Executive Producer (Mandingo_Fever) + Senior Developer (Grok)  
**Approved for implementation in Sprint 0.**