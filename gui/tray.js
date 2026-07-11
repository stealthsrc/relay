const invoke = window.__TAURI__.core.invoke;

const elements = {
  indicator: document.querySelector("#relay-indicator"),
  discordStatus: document.querySelector("#discord-status"),
  discordDetail: document.querySelector("#discord-detail"),
  serverStatus: document.querySelector("#server-status"),
  serverDetail: document.querySelector("#server-detail"),
  mediaState: document.querySelector("#media-widget-state"),
  notificationState: document.querySelector("#notification-widget-state"),
  toggleMedia: document.querySelector("#toggle-media-widget"),
  lockMedia: document.querySelector("#lock-media-widget"),
  toggleNotification: document.querySelector("#toggle-notification-widget"),
  lockNotification: document.querySelector("#lock-notification-widget"),
  openPanel: document.querySelector("#open-panel"),
  quit: document.querySelector("#quit"),
};

function renderWidget(state, stateElement, toggleButton, lockButton, name) {
  stateElement.textContent = state.visible
    ? state.locked ? "Visible · Locked" : "Visible · Movable"
    : "Hidden";
  toggleButton.textContent = state.visible ? "Hide" : "Show";
  toggleButton.setAttribute("aria-pressed", String(state.visible));
  lockButton.classList.toggle("is-unlocked", !state.locked);
  lockButton.setAttribute("aria-label", `${state.locked ? "Unlock" : "Lock"} ${name}`);
  lockButton.setAttribute("aria-pressed", String(state.locked));
}

function render(status) {
  const relayOnline = status.bot.connected && status.server.connected;
  elements.indicator.classList.toggle("is-online", relayOnline);
  elements.indicator.classList.toggle("is-error", !status.server.connected || Boolean(status.bot.error));

  elements.discordStatus.textContent = status.bot.connected ? "Online" : "Offline";
  elements.discordDetail.textContent = status.bot.username || status.bot.error || "Waiting for Discord";
  elements.serverStatus.textContent = status.server.connected ? "Online" : "Offline";
  elements.serverDetail.textContent = `${status.server.overlayClients} output${status.server.overlayClients === 1 ? "" : "s"}`;

  renderWidget(status.widget, elements.mediaState, elements.toggleMedia, elements.lockMedia, "media widget");
  renderWidget(
    status.notificationWidget,
    elements.notificationState,
    elements.toggleNotification,
    elements.lockNotification,
    "notification widget",
  );
}

async function refreshTray() {
  try {
    render(await invoke("get_runtime_status"));
  } catch {
    elements.indicator.classList.remove("is-online");
    elements.indicator.classList.add("is-error");
    elements.serverStatus.textContent = "Unavailable";
    elements.serverDetail.textContent = "Relay not responding";
  }
}

async function runAction(button, command) {
  button.disabled = true;
  try {
    await invoke(command);
    await refreshTray();
  } finally {
    button.disabled = false;
  }
}

elements.openPanel.addEventListener("click", () => invoke("tray_open_control_panel"));
elements.toggleMedia.addEventListener("click", () => runAction(elements.toggleMedia, "tray_toggle_media_widget"));
elements.lockMedia.addEventListener("click", () => runAction(elements.lockMedia, "tray_toggle_media_widget_lock"));
elements.toggleNotification.addEventListener("click", () => runAction(elements.toggleNotification, "tray_toggle_notification_widget"));
elements.lockNotification.addEventListener("click", () => runAction(elements.lockNotification, "tray_toggle_notification_widget_lock"));
elements.quit.addEventListener("click", () => invoke("tray_quit"));

window.refreshTray = refreshTray;
window.addEventListener("focus", refreshTray);
document.addEventListener("visibilitychange", () => {
  if (!document.hidden) refreshTray();
});

refreshTray();
