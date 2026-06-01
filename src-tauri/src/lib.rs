// MandingoForge - Tauri v2 + ComfyUI Sidecar Management
// Professional local AI generative studio foundation

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{Manager, State, WindowEvent, Wry};
use tauri_plugin_store::{Store, StoreExt};

// ========================================
// Persistent Settings (user can point to ANY ComfyUI + ANY venv)
// ========================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSettings {
    /// Full path to the ComfyUI folder (must contain main.py)
    pub comfyui_path: Option<String>,
    /// Full path to the Python executable to use (e.g. from your chosen venv)
    /// Example on Windows: C:\Users\you\venvs\comfy\Scripts\python.exe
    pub python_path: Option<String>,
    /// Default port to launch ComfyUI on
    pub default_port: Option<u16>,
    /// Extra arguments to pass to ComfyUI (e.g. "--cuda-device 0 --force-fp16")
    /// Useful for advanced setups like ComfyUI-Distributed + Cloudflare tunneling.
    pub extra_args: Option<String>,
}

impl AppSettings {
    pub const STORE_KEY: &'static str = "app_settings";

    pub fn load(store: &Store<Wry>) -> Self {
        store
            .get(Self::STORE_KEY)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, store: &Store<Wry>) -> Result<(), String> {
        let value = serde_json::to_value(self).map_err(|e| e.to_string())?;
        store.set(Self::STORE_KEY, value);
        store.save().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn effective_port(&self) -> u16 {
        self.default_port.unwrap_or(8188)
    }
}

// ========================================
// Environment Variable Support + Resolution
// ========================================

/// Environment variable names for private/custom ComfyUI setups.
/// These act as fallbacks when no value is saved in the app Settings.
pub const ENV_COMFYUI_PATH: &str = "COMFYUI_PATH";
pub const ENV_COMFYUI_PYTHON: &str = "COMFYUI_PYTHON";
pub const ENV_COMFYUI_PORT: &str = "COMFYUI_PORT";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectiveSettings {
    /// The ComfyUI folder that will actually be used
    pub comfyui_path: String,
    /// The Python executable that will actually be used
    pub python_path: String,
    /// The port that will actually be used
    pub default_port: u16,

    /// Where the value came from (for UI transparency)
    pub comfyui_source: String,
    pub python_source: String,
    pub port_source: String,

    /// Extra arguments that will be passed (from saved settings or COMFYUI_EXTRA_ARGS env var)
    pub extra_args: Option<String>,
}

impl AppSettings {
    /// Resolves the final values to use, with this priority:
    /// 1. Saved settings in the app (highest - user explicitly chose via Settings UI)
    /// 2. Environment variables (COMFYUI_PATH, COMFYUI_PYTHON, COMFYUI_PORT)
    /// 3. Built-in defaults
    pub fn resolve_effective(&self) -> EffectiveSettings {
        // ComfyUI Path
        let (comfyui_path, comfyui_source) = if let Some(p) = &self.comfyui_path {
            if !p.trim().is_empty() {
                (p.clone(), "Saved Settings".to_string())
            } else if let Ok(env) = std::env::var(ENV_COMFYUI_PATH) {
                if !env.trim().is_empty() {
                    (env, format!("Environment Variable ({})", ENV_COMFYUI_PATH))
                } else {
                    Self::default_comfyui_path()
                }
            } else {
                Self::default_comfyui_path()
            }
        } else if let Ok(env) = std::env::var(ENV_COMFYUI_PATH) {
            if !env.trim().is_empty() {
                (env, format!("Environment Variable ({})", ENV_COMFYUI_PATH))
            } else {
                Self::default_comfyui_path()
            }
        } else {
            Self::default_comfyui_path()
        };

        // Python Executable
        let (python_path, python_source) = if let Some(p) = &self.python_path {
            if !p.trim().is_empty() {
                (p.clone(), "Saved Settings".to_string())
            } else if let Ok(env) = std::env::var(ENV_COMFYUI_PYTHON) {
                if !env.trim().is_empty() {
                    (env, format!("Environment Variable ({})", ENV_COMFYUI_PYTHON))
                } else {
                    Self::default_python_path()
                }
            } else {
                Self::default_python_path()
            }
        } else if let Ok(env) = std::env::var(ENV_COMFYUI_PYTHON) {
            if !env.trim().is_empty() {
                (env, format!("Environment Variable ({})", ENV_COMFYUI_PYTHON))
            } else {
                Self::default_python_path()
            }
        } else {
            Self::default_python_path()
        };

        // Port
        let (port, port_source) = if let Some(p) = self.default_port {
            (p, "Saved Settings".to_string())
        } else if let Ok(env) = std::env::var(ENV_COMFYUI_PORT) {
            if let Ok(parsed) = env.trim().parse::<u16>() {
                (parsed, format!("Environment Variable ({})", ENV_COMFYUI_PORT))
            } else {
                (8188u16, "Default (invalid COMFYUI_PORT)".to_string())
            }
        } else {
            (8188u16, "Default".to_string())
        };

        let resolved_extra = self.extra_args.clone().or_else(|| {
            std::env::var("COMFYUI_EXTRA_ARGS").ok().filter(|v| !v.trim().is_empty())
        });

        EffectiveSettings {
            comfyui_path,
            python_path,
            default_port: port,
            comfyui_source,
            python_source,
            port_source,
            extra_args: resolved_extra,
        }
    }

    fn default_comfyui_path() -> (String, String) {
        let path = std::env::current_dir()
            .unwrap_or_default()
            .join("comfyui")
            .to_string_lossy()
            .to_string();
        (path, "Default (./comfyui)".to_string())
    }

    fn default_python_path() -> (String, String) {
        ("python".to_string(), "Default (python in PATH)".to_string())
    }
}

// ========================================
// ComfyUI Sidecar Manager
// ========================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComfyStatus {
    pub running: bool,
    pub port: u16,
    pub pid: Option<u32>,
    pub health: String, // "healthy" | "unreachable" | "stopped"
    pub message: String,
}

pub struct ComfyManager {
    child: Arc<Mutex<Option<Child>>>,
    port: Arc<Mutex<u16>>,
}

/// Spawn a background thread that forwards a Child's stdout or stderr line-by-line to println!.
/// Keeps the main thread responsive and gives immediate visibility into ComfyUI logs/errors.
fn spawn_log_reader<R: std::io::Read + Send + 'static>(reader: R, stream: &'static str) {
    std::thread::spawn(move || {
        use std::io::{BufRead, BufReader};
        let buf_reader = BufReader::new(reader);
        for line in buf_reader.lines() {
            match line {
                Ok(l) => println!("[ComfyUI {}] {}", stream, l),
                Err(_) => break,
            }
        }
    });
}

impl ComfyManager {
    pub fn new() -> Self {
        Self {
            child: Arc::new(Mutex::new(None)),
            port: Arc::new(Mutex::new(8188)),
        }
    }

    /// Start ComfyUI using the paths from AppSettings (supports ANY location + ANY venv).
    /// Now respects environment variable fallbacks when no saved settings exist.
    pub fn start(&self, settings: &AppSettings, requested_port: Option<u16>) -> Result<ComfyStatus, String> {
        let mut child_guard = self.child.lock().map_err(|e| e.to_string())?;
        
        if child_guard.is_some() {
            return Err("ComfyUI sidecar is already running".to_string());
        }

        let effective = settings.resolve_effective();

        let port = requested_port.unwrap_or(effective.default_port);
        *self.port.lock().map_err(|e| e.to_string())? = port;

        let comfyui_dir = PathBuf::from(&effective.comfyui_path);
        let python_exe = effective.python_path.clone();

        // Resolve extra args (prefer saved, then env var COMFYUI_EXTRA_ARGS as fallback)
        let extra_args: Option<String> = settings.extra_args.clone().or_else(|| {
            std::env::var("COMFYUI_EXTRA_ARGS").ok().filter(|v| !v.trim().is_empty())
        });

        // Verify ComfyUI folder
        let main_py = comfyui_dir.join("main.py");
        if !main_py.exists() {
            return Err(format!(
                "ComfyUI not found at {}.\n\nSource: {}\nPlease either set the correct path in Settings or set the {} environment variable.",
                comfyui_dir.display(),
                effective.comfyui_source,
                ENV_COMFYUI_PATH
            ));
        }

        // Verify the chosen Python exists (critical for venv support)
        let python_path = PathBuf::from(&python_exe);
        if !python_path.exists() {
            return Err(format!(
                "Python executable not found at: {}\n\nSource: {}\nPlease select the python.exe from your desired venv in Settings or set the {} environment variable.",
                python_exe,
                effective.python_source,
                ENV_COMFYUI_PYTHON
            ));
        }

        let mut cmd = Command::new(&python_exe);
        cmd.current_dir(&comfyui_dir)
            .arg("-u")
            .arg("main.py")
            .arg("--listen")
            .arg("localhost")
            .arg("--port")
            .arg(port.to_string());

        // Append any user-provided extra arguments (useful for ComfyUI-Distributed, Cloudflare, etc.)
        if let Some(extra) = &extra_args {
            // Simple split on whitespace (users can use quotes if needed for paths with spaces)
            for arg in extra.split_whitespace() {
                if !arg.is_empty() {
                    cmd.arg(arg);
                }
            }
        }

        // Always enable CORS header. This desktop app uses a separate frontend (Vite on :1420)
        // talking to ComfyUI on a different port. This flag helps avoid cross-origin blocks.
        cmd.arg("--enable-cors-header");
        cmd.arg("*");

        cmd.stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Windows: hide console window for the Python process
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }

        match cmd.spawn() {
            Ok(mut child) => {
                let pid = child.id();

                // Take stdout/stderr handles and forward them in background threads
                if let Some(stdout) = child.stdout.take() {
                    spawn_log_reader(stdout, "stdout");
                }
                if let Some(stderr) = child.stderr.take() {
                    spawn_log_reader(stderr, "stderr");
                }

                *child_guard = Some(child);

                std::thread::sleep(Duration::from_millis(1400));

                Ok(ComfyStatus {
                    running: true,
                    port,
                    pid: Some(pid),
                    health: "starting".to_string(),
                    message: format!(
                        "ComfyUI started using {} (pid {}) on port {}  |  Source: {} + {}",
                        python_exe, pid, port,
                        effective.comfyui_source, effective.python_source
                    ),
                })
            }
            Err(e) => Err(format!(
                "Failed to launch ComfyUI with Python at {}: {}",
                python_exe, e
            )),
        }
    }

    pub fn stop(&self) -> Result<ComfyStatus, String> {
        let mut child_guard = self.child.lock().map_err(|e| e.to_string())?;
        
        if let Some(mut child) = child_guard.take() {
            let port = self.port.lock().map(|g| *g).unwrap_or(8188);

            // Graceful attempt: send an interrupt to ComfyUI before hard-killing the process.
            // This dramatically reduces "ConnectionResetError" + proactor transport crashes
            // on the Python/Windows asyncio side (exactly the error reported in issues).
            // We fire-and-forget the request and give it a short grace period.
            let _ = std::thread::spawn(move || {
                let client = reqwest::blocking::Client::builder()
                    .timeout(std::time::Duration::from_millis(800))
                    .build();

                if let Ok(c) = client {
                    let _ = c.post(format!("http://127.0.0.1:{}/interrupt", port))
                        .json(&serde_json::json!({}))
                        .send();
                }
            });

            // Give ComfyUI a moment to react to the interrupt and start flushing shutdown.
            std::thread::sleep(Duration::from_millis(1100));

            let _ = child.kill();
            let _ = child.wait();

            Ok(ComfyStatus {
                running: false,
                port,
                pid: None,
                health: "stopped".to_string(),
                message: "ComfyUI sidecar stopped successfully".to_string(),
            })
        } else {
            Ok(ComfyStatus {
                running: false,
                port: self.port.lock().map(|g| *g).unwrap_or(8188),
                pid: None,
                health: "stopped".to_string(),
                message: "No running ComfyUI sidecar".to_string(),
            })
        }
    }

    pub async fn health(&self) -> Result<ComfyStatus, String> {
        let port = self.port.lock().map(|g| *g).unwrap_or(8188);
        let running = self.child.lock().map(|g| g.is_some()).unwrap_or(false);

        if !running {
            return Ok(ComfyStatus {
                running: false,
                port,
                pid: None,
                health: "stopped".to_string(),
                message: "Sidecar not running".to_string(),
            });
        }

        // HTTP health probe against ComfyUI
        let url = format!("http://localhost:{}/", port);
        match reqwest::get(&url).await {
            Ok(resp) if resp.status().is_success() => Ok(ComfyStatus {
                running: true,
                port,
                pid: None, // pid tracked separately
                health: "healthy".to_string(),
                message: format!("ComfyUI responding on port {}", port),
            }),
            Ok(resp) => Ok(ComfyStatus {
                running: true,
                port,
                pid: None,
                health: "unhealthy".to_string(),
                message: format!("ComfyUI returned status {}", resp.status()),
            }),
            Err(e) => Ok(ComfyStatus {
                running: true,
                port,
                pid: None,
                health: "unreachable".to_string(),
                message: format!("Health check failed: {}", e),
            }),
        }
    }
}

// ========================================
// Tauri Commands (exposed to frontend)
// ========================================

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust (MandingoForge IPC).", name)
}

/// Start ComfyUI using the user's configured paths (any folder + any venv)
#[tauri::command]
async fn start_comfyui(
    port: Option<u16>,
    state: State<'_, ComfyManager>,
    app: tauri::AppHandle,
) -> Result<ComfyStatus, String> {
    let store = app.store("settings.dat").map_err(|e| e.to_string())?;
    let settings = AppSettings::load(&store);
    state.start(&settings, port)
}

#[tauri::command]
async fn stop_comfyui(state: State<'_, ComfyManager>) -> Result<ComfyStatus, String> {
    state.stop()
}

#[tauri::command]
async fn get_comfyui_status(state: State<'_, ComfyManager>) -> Result<ComfyStatus, String> {
    state.health().await
}

/// Returns the user's saved settings (ComfyUI path, Python/venv path, port, etc.)
#[tauri::command]
async fn get_settings(app: tauri::AppHandle) -> Result<AppSettings, String> {
    let store = app.store("settings.dat").map_err(|e| e.to_string())?;
    Ok(AppSettings::load(&store))
}

/// Returns the **effective** configuration that will actually be used,
/// after applying priority: Saved Settings > Environment Variables > Defaults.
/// This is what the sidecar will use when launching.
#[tauri::command]
async fn get_effective_settings(app: tauri::AppHandle) -> Result<EffectiveSettings, String> {
    let store = app.store("settings.dat").map_err(|e| e.to_string())?;
    let settings = AppSettings::load(&store);
    Ok(settings.resolve_effective())
}

/// Saves settings to disk and returns the saved object
#[tauri::command]
async fn save_settings(settings: AppSettings, app: tauri::AppHandle) -> Result<AppSettings, String> {
    let store = app.store("settings.dat").map_err(|e| e.to_string())?;
    settings.save(&store)?;
    Ok(settings)
}

/// Clears all saved settings. After this, the app will fall back to
/// environment variables (COMFYUI_PATH / COMFYUI_PYTHON / COMFYUI_PORT) or defaults.
#[tauri::command]
async fn clear_settings(app: tauri::AppHandle) -> Result<(), String> {
    let store = app.store("settings.dat").map_err(|e| e.to_string())?;
    let empty = AppSettings::default();
    empty.save(&store)?;
    Ok(())
}

/// Open a native folder picker (for choosing ComfyUI installation)
#[tauri::command]
async fn pick_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let folder = app.dialog().file().blocking_pick_folder();
    Ok(folder.map(|p| p.to_string()))
}

/// Open a native file picker (for choosing python.exe from a venv)
#[tauri::command]
async fn pick_python_executable(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let file = app.dialog()
        .file()
        .add_filter("Python Interpreter", &[ "exe", "" ])  // Windows .exe or any file on other OS
        .blocking_pick_file();
    Ok(file.map(|p| p.to_string()))
}

/// Lists available checkpoint models by scanning the checkpoints folder
/// using the effective ComfyUI configuration (Saved Settings → Env → Defaults).
/// Returns only filenames (e.g. "realisticVisionV51.safetensors").
/// This is used by the Sprint 1 Quick Forge model selector.
#[tauri::command]
async fn list_checkpoints(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    let store = app.store("settings.dat").map_err(|e| e.to_string())?;
    let settings = AppSettings::load(&store);
    let effective = settings.resolve_effective();

    let checkpoints_dir = PathBuf::from(&effective.comfyui_path)
        .join("models")
        .join("checkpoints");

    if !checkpoints_dir.exists() {
        // No models folder yet — user hasn't placed any checkpoints.
        // Return empty list; frontend will show strong guidance.
        return Ok(vec![]);
    }

    let mut models: Vec<String> = Vec::new();
    let allowed_exts = [".safetensors", ".ckpt", ".pt", ".pth", ".bin", ".gguf"];

    match std::fs::read_dir(&checkpoints_dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        if let Some(name) = entry.file_name().to_str() {
                            let lower = name.to_lowercase();
                            if allowed_exts.iter().any(|ext| lower.ends_with(ext)) {
                                models.push(name.to_string());
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            return Err(format!(
                "Failed to read checkpoints directory at {}: {}",
                checkpoints_dir.display(),
                e
            ));
        }
    }

    models.sort();
    Ok(models)
}

/// Detailed readiness probe. Distinguishes between "sidecar process running"
/// and "ComfyUI fully initialized with models loaded and ready to accept prompts".
/// Used to give users accurate feedback during the long 30-90s warm-up on first launch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComfyReadiness {
    pub status: String, // "stopped" | "starting" | "loading_models" | "ready" | "unreachable"
    pub message: String,
    pub port: u16,
}

#[tauri::command]
async fn get_comfyui_readiness(state: State<'_, ComfyManager>) -> Result<ComfyReadiness, String> {
    let port = state.port.lock().map(|g| *g).unwrap_or(8188);
    let basic_running = state.child.lock().map(|g| g.is_some()).unwrap_or(false);

    if !basic_running {
        return Ok(ComfyReadiness {
            status: "stopped".to_string(),
            message: "ComfyUI sidecar is not running".to_string(),
            port,
        });
    }

    // First basic HTTP probe
    let base_url = format!("http://localhost:{}", port);
    let root_resp = reqwest::get(&base_url).await;

    if root_resp.is_err() || !root_resp.unwrap().status().is_success() {
        return Ok(ComfyReadiness {
            status: "loading_models".to_string(),
            message: "Sidecar running — ComfyUI still starting up...".to_string(),
            port,
        });
    }

    // Stronger probe: system_stats only responds cleanly once core systems are up.
    // If this fails or times out, models are likely still loading.
    let stats_url = format!("{}/system_stats", base_url);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(4))
        .build()
        .map_err(|e| e.to_string())?;

    match client.get(&stats_url).send().await {
        Ok(resp) if resp.status().is_success() => {
            // Successfully got system stats → ComfyUI is ready for work
            Ok(ComfyReadiness {
                status: "ready".to_string(),
                message: format!("ComfyUI ready on port {}", port),
                port,
            })
        }
        _ => Ok(ComfyReadiness {
            status: "loading_models".to_string(),
            message: "Sidecar running but still loading models (this can take 30–90 seconds on first launch)...".to_string(),
            port,
        }),
    }
}

// ========================================
// Sprint 1: Generation Saving (PNG + sidecar metadata JSON)
// ========================================

#[derive(Debug, Deserialize)]
pub struct SaveGenerationRequest {
    pub filename: String,
    pub image_base64: String,
    pub prompt: String,
    pub negative_prompt: String,
    pub model: String,
    pub steps: u32,
    pub cfg: f32,
    pub seed: i64,
    pub width: u32,
    pub height: u32,
    pub sampler: Option<String>,
    /// ISO timestamp string provided by the frontend (avoids extra Rust deps)
    pub saved_at: String,
}

#[tauri::command]
async fn save_generation(request: SaveGenerationRequest) -> Result<String, String> {
    // Decode base64 image data
    let image_bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        &request.image_base64,
    )
    .map_err(|e| format!("Invalid base64 image data: {}", e))?;

    // Resolve gallery directory next to the executable / current working dir
    let gallery_dir = std::env::current_dir()
        .unwrap_or_default()
        .join("gallery");

    std::fs::create_dir_all(&gallery_dir)
        .map_err(|e| format!("Failed to create gallery directory: {}", e))?;

    let png_path = gallery_dir.join(&request.filename);
    std::fs::write(&png_path, &image_bytes)
        .map_err(|e| format!("Failed to write image file: {}", e))?;

    // Write companion metadata JSON (same base name)
    let json_filename = request.filename.replace(".png", ".json");
    let json_path = gallery_dir.join(json_filename);

    let metadata = serde_json::json!({
        "saved_at": request.saved_at,
        "prompt": request.prompt,
        "negative_prompt": request.negative_prompt,
        "model": request.model,
        "steps": request.steps,
        "cfg": request.cfg,
        "seed": request.seed,
        "width": request.width,
        "height": request.height,
        "sampler": request.sampler,
        "source": "MandingoForge Sprint 1"
    });

    let metadata_str = serde_json::to_string_pretty(&metadata)
        .map_err(|e| e.to_string())?;
    std::fs::write(&json_path, metadata_str)
        .map_err(|e| format!("Failed to write metadata file: {}", e))?;

    Ok(format!(
        "Saved to gallery/{} + {}",
        request.filename,
        json_path.file_name().unwrap().to_string_lossy()
    ))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).try_init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .manage(ComfyManager::new())
        .invoke_handler(tauri::generate_handler![
            greet,
            start_comfyui,
            stop_comfyui,
            get_comfyui_status,
            get_settings,
            save_settings,
            clear_settings,
            get_effective_settings,
            pick_folder,
            pick_python_executable,
            list_checkpoints,
            get_comfyui_readiness,
            save_generation,
        ])
        .setup(|_app| {
            println!("[MandingoForge] Tauri app initialized. Settings + flexible ComfyUI sidecar ready.");
            Ok(())
        })
        // Simple shutdown hook: ensure ComfyUI sidecar is killed when the window/app closes.
        // This prevents orphaned Python processes on force-quit or normal close.
        .on_window_event(|window, event| {
            if matches!(event, WindowEvent::CloseRequested { .. } | WindowEvent::Destroyed) {
                if let Some(state) = window.app_handle().try_state::<ComfyManager>() {
                    let _ = state.stop();
                    println!("[MandingoForge] Sidecar stopped via window close hook");
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running MandingoForge (Tauri application)");
}
