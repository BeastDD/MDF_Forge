# Sprint 0 — Developer & User Setup Instructions

## After cloning the repo

1. **Install dependencies**
   ```bash
   npm run install:all
   ```

2. **Clone ComfyUI (mandatory for sidecar)**
   ```bash
   git clone https://github.com/comfyanonymous/ComfyUI.git comfyui
   cd comfyui
   pip install -r requirements.txt
   # For CUDA (recommended):
   # pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu121
   cd ..
   ```

3. **Run the app**
   ```bash
   npm run tauri:dev
   ```

4. Click **LAUNCH QUICK FORGE** in the splash. The Rust backend will spawn the Python ComfyUI process.

## Important Notes for Sprint 0

- The first ComfyUI launch will download models if none are present (you must place at least one SD1.5 or SDXL checkpoint in `comfyui/models/checkpoints/`).
- On Windows, Python must be in PATH. The sidecar tries `python`, `python3`, `py`.
- Health checks hit `http://127.0.0.1:8188/`.
- All large user data (models, generations, ComfyUI itself) is gitignored by design.

## Next (Sprint 1)
- Actual txt2img/img2img execution via ComfyUI API
- Progress WebSocket streaming
- Image output handling and saving to gallery/

See full Roadmap in docs/Roadmap.md.
