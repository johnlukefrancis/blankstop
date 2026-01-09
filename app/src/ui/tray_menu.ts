import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

type TrayMenuState = {
  enabled: boolean;
  paused: boolean;
};

const elements = {
  enabled: document.getElementById("menu-enabled") as HTMLInputElement,
  pause: document.getElementById("menu-pause") as HTMLButtonElement,
  settings: document.getElementById("menu-settings") as HTMLButtonElement,
  sanitize: document.getElementById("menu-sanitize") as HTMLButtonElement,
  quit: document.getElementById("menu-quit") as HTMLButtonElement,
};

function applyState(state: TrayMenuState) {
  elements.enabled.checked = state.enabled;
  elements.pause.textContent = state.paused ? "Paused" : "Pause 5 min";
  elements.pause.classList.toggle("paused", state.paused);
}

async function handleAction(action: string) {
  await invoke("tray_menu_action", { action });
}

async function hideMenu() {
  await invoke("hide_tray_menu");
}

async function init() {
  // Load initial state
  const state = await invoke<TrayMenuState>("get_tray_menu_state");
  applyState(state);

  // Listen for state updates
  listen<TrayMenuState>("tray-menu-state", (event) => {
    applyState(event.payload);
  });

  // Wire up click handlers
  elements.enabled.addEventListener("change", () => handleAction("enabled"));
  elements.pause.addEventListener("click", () => handleAction("pause_5"));
  elements.settings.addEventListener("click", () => handleAction("settings"));
  elements.sanitize.addEventListener("click", () => handleAction("sanitize_now"));
  elements.quit.addEventListener("click", () => handleAction("quit"));

  // Close on blur (click outside window)
  const currentWindow = getCurrentWindow();
  currentWindow.onFocusChanged(({ payload: focused }) => {
    if (!focused) {
      hideMenu();
    }
  });
}

init();
