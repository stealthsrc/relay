mod artwork;
mod bot;
mod commands;
mod config;
mod credentials;
mod model;
mod notification_widget;
mod server;
mod state;
mod tts;
mod updater;
mod widget;

use std::{env, path::PathBuf, process::Command, sync::Arc, time::Duration};

use tauri::{
    AppHandle, Manager, PhysicalPosition, Theme, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

use crate::{
    bot::start_bot,
    commands::{
        apply_config, approve_pending_media, clear_notification_sound, clear_overlay,
        clear_pending_media, control_audio, get_bootstrap, get_media_artwork, get_runtime_status,
        get_widget_bootstrap, pick_notification_sound, refresh_channels, regenerate_secret,
        reject_pending_media, replay_media, save_command_settings, save_credentials,
        set_interface_preferences, set_notification_sound_enabled,
        set_notification_sound_obs_enabled, set_notification_widget_locked,
        set_notification_widget_visible, set_output_geometry, set_widget_locked, skip_media,
        test_output, toggle_widget,
    },
    config::migrate_legacy_config,
    model::{MediaKind, ServerStatus},
    notification_widget::restore as restore_notification_widget,
    server::start_server,
    state::{AppCore, MediaDeliveryRequest},
    updater::{check_for_updates, download_and_install_update, get_app_version},
    widget::restore as restore_widget,
};

const TRAY_PANEL_LABEL: &str = "tray-panel";
const TRAY_PANEL_WIDTH: f64 = 336.0;
const TRAY_PANEL_HEIGHT: f64 = 430.0;
const TRAY_PANEL_MARGIN: i32 = 10;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main_window(app);
        }))
        .setup(|app| {
            load_dotenv();
            let config_directory = app.path().app_config_dir()?;
            migrate_legacy_config(&config_directory)?;
            let config_path = config_directory.join("config.json");
            let core = AppCore::load(config_path)?;
            app.manage(core.clone());

            register_skip_shortcut(app, core.clone())?;
            build_tray_panel(app)?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().expect("application icon").clone())
                .tooltip("Relay")
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        position,
                        button: MouseButton::Left | MouseButton::Right,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        toggle_tray_panel(tray.app_handle(), position);
                    }
                })
                .build(app)?;

            if let Some(window) = app.get_webview_window("main") {
                let window_to_hide = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = window_to_hide.hide();
                    }
                });
            }

            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let server_started = match start_server(core.clone()).await {
                    Ok(()) => true,
                    Err(error) => {
                        core.server_status.write().await.error = Some(error.to_string());
                        false
                    }
                };
                if server_started {
                    let _ = restore_widget(&app_handle, core.clone()).await;
                    let _ = restore_notification_widget(&app_handle, core.clone()).await;
                    let (media_delivery, requests) = tokio::sync::mpsc::unbounded_channel();
                    core.set_media_delivery(media_delivery).await;
                    tauri::async_runtime::spawn(deliver_media_to_local_widget(
                        app_handle.clone(),
                        core.clone(),
                        requests,
                    ));
                }
                if let Err(error) = start_bot(core.clone()).await {
                    core.bot_status.write().await.error = Some(error.to_string());
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_bootstrap,
            get_widget_bootstrap,
            get_runtime_status,
            refresh_channels,
            set_interface_preferences,
            set_output_geometry,
            save_credentials,
            apply_config,
            save_command_settings,
            clear_overlay,
            replay_media,
            skip_media,
            test_output,
            control_audio,
            get_media_artwork,
            approve_pending_media,
            reject_pending_media,
            clear_pending_media,
            regenerate_secret,
            toggle_widget,
            set_widget_locked,
            set_notification_widget_visible,
            set_notification_widget_locked,
            set_notification_sound_enabled,
            set_notification_sound_obs_enabled,
            pick_notification_sound,
            clear_notification_sound,
            tray_open_control_panel,
            tray_toggle_media_widget,
            tray_toggle_media_widget_lock,
            tray_toggle_notification_widget,
            tray_toggle_notification_widget_lock,
            tray_quit,
            set_window_theme,
            open_help_link,
            get_app_version,
            check_for_updates,
            download_and_install_update,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Relay");
}

async fn deliver_media_to_local_widget(
    app: AppHandle,
    core: Arc<AppCore>,
    mut requests: tokio::sync::mpsc::UnboundedReceiver<MediaDeliveryRequest>,
) {
    while let Some(request) = requests.recv().await {
        let should_wake = {
            let status = core.server_status.read().await;
            should_wake_media_widget(&status, request.kind)
        };
        if should_wake && widget::show(&app, core.clone(), false).await.is_ok() {
            for _ in 0..40 {
                let connected = {
                    let status = core.server_status.read().await;
                    media_widget_connected(&status)
                };
                if connected {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }
        let _ = request.ready.send(());
    }
}

fn should_wake_media_widget(status: &ServerStatus, kind: MediaKind) -> bool {
    if media_widget_connected(status) {
        return false;
    }
    match kind {
        MediaKind::Image | MediaKind::Gif | MediaKind::Video => {
            status.outputs.visual.obs_clients == 0
        }
        MediaKind::Audio => status.outputs.audio.obs_clients == 0,
    }
}

fn media_widget_connected(status: &ServerStatus) -> bool {
    status.outputs.visual.widget_clients > 0 || status.outputs.audio.widget_clients > 0
}

fn register_skip_shortcut(app: &mut tauri::App, core: Arc<AppCore>) -> tauri::Result<()> {
    use tauri_plugin_global_shortcut::{
        Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
    };

    let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyS);
    let handler_shortcut = shortcut;
    app.handle().plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |_app, pressed_shortcut, event| {
                if pressed_shortcut == &handler_shortcut && event.state() == ShortcutState::Pressed
                {
                    let _ = core.relay_tx.send(crate::model::RelayEvent::Skip);
                }
            })
            .build(),
    )?;
    let _ = app.global_shortcut().register(shortcut);
    Ok(())
}

fn build_tray_panel(app: &mut tauri::App) -> tauri::Result<()> {
    let window =
        WebviewWindowBuilder::new(app, TRAY_PANEL_LABEL, WebviewUrl::App("tray.html".into()))
            .title("Relay - Tray")
            .inner_size(TRAY_PANEL_WIDTH, TRAY_PANEL_HEIGHT)
            .resizable(false)
            .decorations(false)
            .transparent(true)
            .shadow(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .minimizable(false)
            .maximizable(false)
            .visible(false)
            .focused(false)
            .build()?;

    let window_to_hide = window.clone();
    window.on_window_event(move |event| match event {
        tauri::WindowEvent::Focused(false) => {
            let _ = window_to_hide.hide();
        }
        tauri::WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            let _ = window_to_hide.hide();
        }
        _ => {}
    });
    Ok(())
}

fn toggle_tray_panel(app: &AppHandle, click: PhysicalPosition<f64>) {
    let Some(window) = app.get_webview_window(TRAY_PANEL_LABEL) else {
        return;
    };
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
        return;
    }
    if let Ok(position) = tray_panel_position(app, click) {
        let _ = window.set_position(position);
    }
    let _ = window.eval("window.refreshTray?.()");
    let _ = window.show();
    let _ = window.set_focus();
}

fn tray_panel_position(
    app: &AppHandle,
    click: PhysicalPosition<f64>,
) -> tauri::Result<PhysicalPosition<i32>> {
    let monitors = app.available_monitors()?;
    let click_x = click.x.round() as i32;
    let click_y = click.y.round() as i32;
    let monitor = monitors
        .iter()
        .find(|monitor| {
            let area = monitor.work_area();
            click_x >= area.position.x
                && click_y >= area.position.y
                && click_x < area.position.x + area.size.width as i32
                && click_y < area.position.y + area.size.height as i32
        })
        .cloned()
        .or(app.primary_monitor()?)
        .or_else(|| monitors.into_iter().next());
    let Some(monitor) = monitor else {
        return Ok(PhysicalPosition::new(click_x, click_y));
    };

    let area = monitor.work_area();
    let scale = monitor.scale_factor();
    let width = (TRAY_PANEL_WIDTH * scale).round() as i32;
    let height = (TRAY_PANEL_HEIGHT * scale).round() as i32;
    let min_x = area.position.x + TRAY_PANEL_MARGIN;
    let max_x = (area.position.x + area.size.width as i32 - width - TRAY_PANEL_MARGIN).max(min_x);
    let min_y = area.position.y + TRAY_PANEL_MARGIN;
    let max_y = (area.position.y + area.size.height as i32 - height - TRAY_PANEL_MARGIN).max(min_y);
    let x = (click_x - width / 2).clamp(min_x, max_x);
    let above = click_y - height - TRAY_PANEL_MARGIN;
    let y = if above >= min_y {
        above.min(max_y)
    } else {
        (click_y + TRAY_PANEL_MARGIN).clamp(min_y, max_y)
    };
    Ok(PhysicalPosition::new(x, y))
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[tauri::command]
fn set_window_theme(window: WebviewWindow, theme: String) -> Result<(), String> {
    let dark = match theme.as_str() {
        "dark" => true,
        "light" => false,
        _ => return Err("Unsupported window theme.".into()),
    };
    window
        .set_theme(Some(if dark { Theme::Dark } else { Theme::Light }))
        .map_err(|error| error.to_string())?;
    apply_titlebar_theme(&window, dark).map_err(|error| error.to_string())
}

#[tauri::command]
fn open_help_link(link: String) -> Result<(), String> {
    let url = match link.as_str() {
        "discord" => "https://discord.com/developers/applications",
        "obs" => "https://obsproject.com/kb/browser-source",
        "github" => "https://github.com/stealthsrc",
        "relay-releases" => "https://github.com/stealthsrc/relay/releases/latest",
        "privacy-global" => {
            "https://unctad.org/page/data-protection-and-privacy-legislation-worldwide"
        }
        _ => return Err("Unsupported help link.".into()),
    };
    Command::new("explorer.exe")
        .arg(url)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[cfg(target_os = "windows")]
fn apply_titlebar_theme(window: &WebviewWindow, dark: bool) -> windows::core::Result<()> {
    use std::{ffi::c_void, mem::size_of};
    use windows::Win32::Graphics::Dwm::{
        DWMWA_BORDER_COLOR, DWMWA_CAPTION_COLOR, DWMWA_TEXT_COLOR, DwmSetWindowAttribute,
    };

    let hwnd = window.hwnd().map_err(|error| {
        windows::core::Error::new(
            windows::core::HRESULT(0x80004005_u32 as i32),
            error.to_string(),
        )
    })?;
    let (caption, text, border) = if dark {
        (
            colorref(0, 0, 0),
            colorref(245, 245, 241),
            colorref(0, 0, 0),
        )
    } else {
        (
            colorref(244, 244, 241),
            colorref(17, 17, 16),
            colorref(222, 222, 216),
        )
    };
    for (attribute, color) in [
        (DWMWA_CAPTION_COLOR, caption),
        (DWMWA_TEXT_COLOR, text),
        (DWMWA_BORDER_COLOR, border),
    ] {
        unsafe {
            DwmSetWindowAttribute(
                hwnd,
                attribute,
                (&color as *const u32).cast::<c_void>(),
                size_of::<u32>() as u32,
            )?;
        }
    }
    Ok(())
}

#[cfg(target_os = "windows")]
const fn colorref(red: u32, green: u32, blue: u32) -> u32 {
    red | (green << 8) | (blue << 16)
}

#[tauri::command]
fn tray_open_control_panel(app: AppHandle, window: WebviewWindow) {
    let _ = window.hide();
    show_main_window(&app);
}

#[tauri::command]
async fn tray_toggle_media_widget(
    app: AppHandle,
    core: tauri::State<'_, Arc<AppCore>>,
) -> Result<widget::WidgetState, String> {
    widget::toggle(&app, core.inner().clone())
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn tray_toggle_media_widget_lock(
    app: AppHandle,
    core: tauri::State<'_, Arc<AppCore>>,
) -> Result<widget::WidgetState, String> {
    widget::toggle_lock(&app, core.inner().clone())
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn tray_toggle_notification_widget(
    app: AppHandle,
    core: tauri::State<'_, Arc<AppCore>>,
) -> Result<notification_widget::NotificationWidgetState, String> {
    let current = notification_widget::state(&app, &core).await;
    notification_widget::set_visible(&app, core.inner().clone(), !current.visible)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn tray_toggle_notification_widget_lock(
    app: AppHandle,
    core: tauri::State<'_, Arc<AppCore>>,
) -> Result<notification_widget::NotificationWidgetState, String> {
    let current = notification_widget::state(&app, &core).await;
    notification_widget::set_locked(&app, core.inner().clone(), !current.locked)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn tray_quit(app: AppHandle) {
    app.exit(0);
}

fn load_dotenv() {
    let mut candidates = Vec::<PathBuf>::new();
    if let Ok(executable) = env::current_exe()
        && let Some(parent) = executable.parent()
    {
        candidates.push(parent.join(".env"));
    }
    if let Ok(current_directory) = env::current_dir() {
        candidates.push(current_directory.join(".env"));
    }
    for candidate in candidates {
        if candidate.is_file() {
            let _ = dotenvy::from_path(candidate);
            break;
        }
    }
}
