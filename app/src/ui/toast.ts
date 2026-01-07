import { listen } from "@tauri-apps/api/event";
import { getCurrent } from "@tauri-apps/api/window";

const toast = document.getElementById("toast-root") as HTMLDivElement;
const message = document.getElementById("message") as HTMLDivElement;
const windowHandle = getCurrent();

let hideTimer: number | undefined;
const defaultMessage = message.textContent?.trim() || "Sanitized clipboard";

const hideWindow = async () => {
  toast.classList.remove("show");
  await windowHandle.hide();
};

void hideWindow();

function showToast(text: string) {
  const payload = text.trim().length > 0 ? text : defaultMessage;
  message.textContent = payload;
  toast.classList.add("show");
  void windowHandle.show();
  if (hideTimer) {
    window.clearTimeout(hideTimer);
  }
  hideTimer = window.setTimeout(async () => {
    await hideWindow();
  }, 1200);
}

listen<string>("toast-message", (event) => {
  showToast(event.payload);
});
