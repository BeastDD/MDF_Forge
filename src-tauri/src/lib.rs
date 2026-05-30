// MandingoForge - Tauri v2 + ComfyUI Sidecar Management
// Professional local AI generative studio foundation

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{State, Wry};
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

impl ComfyManager {
    pub fn new() -> Self {
        Self {
            child: Arc::new(Mutex::new(None)),
            port: Arc::new(Mutex::new(8188)),
        }
    }

    /// Start ComfyUI using the paths from AppSettings (supports ANY location + ANY venv)
    pub fn start(&self, settings: &AppSettings, requested_port: Option<u16>) -> Result<ComfyStatus, String> {
        let mut child_guard = self.child.lock().map_err(|e| e.to_string())?;
        
        if child_guard.is_some() {
            return Err("ComfyUI sidecar is already running".to_string());
        }

        let port = requested_port.unwrap_or_else(|| settings.effective_port());
        *self.port.lock().map_err(|e| e.to_string())? = port;

        // Resolve configured paths (fall back to old defaults only if nothing is set)
        let comfyui_dir = match &settings.comfyui_path {
            Some(p) if !p.trim().is_empty() => PathBuf::from(p),
            _ => std::env::current_dir().unwrap_or_default().join("comfyui"),
        };

        let python_exe = match &settings.python_path {
            Some(p) if !p.trim().is_empty() => p.clone(),
            _ => "python".to_string(), // last resort fallback
        };

        // Verify ComfyUI folder
        let main_py = comfyui_dir.join("main.py");
        if !main_py.exists() {
            return Err(format!(
                "ComfyUI not found at {}. Please set the correct path in Settings.",
                comfyui_dir.display()
            ));
        }

        // Verify the chosen Python exists (critical for venv support)
        let python_path = PathBuf::from(&python_exe);
        if !python_path.exists() {
            return Err(format!(
                "Python executable not found at: {}\n\nPlease select the python.exe from your desired venv in Settings.",
                python_exe
            ));
        }

        let mut cmd = Command::new(&python_exe);
        cmd.current_dir(&comfyui_dir)
            .arg("-u")
            .arg("main.py")
            .arg("--listen")
            .arg("127.0.0.1")
            .arg("--port")
            .arg(port.to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Windows: hide console window for the Python process
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }

        match cmd.spawn() {
            Ok(child) => {
                let pid = child.id();
                *child_guard = Some(child);

                std::thread::sleep(Duration::from_millis(1400));

                Ok(ComfyStatus {
                    running: true,
                    port,
                    pid: Some(pid),
                    health: "starting".to_string(),
                    message: format!(
                        "ComfyUI started using {} (pid {}) on port {}",
                        python_exe, pid, port
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
            let _ = child.kill();
            let _ = child.wait();
            Ok(ComfyStatus {
                running: false,
                port: self.port.lock().map(|g| *g).unwrap_or(8188),
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
        let url = format!("http://127.0.0.1:{}/", port);
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

/// Saves settings to disk and returns the saved object
#[tauri::command]
async fn save_settings(settings: AppSettings, app: tauri::AppHandle) -> Result<AppSettings, String> {
    let store = app.store("settings.dat").map_err(|e| e.to_string())?;
    settings.save(&store)?;
    Ok(settings)
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
        .add_filter("Python Interpreter", &["exe", ""])  // Windows .exe or any file on other OS
        .blocking_pick_file();
    Ok(file.map(|p| p.to_string()))
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
            pick_folder,
            pick_python_executable,
        ])
        .setup(|_app| {
            println!("[MandingoForge] Tauri app initialized. Settings + flexible ComfyUI sidecar ready.");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running MandingoForge (Tauri application)");
}
