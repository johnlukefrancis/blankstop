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

const search = new URLSearchParams(window.location.search);
const isToast = search.get("toast") === "1" || window.__blankstopToast === true;

document.body.classList.toggle("toast-mode", isToast);
document.body.classList.toggle("settings-mode", !isToast);

if (isToast) {
  import("../ui/toast");
} else {
  initSettings();
}

function initSettings() {
  const elements = {
    enabled: document.getElementById("enabled") as HTMLInputElement,
    toast: document.getElementById("toast_enabled") as HTMLInputElement,
    onlyAllowlisted: document.getElementById("only_allowlisted") as HTMLInputElement,
    runOnStartup: document.getElementById("run_on_startup") as HTMLInputElement,
    allowlist: document.getElementById("allowlist") as HTMLTextAreaElement,
    status: document.getElementById("status-pill") as HTMLDivElement,
    lastSource: document.getElementById("last-source") as HTMLSpanElement,
    log: document.getElementById("log") as HTMLDivElement,
    debugSummary: document.getElementById("debug-summary") as HTMLPreElement,
    debugClipboard: document.getElementById("debug-clipboard") as HTMLPreElement,
    debugClipboardEscaped: document.getElementById(
      "debug-clipboard-escaped",
    ) as HTMLPreElement,
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
    elements.onlyAllowlisted.checked = state.config.only_allowlisted;
    elements.runOnStartup.checked = state.config.run_on_startup;
    elements.allowlist.value = state.config.allowlist.join("\n");
    elements.status.textContent = state.config.enabled ? "Enabled" : "Disabled";
    elements.status.classList.toggle("disabled", !state.config.enabled);
    elements.lastSource.textContent = state.last_source_exe ?? "None";
    elements.debugSummary.textContent = state.debug_last_summary ?? "None";
    const lastClipboard = state.debug_last_clipboard;
    elements.debugClipboard.textContent = lastClipboard ?? "None";
    elements.debugClipboardEscaped.textContent = lastClipboard
      ? toVisibleDebug(lastClipboard)
      : "None";
    renderLog(state.log);
    applying = false;
  }

  function renderLog(entries: LogEntry[]) {
    elements.log.innerHTML = "";
    if (!entries.length) {
      elements.log.innerHTML = `<div class="log-entry"><span>No sanitized events yet.</span></div>`;
      return;
    }
    for (const entry of entries) {
      const time = new Date(entry.timestamp_ms).toLocaleTimeString();
      const source = entry.source_exe ? ` from ${entry.source_exe}` : "";
      const item = document.createElement("div");
      item.className = "log-entry";
      item.innerHTML = `<strong>${entry.summary}</strong><span>${time}${source}</span>`;
      elements.log.appendChild(item);
    }
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

  function setupListeners() {
    elements.enabled.addEventListener("change", saveConfig);
    elements.toast.addEventListener("change", saveConfig);
    elements.onlyAllowlisted.addEventListener("change", saveConfig);
    elements.allowlist.addEventListener("change", saveConfig);
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

function toVisibleDebug(input: string): string {
  let out = "";
  for (const ch of input) {
    if (ch === "\r") {
      out += "\\r";
      continue;
    }
    if (ch === "\n") {
      out += "\\n\n";
      continue;
    }
    if (isInvisibleFormatChar(ch)) {
      const code = ch.codePointAt(0);
      if (code !== undefined) {
        out += `\\u{${code.toString(16).toUpperCase().padStart(4, "0")}}`;
      }
      continue;
    }
    out += ch;
  }
  return out;
}

function isInvisibleFormatChar(ch: string): boolean {
  switch (ch) {
    case "\u200B":
    case "\u200C":
    case "\u200D":
    case "\u2060":
    case "\uFEFF":
    case "\u00AD":
      return true;
    default:
      return false;
  }
}
