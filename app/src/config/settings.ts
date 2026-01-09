import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  disable as disableAutostart,
  enable as enableAutostart,
  isEnabled as isAutostartEnabled,
} from "@tauri-apps/plugin-autostart";

type Config = {
  enabled: boolean;
  toast_enabled: boolean;
  status_toast_enabled: boolean;
  only_allowlisted: boolean;
  allowlist: string[];
  run_on_startup: boolean;
};

type LogEntry = {
  timestamp_ms: number;
  source_exe: string | null;
  summary: string;
};

type UiState = {
  config: Config;
  last_source_exe: string | null;
  log: LogEntry[];
  debug_last_clipboard: string | null;
  debug_last_summary: string | null;
};

const DEFAULT_ALLOWLIST = [
  "WindowsTerminal.exe",
  "wt.exe",
  "Code.exe",
  "conhost.exe",
  "pwsh.exe",
  "powershell.exe",
  "cmd.exe",
  "bash.exe",
  "wsl.exe",
];

const search = new URLSearchParams(window.location.search);
const isToast = search.get("toast") === "1" || window.__blankstopToast === true;
const isTrayMenu = window.__blankstopTrayMenu === true;

document.body.classList.toggle("toast-mode", isToast);
document.body.classList.toggle("tray-menu-mode", isTrayMenu);
document.body.classList.toggle("settings-mode", !isToast && !isTrayMenu);
document.documentElement.classList.toggle("toast-mode", isToast);
document.documentElement.classList.toggle("tray-menu-mode", isTrayMenu);
document.documentElement.classList.toggle("settings-mode", !isToast && !isTrayMenu);

if (isToast) {
  import("../ui/toast");
} else if (isTrayMenu) {
  import("../ui/tray_menu");
} else {
  initSettings();
}

function initSettings() {
  const elements = {
    enabled: document.getElementById("enabled") as HTMLInputElement,
    toast: document.getElementById("toast_enabled") as HTMLInputElement,
    statusToast: document.getElementById("status_toast_enabled") as HTMLInputElement,
    onlyAllowlisted: document.getElementById("only_allowlisted") as HTMLInputElement,
    runOnStartup: document.getElementById("run_on_startup") as HTMLInputElement,
    allowlist: document.getElementById("allowlist") as HTMLTextAreaElement,
    resetAllowlist: document.getElementById("reset-allowlist") as HTMLButtonElement,
    lastSource: document.getElementById("last-source") as HTMLSpanElement,
    lastAction: document.getElementById("last-action") as HTMLSpanElement,
  };

  let currentState: UiState | null = null;
  let applying = false;

  async function loadState() {
    const state = await invoke<UiState>("get_ui_state");
    const autostart = await isAutostartEnabled().catch(() => state.config.run_on_startup);
    state.config.run_on_startup = autostart;
    applyState(state);
    await invoke<UiState>("update_config", { config: state.config }).catch(() => null);
  }

  function applyState(state: UiState) {
    applying = true;
    currentState = state;
    elements.enabled.checked = state.config.enabled;
    elements.toast.checked = state.config.toast_enabled;
    elements.statusToast.checked = state.config.status_toast_enabled;
    elements.onlyAllowlisted.checked = state.config.only_allowlisted;
    elements.runOnStartup.checked = state.config.run_on_startup;
    elements.allowlist.value = state.config.allowlist.join("\n");
    elements.lastSource.textContent = state.last_source_exe ?? "None";

    const lastEntry = state.log[0];
    if (lastEntry) {
      elements.lastAction.textContent = new Date(lastEntry.timestamp_ms).toLocaleTimeString();
    } else {
      elements.lastAction.textContent = "Never";
    }

    applying = false;
  }

  async function saveConfig() {
    if (applying || !currentState) {
      return;
    }
    const allowlist = elements.allowlist.value
      .split("\n")
      .map((line) => line.trim())
      .filter(Boolean);
    const config: Config = {
      enabled: elements.enabled.checked,
      toast_enabled: elements.toast.checked,
      status_toast_enabled: elements.statusToast.checked,
      only_allowlisted: elements.onlyAllowlisted.checked,
      allowlist,
      run_on_startup: elements.runOnStartup.checked,
    };
    const updated = await invoke<UiState>("update_config", { config }).catch(() => null);
    if (updated) {
      applyState(updated);
    }
  }

  async function syncAutostart(checked: boolean) {
    try {
      if (checked) {
        await enableAutostart();
      } else {
        await disableAutostart();
      }
      elements.runOnStartup.checked = await isAutostartEnabled();
    } catch {
      // ignore plugin errors and fall back to config state
    }
  }

  function resetAllowlist() {
    elements.allowlist.value = DEFAULT_ALLOWLIST.join("\n");
    saveConfig();
  }

  function setupListeners() {
    elements.enabled.addEventListener("change", saveConfig);
    elements.toast.addEventListener("change", saveConfig);
    elements.statusToast.addEventListener("change", saveConfig);
    elements.onlyAllowlisted.addEventListener("change", saveConfig);
    elements.allowlist.addEventListener("change", saveConfig);
    elements.resetAllowlist.addEventListener("click", resetAllowlist);
    elements.runOnStartup.addEventListener("change", async () => {
      await syncAutostart(elements.runOnStartup.checked);
      await saveConfig();
    });

    listen<UiState>("ui-state", (event) => {
      applyState(event.payload);
    });
  }

  setupListeners();
  loadState();
}
