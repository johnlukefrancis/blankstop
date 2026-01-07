const search = new URLSearchParams(window.location.search);
const isToast = search.get("toast") === "1";

document.body.classList.toggle("toast-mode", isToast);
document.body.classList.toggle("settings-mode", !isToast);

if (isToast) {
  import("./toast");
} else {
  import("./settings");
}
