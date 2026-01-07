import { listen } from "@tauri-apps/api/event";

const toast = document.getElementById("toast-root") as HTMLDivElement;
const message = document.getElementById("message") as HTMLDivElement;

let hideTimer: number | undefined;
const defaultMessage = message.textContent?.trim() || "Sanitized clipboard";

function showToast(text: string) {
  const payload = text.trim().length > 0 ? text : defaultMessage;
  message.textContent = payload;
  toast.classList.add("show");
  if (hideTimer) {
    window.clearTimeout(hideTimer);
  }
  hideTimer = window.setTimeout(async () => {
    toast.classList.remove("show");
  }, 1200);
}

listen<string>("toast-message", (event) => {
  showToast(event.payload);
});
