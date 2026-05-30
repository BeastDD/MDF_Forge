import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ArrowLeft, FolderOpen, FileCode2, Save, TestTube } from "lucide-react";

interface AppSettings {
  comfyui_path?: string | null;
  python_path?: string | null;
  default_port?: number | null;
}

interface SettingsProps {
  onBack: () => void;
}

export default function Settings({ onBack }: SettingsProps) {
  const [settings, setSettings] = useState<AppSettings>({
    comfyui_path: "",
    python_path: "",
    default_port: 8188,
  });
  const [status, setStatus] = useState("");
  const [isSaving, setIsSaving] = useState(false);

  // Load existing settings on mount
  useEffect(() => {
    loadSettings();
  }, []);

  async function loadSettings() {
    try {
      const loaded = await invoke<AppSettings>("get_settings");
      setSettings({
        comfyui_path: loaded.comfyui_path || "",
        python_path: loaded.python_path || "",
        default_port: loaded.default_port || 8188,
      });
    } catch (e) {
      setStatus("Failed to load settings: " + String(e));
    }
  }

  async function handlePickComfyUI() {
    try {
      const path = await invoke<string | null>("pick_folder");
      if (path) {
        setSettings((s) => ({ ...s, comfyui_path: path }));
        setStatus("ComfyUI folder selected");
      }
    } catch (e) {
      setStatus("Folder picker error: " + String(e));
    }
  }

  async function handlePickPython() {
    try {
      const path = await invoke<string | null>("pick_python_executable");
      if (path) {
        setSettings((s) => ({ ...s, python_path: path }));
        setStatus("Python executable selected (ideal for venvs)");
      }
    } catch (e) {
      setStatus("File picker error: " + String(e));
    }
  }

  async function handleSave() {
    setIsSaving(true);
    setStatus("Saving settings...");
    try {
      await invoke("save_settings", {
        settings: {
          comfyui_path: settings.comfyui_path || null,
          python_path: settings.python_path || null,
          default_port: settings.default_port || 8188,
        },
      });
      setStatus("Settings saved successfully ✓");
    } catch (e) {
      setStatus("Save failed: " + String(e));
    } finally {
      setIsSaving(false);
    }
  }

  async function handleTestPaths() {
    setStatus("Testing paths against current settings...");
    try {
      // We can trigger a dry validation by attempting to start (it will fail fast if paths are bad)
      // For now we just re-fetch and show what is configured
      const current = await invoke<AppSettings>("get_settings");
      const hasComfy = current.comfyui_path && current.comfyui_path.length > 3;
      const hasPy = current.python_path && current.python_path.length > 3;

      if (!hasComfy || !hasPy) {
        setStatus("Paths not fully configured. Please set both ComfyUI folder and Python executable.");
      } else {
        setStatus(`Ready. Will use:\n• ComfyUI: ${current.comfyui_path}\n• Python: ${current.python_path}`);
      }
    } catch (e) {
      setStatus("Test failed: " + String(e));
    }
  }

  return (
    <div className="min-h-screen mf-splash p-8 text-[#F5F0E6]">
      <div className="max-w-3xl mx-auto">
        {/* Header */}
        <div className="flex items-center gap-4 mb-8">
          <button
            onClick={onBack}
            className="mf-btn-secondary flex items-center gap-2 px-4 py-2"
          >
            <ArrowLeft size={18} /> Back to Splash
          </button>
          <div>
            <h1 className="text-4xl font-semibold tracking-tight text-[#C5A46E]">
              Settings
            </h1>
            <p className="text-[#A8A29E] mt-1">Configure ComfyUI location and Python environment</p>
          </div>
        </div>

        <div className="mf-card p-8 space-y-8">
          {/* ComfyUI Path */}
          <div>
            <label className="block text-sm font-medium text-[#C5A46E] mb-2">
              ComfyUI Installation Folder
            </label>
            <div className="flex gap-3">
              <input
                type="text"
                value={settings.comfyui_path || ""}
                onChange={(e) =>
                  setSettings((s) => ({ ...s, comfyui_path: e.target.value }))
                }
                placeholder="C:\AI\ComfyUI or /Users/you/AI/ComfyUI"
                className="mf-input flex-1 font-mono text-sm"
              />
              <button
                onClick={handlePickComfyUI}
                className="mf-btn-secondary flex items-center gap-2 px-5"
              >
                <FolderOpen size={18} /> Browse
              </button>
            </div>
            <p className="text-xs text-[#A8A29E] mt-1.5">
              Must contain <span className="font-mono">main.py</span>
            </p>
          </div>

          {/* Python / Venv Executable */}
          <div>
            <label className="block text-sm font-medium text-[#C5A46E] mb-2">
              Python Executable (from your chosen venv)
            </label>
            <div className="flex gap-3">
              <input
                type="text"
                value={settings.python_path || ""}
                onChange={(e) =>
                  setSettings((s) => ({ ...s, python_path: e.target.value }))
                }
                placeholder="C:\Users\you\venvs\comfy\Scripts\python.exe"
                className="mf-input flex-1 font-mono text-sm"
              />
              <button
                onClick={handlePickPython}
                className="mf-btn-secondary flex items-center gap-2 px-5"
              >
                <FileCode2 size={18} /> Choose python.exe
              </button>
            </div>
            <p className="text-xs text-[#A8A29E] mt-1.5">
              This is the key to using <strong>any virtual environment</strong>. Point it at the <span className="font-mono">python.exe</span> inside your venv folder.
            </p>
          </div>

          {/* Port */}
          <div>
            <label className="block text-sm font-medium text-[#C5A46E] mb-2">
              Default ComfyUI Port
            </label>
            <input
              type="number"
              value={settings.default_port || 8188}
              onChange={(e) =>
                setSettings((s) => ({ ...s, default_port: parseInt(e.target.value) || 8188 }))
              }
              className="mf-input w-40 font-mono"
            />
          </div>

          {/* Actions */}
          <div className="flex flex-wrap gap-4 pt-4 border-t border-[#2A2A2A]">
            <button
              onClick={handleSave}
              disabled={isSaving}
              className="mf-btn flex items-center gap-2 px-8 py-3 disabled:opacity-70"
            >
              <Save size={18} />
              {isSaving ? "SAVING..." : "SAVE SETTINGS"}
            </button>

            <button
              onClick={handleTestPaths}
              className="mf-btn-secondary flex items-center gap-2 px-6 py-3"
            >
              <TestTube size={18} /> TEST PATHS
            </button>
          </div>

          {/* Status / Feedback */}
          {status && (
            <div className="mt-4 p-4 rounded-md bg-[#121212] border border-[#2A2A2A] text-sm font-mono whitespace-pre-wrap text-[#C5A46E]">
              {status}
            </div>
          )}
        </div>

        <div className="mt-6 text-xs text-[#A8A29E] max-w-prose">
          Changes take effect the next time you launch the ComfyUI sidecar. 
          You can use completely different ComfyUI installs and virtual environments per machine or per project.
        </div>
      </div>
    </div>
  );
}
