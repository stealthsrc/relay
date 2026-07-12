use std::sync::{Arc, atomic::Ordering};

use anyhow::{Context, Result};
use serde::Serialize;
use tauri::{
    AppHandle, Manager, PhysicalPosition, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};

use crate::{credentials::load_or_create_relay_secret, state::AppCore};

const WIDGET_LABEL: &str = "widget";
const WIDGET_WIDTH: f64 = 640.0;
const WIDGET_HEIGHT: f64 = 360.0;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetState {
    pub visible: bool,
    pub locked: bool,
}

pub async fn state(app: &AppHandle, core: &Arc<AppCore>) -> WidgetState {
    let config = core.config.read().await;
    let visible = app
        .get_webview_window(WIDGET_LABEL)
        .and_then(|window| window.is_visible().ok())
        .unwrap_or(config.widget_visible);
    WidgetState {
        visible,
        locked: config.widget_locked,
    }
}

pub async fn restore(app: &AppHandle, core: Arc<AppCore>) -> Result<()> {
    if core.config.read().await.widget_visible {
        let window = ensure_window(app, core.clone()).await?;
        window.show()?;
    }
    Ok(())
}

pub async fn toggle(app: &AppHandle, core: Arc<AppCore>) -> Result<WidgetState> {
    let is_visible = app
        .get_webview_window(WIDGET_LABEL)
        .and_then(|window| window.is_visible().ok())
        .unwrap_or(false);

    if is_visible {
        if let Some(window) = app.get_webview_window(WIDGET_LABEL) {
            window.hide()?;
        }
        update_visibility(&core, false).await?;
    } else {
        let window = ensure_window(app, core.clone()).await?;
        window.show()?;
        if !core.config.read().await.widget_locked {
            let _ = window.set_focus();
        }
        update_visibility(&core, true).await?;
    }
    Ok(state(app, &core).await)
}

pub async fn set_locked(app: &AppHandle, core: Arc<AppCore>, locked: bool) -> Result<WidgetState> {
    core.update_config(|config| config.widget_locked = locked)
        .await?;
    if let Some(window) = app.get_webview_window(WIDGET_LABEL) {
        apply_lock(&window, locked)?;
    }
    Ok(state(app, &core).await)
}

pub async fn toggle_lock(app: &AppHandle, core: Arc<AppCore>) -> Result<WidgetState> {
    let locked = !core.config.read().await.widget_locked;
    set_locked(app, core, locked).await
}

pub async fn refresh(app: &AppHandle, core: &Arc<AppCore>) -> Result<()> {
    if let Some(window) = app.get_webview_window(WIDGET_LABEL) {
        let url: tauri::Url = widget_url(core).await?.parse()?;
        if needs_navigation(&window, &url) {
            window.navigate(url)?;
        }
    }
    Ok(())
}

/// Reloading interrupts the media playing in the window, so only navigate
/// when the connection itself changed (port or secret). Language and lock
/// changes are already applied live through the WebSocket and apply_lock.
pub fn needs_navigation(window: &WebviewWindow, target: &tauri::Url) -> bool {
    let Ok(current) = window.url() else {
        return true;
    };
    current.origin() != target.origin()
        || query_param(&current, "secret") != query_param(target, "secret")
}

fn query_param(url: &tauri::Url, name: &str) -> Option<String> {
    url.query_pairs()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.into_owned())
}

async fn ensure_window(app: &AppHandle, core: Arc<AppCore>) -> Result<WebviewWindow> {
    if let Some(window) = app.get_webview_window(WIDGET_LABEL) {
        return Ok(window);
    }

    let config = core.config.read().await.clone();
    let url = widget_url(&core).await?;
    let window = WebviewWindowBuilder::new(app, WIDGET_LABEL, WebviewUrl::External(url.parse()?))
        .title("Relay - Widget")
        .inner_size(WIDGET_WIDTH, WIDGET_HEIGHT)
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

    let position = resolved_position(app, config.widget_x.zip(config.widget_y))?;
    window.set_position(position)?;
    apply_lock(&window, config.widget_locked)?;
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
        let generation = core.widget_move_generation.fetch_add(1, Ordering::Relaxed) + 1;
        let core = core.clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(400)).await;
            if core.widget_move_generation.load(Ordering::Relaxed) != generation {
                return;
            }
            let _ = core
                .update_config(|config| {
                    config.widget_x = Some(position.x);
                    config.widget_y = Some(position.y);
                })
                .await;
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
    let physical_width = (WIDGET_WIDTH * scale).round() as i32;
    let physical_height = (WIDGET_HEIGHT * scale).round() as i32;
    Ok(PhysicalPosition::new(
        area.position.x + (area.size.width as i32 - physical_width) / 2,
        area.position.y + (area.size.height as i32 - physical_height) / 2,
    ))
}

async fn update_visibility(core: &Arc<AppCore>, visible: bool) -> Result<()> {
    core.update_config(|config| config.widget_visible = visible)
        .await
        .map(|_| ())
}

async fn widget_url(core: &Arc<AppCore>) -> Result<String> {
    let config = core.config.read().await.clone();
    let secret = load_or_create_relay_secret()?;
    let preferences = core.interface_preferences.read().await;
    Ok(format!(
        "http://127.0.0.1:{}/overlay?secret={secret}&widget=1&locked={}&lang={}",
        config.port,
        if config.widget_locked { 1 } else { 0 },
        preferences.language,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widget_uses_arklay_dimensions() {
        assert_eq!((WIDGET_WIDTH, WIDGET_HEIGHT), (640.0, 360.0));
    }
}
