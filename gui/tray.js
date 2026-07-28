const invoke = window.__TAURI__.core.invoke;

const trayTranslations = {
  en: { relayStatus: "Relay status", connectionStatus: "Connection status", discord: "Discord", localRelay: "Local relay", system: "System", openPanel: "Open control panel", displayWidgets: "Display widgets", mediaOverlay: "Media overlay", ttsNotifications: "TTS notifications", runningLocally: "Running locally", startWithWindows: "Start with Windows", quitRelay: "Quit Relay", checking: "Checking", connecting: "Connecting…", zeroOutputs: "0 outputs", online: "Online", offline: "Offline", waitingDiscord: "Waiting for Discord", output: "output", outputs: "outputs", hidden: "Hidden", visibleLocked: "Visible · Locked", visibleMovable: "Visible · Movable", hide: "Hide", show: "Show", lock: "Lock", unlock: "Unlock", mediaWidget: "media widget", notificationWidget: "notification widget", unavailable: "Unavailable", notResponding: "Relay not responding" },
  fr: { relayStatus: "Statut de Relay", connectionStatus: "État de la connexion", discord: "Discord", localRelay: "Relais local", system: "Système", openPanel: "Ouvrir le panneau de contrôle", displayWidgets: "Widgets d’affichage", mediaOverlay: "Overlay média", ttsNotifications: "Notifications TTS", runningLocally: "Exécution locale", startWithWindows: "Démarrer avec Windows", quitRelay: "Quitter Relay", checking: "Vérification", connecting: "Connexion…", zeroOutputs: "0 sortie", online: "En ligne", offline: "Hors ligne", waitingDiscord: "En attente de Discord", output: "sortie", outputs: "sorties", hidden: "Masqué", visibleLocked: "Visible · Verrouillé", visibleMovable: "Visible · Déplaçable", hide: "Masquer", show: "Afficher", lock: "Verrouiller", unlock: "Déverrouiller", mediaWidget: "le widget média", notificationWidget: "le widget de notifications", unavailable: "Indisponible", notResponding: "Relay ne répond pas" },
  es: { relayStatus: "Estado de Relay", connectionStatus: "Estado de la conexión", discord: "Discord", localRelay: "Relay local", system: "Sistema", openPanel: "Abrir el panel de control", displayWidgets: "Widgets de pantalla", mediaOverlay: "Overlay multimedia", ttsNotifications: "Notificaciones TTS", runningLocally: "Ejecución local", startWithWindows: "Iniciar con Windows", quitRelay: "Salir de Relay", checking: "Comprobando", connecting: "Conectando…", zeroOutputs: "0 salidas", online: "En línea", offline: "Sin conexión", waitingDiscord: "Esperando a Discord", output: "salida", outputs: "salidas", hidden: "Oculto", visibleLocked: "Visible · Bloqueado", visibleMovable: "Visible · Desplazable", hide: "Ocultar", show: "Mostrar", lock: "Bloquear", unlock: "Desbloquear", mediaWidget: "widget multimedia", notificationWidget: "widget de notificaciones", unavailable: "No disponible", notResponding: "Relay no responde" },
  de: { relayStatus: "Relay-Status", connectionStatus: "Verbindungsstatus", discord: "Discord", localRelay: "Lokales Relay", system: "System", openPanel: "Bedienfeld öffnen", displayWidgets: "Anzeige-Widgets", mediaOverlay: "Medien-Overlay", ttsNotifications: "TTS-Benachrichtigungen", runningLocally: "Läuft lokal", startWithWindows: "Mit Windows starten", quitRelay: "Relay beenden", checking: "Wird geprüft", connecting: "Verbindung…", zeroOutputs: "0 Ausgaben", online: "Online", offline: "Offline", waitingDiscord: "Warten auf Discord", output: "Ausgabe", outputs: "Ausgaben", hidden: "Ausgeblendet", visibleLocked: "Sichtbar · Gesperrt", visibleMovable: "Sichtbar · Verschiebbar", hide: "Ausblenden", show: "Anzeigen", lock: "Sperren", unlock: "Entsperren", mediaWidget: "Medien-Widget", notificationWidget: "Benachrichtigungs-Widget", unavailable: "Nicht verfügbar", notResponding: "Relay antwortet nicht" },
};

let language = "en";

function translate(key) {
  return trayTranslations[language]?.[key] || trayTranslations.en[key] || key;
}

function applyTrayLanguage() {
  const storedLanguage = localStorage.getItem("relay-language");
  const storedDesign = localStorage.getItem("relay-design");
  const storedTheme = localStorage.getItem("relay-theme");
  language = Object.hasOwn(trayTranslations, storedLanguage) ? storedLanguage : "en";
  document.documentElement.lang = language;
  document.documentElement.dataset.design = ["anthropic", "neo-brutalism"].includes(storedDesign)
    ? storedDesign
    : "openai";
  document.documentElement.dataset.theme = storedTheme === "light" ? "light" : "dark";
  for (const element of document.querySelectorAll("[data-i18n]")) {
    element.textContent = translate(element.dataset.i18n);
  }
  for (const element of document.querySelectorAll("[data-i18n-aria]")) {
    element.setAttribute("aria-label", translate(element.dataset.i18nAria));
  }
}

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
  startup: document.querySelector("#toggle-startup"),
  quit: document.querySelector("#quit"),
};

function renderWidget(state, stateElement, toggleButton, lockButton, name) {
  stateElement.textContent = state.visible
    ? translate(state.locked ? "visibleLocked" : "visibleMovable")
    : translate("hidden");
  toggleButton.textContent = translate(state.visible ? "hide" : "show");
  toggleButton.setAttribute("aria-pressed", String(state.visible));
  lockButton.classList.toggle("is-unlocked", !state.locked);
  lockButton.setAttribute("aria-label", `${translate(state.locked ? "unlock" : "lock")} ${translate(name)}`);
  lockButton.setAttribute("aria-pressed", String(state.locked));
}

function render(status) {
  const relayOnline = status.bot.connected && status.server.connected;
  elements.indicator.classList.toggle("is-online", relayOnline);
  elements.indicator.classList.toggle("is-error", !status.server.connected || Boolean(status.bot.error));

  elements.discordStatus.textContent = translate(status.bot.connected ? "online" : "offline");
  elements.discordDetail.textContent = status.bot.username || status.bot.error || translate("waitingDiscord");
  elements.serverStatus.textContent = translate(status.server.connected ? "online" : "offline");
  elements.serverDetail.textContent = `${status.server.overlayClients} ${translate(status.server.overlayClients === 1 ? "output" : "outputs")}`;

  renderWidget(status.widget, elements.mediaState, elements.toggleMedia, elements.lockMedia, "mediaWidget");
  renderWidget(
    status.notificationWidget,
    elements.notificationState,
    elements.toggleNotification,
    elements.lockNotification,
    "notificationWidget",
  );
}

async function refreshTray() {
  applyTrayLanguage();
  try {
    const [status, startupEnabled] = await Promise.all([
      invoke("get_runtime_status"),
      invoke("get_start_with_windows").catch(() => false),
    ]);
    render(status);
    elements.startup.setAttribute("aria-pressed", String(startupEnabled));
    elements.startup.textContent = `${startupEnabled ? "✓ " : ""}${translate("startWithWindows")}`;
  } catch {
    elements.indicator.classList.remove("is-online");
    elements.indicator.classList.add("is-error");
    elements.serverStatus.textContent = translate("unavailable");
    elements.serverDetail.textContent = translate("notResponding");
  }
}

async function runAction(button, command, payload) {
  button.disabled = true;
  try {
    await invoke(command, payload);
    await refreshTray();
  } catch {
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
elements.startup.addEventListener("click", () => {
  const enabled = elements.startup.getAttribute("aria-pressed") !== "true";
  return runAction(elements.startup, "set_start_with_windows", { enabled });
});
elements.quit.addEventListener("click", () => invoke("tray_quit"));

window.refreshTray = refreshTray;
window.addEventListener("focus", refreshTray);
document.addEventListener("visibilitychange", () => {
  if (!document.hidden) refreshTray();
});

refreshTray();
