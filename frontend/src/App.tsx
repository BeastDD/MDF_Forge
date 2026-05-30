import { useState } from "react";
import { motion } from "framer-motion";
import { Play, Settings as SettingsIcon, Image, Video, Users } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import Settings from "./Settings";

type View = "splash" | "settings";

function App() {
  const [view, setView] = useState<View>("splash");
  const [status, setStatus] = useState<"idle" | "starting" | "running" | "error">("idle");
  const [statusMessage, setStatusMessage] = useState("Ready to forge");

  // Basic greet kept for IPC validation during dev
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    try {
      const msg = await invoke<string>("greet", { name: name || "Producer" });
      setGreetMsg(msg);
    } catch (e) {
      setGreetMsg("IPC error: " + String(e));
    }
  }

  // Launch sidecar — now respects whatever the user saved in Settings
  async function launchComfyUISidecar() {
    setStatus("starting");
    setStatusMessage("Initializing ComfyUI sidecar (using your saved paths)...");
    try {
      const result = await invoke<any>("start_comfyui", { port: null });
      setStatus("running");
      setStatusMessage(result?.message || "ComfyUI sidecar active");
    } catch (e) {
      setStatus("error");
      setStatusMessage("Failed: " + String(e));
    }
  }

  async function refreshStatus() {
    try {
      const s = await invoke<any>("get_comfyui_status");
      setStatus(s?.running ? "running" : "idle");
      setStatusMessage(s?.message || "Status refreshed");
    } catch (e) {
      setStatusMessage("Status check failed: " + String(e));
    }
  }

  // Simple view switcher (we'll evolve this into proper routing later)
  if (view === "settings") {
    return <Settings onBack={() => setView("splash")} />;
  }

  return (
    <div className="min-h-screen mf-splash flex flex-col">
      {/* Top bar - premium minimal */}
      <div className="h-14 border-b border-[#2A2A2A] flex items-center justify-between px-8">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-full bg-gradient-to-br from-[#C5A46E] to-[#8B0000] flex items-center justify-center">
            <span className="text-[#0A0A0A] text-sm font-bold tracking-[2px]">MF</span>
          </div>
          <div>
            <span className="font-semibold tracking-[3px] text-[#C5A46E] text-lg">MANDINGOFORGE</span>
          </div>
        </div>
        <div className="flex items-center gap-6 text-sm text-[#A8A29E]">
          <div className="flex items-center gap-2">
            <span className={`status-dot ${status === 'running' ? 'green' : status === 'error' ? 'red' : 'gold'}`} />
            <span>{statusMessage}</span>
          </div>
          <button 
            onClick={() => setView("settings")} 
            className="mf-btn-secondary text-xs px-3 py-1 flex items-center gap-1"
          >
            <SettingsIcon size={14} /> SETTINGS
          </button>
        </div>
      </div>

      {/* Hero / Splash Content */}
      <div className="flex-1 flex flex-col items-center justify-center px-8 text-center">
        <motion.div
          initial={{ opacity: 0, y: 40 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, ease: [0.22, 1, 0.36, 1] }}
        >
          <div className="mb-6">
            <div className="inline-block px-4 py-1 rounded-full border border-[#C5A46E]/30 text-[#C5A46E] text-xs tracking-[4px] mb-4">
              PROFESSIONAL • LOCAL • UNCENSORED
            </div>
          </div>

          <h1 className="text-[72px] leading-[1.05] font-semibold tracking-[-3.5px] text-[#F5F0E6] mb-4">
            MANDINGOFORGE
          </h1>
          
          <p className="max-w-[620px] mx-auto text-2xl text-[#C5A46E] font-light tracking-[-0.5px] mb-3">
            The definitive AI studio for gay interracial, BBC, raceplay &amp; Mandingo art.
          </p>
          
          <p className="max-w-md mx-auto text-[#A8A29E] text-lg mb-12">
            Built like ComfyUI + InvokeAI, but every pixel speaks the language of the niche.
          </p>
        </motion.div>

        {/* Primary Actions */}
        <motion.div 
          className="flex flex-wrap gap-4 justify-center"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.35, duration: 0.6 }}
        >
          <button 
            onClick={launchComfyUISidecar}
            disabled={status === "starting" || status === "running"}
            className="mf-btn flex items-center gap-3 text-base px-10 py-4 disabled:opacity-60"
          >
            <Play size={20} />
            {status === "running" ? "SIDE CAR RUNNING" : "LAUNCH QUICK FORGE"}
          </button>

          <button className="mf-btn-secondary flex items-center gap-3 text-base px-8 py-4">
            <Image size={18} /> BROWSE GALLERY
          </button>

          <button onClick={refreshStatus} className="mf-btn-secondary flex items-center gap-2 text-sm px-5 py-4">
            REFRESH STATUS
          </button>
        </motion.div>

        {/* Mode Pills - preview of future navigation */}
        <motion.div 
          className="mt-16 flex flex-wrap gap-3 justify-center"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 0.6 }}
        >
          {[
            { icon: <Play size={15} />, label: "Quick Forge" },
            { icon: <Video size={15} />, label: "Video Lab" },
            { icon: <Users size={15} />, label: "Archetypes" },
            { icon: <SettingsIcon size={15} />, label: "Model Manager" },
          ].map((mode, i) => (
            <div 
              key={i}
              className="px-5 py-2 rounded-full border border-[#2A2A2A] bg-[#121212] text-sm text-[#A8A29E] flex items-center gap-2 hover:border-[#C5A46E]/40 hover:text-[#C5A46E] transition-colors cursor-pointer"
            >
              {mode.icon} {mode.label}
            </div>
          ))}
        </motion.div>
      </div>

      {/* Bottom status bar */}
      <div className="h-12 border-t border-[#2A2A2A] flex items-center px-8 text-xs text-[#A8A29E] justify-between">
        <div>
          Sprint 0 + Flexible Sidecar • Any ComfyUI + Any Venv (see Settings)
        </div>
        <div className="flex items-center gap-4">
          <span>100% Local • Private • Uncensored</span>
          {/* Dev helper - remove in later sprints */}
          <div className="flex items-center gap-2 border-l border-[#2A2A2A] pl-4">
            <input
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Test IPC name"
              className="mf-input text-xs py-1 px-3 w-32"
            />
            <button onClick={greet} className="text-[#C5A46E] hover:underline">Test Rust</button>
            {greetMsg && <span className="text-[#C5A46E] ml-2">{greetMsg}</span>}
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
