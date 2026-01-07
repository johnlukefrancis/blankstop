import { listen } from "@tauri-apps/api/event";

const toast = document.getElementById("toast") as HTMLDivElement;
const message = document.getElementById("message") as HTMLDivElement;

let hideTimer: number | undefined;

function showToast(text: string) {
  message.textContent = text;
  toast.classList.add("show");
  if (hideTimer) {
    window.clearTimeout(hideTimer);
  }
  hideTimer = window.setTimeout(() => {
    toast.classList.remove("show");
  }, 1200);
}

listen<string>("toast-message", (event) => {
  showToast(event.payload);
});
