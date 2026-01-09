import { listen } from "@tauri-apps/api/event";

const toast = document.getElementById("toast-root");
const message = document.getElementById("message");

let hideTimer: number | undefined;
const defaultMessage = message?.textContent?.trim() || "Sanitized clipboard";
const SHOW_DURATION = 1200;
const EXIT_DURATION = 150;

function showToast(text: string) {
  if (!(toast instanceof HTMLDivElement) || !(message instanceof HTMLDivElement)) {
    return;
  }

  const payload = text.trim().length > 0 ? text : defaultMessage;
  message.textContent = payload;

  // Clear any pending hide
  if (hideTimer) {
    window.clearTimeout(hideTimer);
  }

  // Remove hiding class if present, show immediately
  toast.classList.remove("hiding");
  toast.classList.add("show");

  // Schedule hide with smooth exit
  hideTimer = window.setTimeout(() => {
    toast.classList.add("hiding");
    toast.classList.remove("show");

    // Clean up hiding class after transition
    window.setTimeout(() => {
      toast.classList.remove("hiding");
    }, EXIT_DURATION);
  }, SHOW_DURATION);
}

listen<string>("toast-message", (event) => {
  showToast(event.payload);
});
