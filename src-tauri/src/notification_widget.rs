use std::sync::{Arc, atomic::Ordering};

use anyhow::{Context, Result};
use serde::Serialize;
use tauri::{
    AppHandle, Manager, PhysicalPosition, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};

use crate::{credentials::load_or_create_relay_secret, state::AppCore};

const WINDOW_LABEL: &str = "notification-widget";
const WINDOW_WIDTH: f64 = 510.0;
const WINDOW_HEIGHT: f64 = 130.0;
const DEFAULT_MARGIN: i32 = 28;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationWidgetState {
    pub visible: bool,
    pub locked: bool,
}

pub async fn state(app: &AppHandle, core: &Arc<AppCore>) -> NotificationWidgetState {
    let config = core.config.read().await;
    let visible = app
        .get_webview_window(WINDOW_LABEL)
        .and_then(|window| window.is_visible().ok())
        .unwrap_or(config.notification_widget_visible);
    NotificationWidgetState {
        visible,
        locked: config.notification_widget_locked,
    }
}

pub async fn restore(app: &AppHandle, core: Arc<AppCore>) -> Result<()> {
    if core.config.read().await.notification_widget_visible {
        let window = ensure_window(app, core.clone()).await?;
        window.show()?;
    }
    Ok(())
}

pub async fn set_visible(
    app: &AppHandle,
    core: Arc<AppCore>,
    visible: bool,
) -> Result<NotificationWidgetState> {
    if visible {
        let window = ensure_window(app, core.clone()).await?;
        window.show()?;
        if !core.config.read().await.notification_widget_locked {
            let _ = window.set_focus();
        }
    } else if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
        window.hide()?;
    }
    update_visibility(&core, visible).await?;
    Ok(state(app, &core).await)
}

pub async fn set_locked(
    app: &AppHandle,
    core: Arc<AppCore>,
    locked: bool,
) -> Result<NotificationWidgetState> {
    let mut config = core.config.read().await.clone();
    config.notification_widget_locked = locked;
    core.set_config(config).await?;
    if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
        apply_lock(&window, locked)?;
        window.navigate(widget_url(&core).await?.parse()?)?;
    }
    Ok(state(app, &core).await)
}

pub async fn refresh(app: &AppHandle, core: &Arc<AppCore>) -> Result<()> {
    if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
        window.navigate(widget_url(core).await?.parse()?)?;
    }
    Ok(())
}

async fn ensure_window(app: &AppHandle, core: Arc<AppCore>) -> Result<WebviewWindow> {
    if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
        return Ok(window);
    }

    let config = core.config.read().await.clone();
    let url = widget_url(&core).await?;
    let window = WebviewWindowBuilder::new(app, WINDOW_LABEL, WebviewUrl::External(url.parse()?))
        .title("Relay - TTS notifications")
        .inner_size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .minimizable(false)
        .maximizable(false)
        .focused(false)
        .visible(false)
        .build()?;

    let position = resolved_position(
        app,
        config
            .notification_widget_x
            .zip(config.notification_widget_y),
    )?;
    window.set_position(position)?;
    apply_lock(&window, config.notification_widget_locked)?;
    watch_position(&window, core);
    Ok(window)
}

fn apply_lock(window: &WebviewWindow, locked: bool) -> Result<()> {
    window.set_focusable(!locked)?;
    window.set_ignore_cursor_events(locked)?;
    window.eval(format!(
        "window.setWidgetLocked?.({})",
        if locked { "true" } else { "false" }
    ))?;
    Ok(())
}

fn watch_position(window: &WebviewWindow, core: Arc<AppCore>) {
    window.on_window_event(move |event| {
        let tauri::WindowEvent::Moved(position) = event else {
            return;
        };
        let position = *position;
        let generation = core
            .notification_widget_move_generation
            .fetch_add(1, Ordering::Relaxed)
            + 1;
        let core = core.clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(400)).await;
            if core
                .notification_widget_move_generation
                .load(Ordering::Relaxed)
                != generation
            {
                return;
            }
            let mut config = core.config.read().await.clone();
            config.notification_widget_x = Some(position.x);
            config.notification_widget_y = Some(position.y);
            let _ = core.set_config(config).await;
        });
    });
}

fn resolved_position(app: &AppHandle, saved: Option<(i32, i32)>) -> Result<PhysicalPosition<i32>> {
    let monitors = app.available_monitors()?;
    if let Some((x, y)) = saved
        && monitors.iter().any(|monitor| {
            let area = monitor.work_area();
            x >= area.position.x
                && y >= area.position.y
                && x < area.position.x + area.size.width as i32
                && y < area.position.y + area.size.height as i32
        })
    {
        return Ok(PhysicalPosition::new(x, y));
    }

    let monitor = app
        .primary_monitor()?
        .or_else(|| monitors.into_iter().next())
        .context("no display is available")?;
    let area = monitor.work_area();
    let scale = monitor.scale_factor();
    let physical_width = (WINDOW_WIDTH * scale).round() as i32;
    Ok(PhysicalPosition::new(
        area.position.x + area.size.width as i32 - physical_width - DEFAULT_MARGIN,
        area.position.y + DEFAULT_MARGIN,
    ))
}

async fn update_visibility(core: &Arc<AppCore>, visible: bool) -> Result<()> {
    let mut config = core.config.read().await.clone();
    config.notification_widget_visible = visible;
    core.set_config(config).await
}

async fn widget_url(core: &Arc<AppCore>) -> Result<String> {
    let config = core.config.read().await.clone();
    let secret = load_or_create_relay_secret()?;
    let preferences = core.interface_preferences.read().await;
    Ok(format!(
        "http://127.0.0.1:{}/notifications?secret={secret}&target=widget&locked={}&lang={}",
        config.port,
        if config.notification_widget_locked {
            1
        } else {
            0
        },
        preferences.language,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notification_widget_uses_compact_console_dimensions() {
        assert_eq!((WINDOW_WIDTH, WINDOW_HEIGHT), (510.0, 130.0));
    }
}
