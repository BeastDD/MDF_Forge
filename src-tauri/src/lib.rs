// MandingoForge - Tauri v2 + ComfyUI Sidecar Management (Sprint 0)
// Professional local AI studio foundation

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::State;

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
    comfyui_dir: PathBuf,
}

impl ComfyManager {
    pub fn new() -> Self {
        // Default location per Sprint 0 roadmap structure
        let comfyui_dir = std::env::current_dir()
            .unwrap_or_default()
            .join("comfyui");

        Self {
            child: Arc::new(Mutex::new(None)),
            port: Arc::new(Mutex::new(8188)),
            comfyui_dir,
        }
    }

    pub fn start(&self, requested_port: Option<u16>) -> Result<ComfyStatus, String> {
        let mut child_guard = self.child.lock().map_err(|e| e.to_string())?;
        
        if child_guard.is_some() {
            return Err("ComfyUI sidecar is already running".to_string());
        }

        let port = requested_port.unwrap_or(8188);
        *self.port.lock().map_err(|e| e.to_string())? = port;

        // Verify comfyui directory exists
        let main_py = self.comfyui_dir.join("main.py");
        if !main_py.exists() {
            return Err(format!(
                "ComfyUI not found at {}. Run: git clone https://github.com/comfyanonymous/ComfyUI.git comfyui (from project root)",
                self.comfyui_dir.display()
            ));
        }

        // Try common python executables (Windows friendly)
        let python_candidates = ["python", "python3", "py"];
        let mut last_error = String::new();

        for py in python_candidates {
            let mut cmd = Command::new(py);
            cmd.current_dir(&self.comfyui_dir)
                .arg("-u")
                .arg("main.py")
                .arg("--listen")
                .arg("127.0.0.1")
                .arg("--port")
                .arg(port.to_string())
                // Critical: do not inherit stdio in a way that blocks, but allow logging
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            // On Windows, prevent extra console window for the python process in release
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
            }

            match cmd.spawn() {
                Ok(child) => {
                    let pid = child.id();
                    *child_guard = Some(child);

                    // Give ComfyUI a moment to boot
                    std::thread::sleep(Duration::from_millis(1200));

                    return Ok(ComfyStatus {
                        running: true,
                        port,
                        pid: Some(pid),
                        health: "starting".to_string(),
                        message: format!("ComfyUI sidecar started (pid {}) on port {}", pid, port),
                    });
                }
                Err(e) => {
                    last_error = format!("Failed with '{}': {}", py, e);
                    continue;
                }
            }
        }

        Err(format!(
            "Could not spawn ComfyUI. Tried python/python3/py. Last error: {}. Is Python installed and in PATH?",
            last_error
        ))
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

#[tauri::command]
async fn start_comfyui(
    port: Option<u16>,
    state: State<'_, ComfyManager>,
) -> Result<ComfyStatus, String> {
    state.start(port)
}

#[tauri::command]
async fn stop_comfyui(state: State<'_, ComfyManager>) -> Result<ComfyStatus, String> {
    state.stop()
}

#[tauri::command]
async fn get_comfyui_status(state: State<'_, ComfyManager>) -> Result<ComfyStatus, String> {
    state.health().await
}

// Simple config read (expand in later sprints)
#[tauri::command]
fn get_app_config() -> serde_json::Value {
    serde_json::json!({
        "app": "MandingoForge",
        "version": "0.1.0-sprint0",
        "comfyui_default_port": 8188,
        "theme": "luxury-dark"
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize simple logger for development visibility
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).try_init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(ComfyManager::new())
        .invoke_handler(tauri::generate_handler![
            greet,
            start_comfyui,
            stop_comfyui,
            get_comfyui_status,
            get_app_config
        ])
        .setup(|_app| {
            // Optional: log startup
            println!("[MandingoForge] Tauri app initialized. Sidecar manager ready.");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running MandingoForge (Tauri application)");
}
