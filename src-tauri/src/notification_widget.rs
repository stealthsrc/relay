use std::sync::{Arc, atomic::Ordering};

use anyhow::{Context, Result};
use serde::Serialize;
use tauri::{
    AppHandle, LogicalSize, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder, WindowSizeConstraints,
};

use crate::{
    config::{MIN_WIDGET_HEIGHT, MIN_WIDGET_WIDTH},
    credentials::load_or_create_relay_secret,
    state::AppCore,
};

const WINDOW_LABEL: &str = "notification-widget";
const DEFAULT_MARGIN: i32 = 28;
const CARD_BASE_HEIGHT: f64 = 94.0;
const WIDGET_VERTICAL_PADDING: f64 = 16.0;

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
    core.update_config(|config| config.notification_widget_locked = locked)
        .await?;
    if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
        apply_lock(&window, locked)?;
    }
    Ok(state(app, &core).await)
}

pub fn clamp_requested_size(
    app: &AppHandle,
    width: f64,
    height: f64,
    content_scale: u16,
) -> Result<(f64, f64)> {
    let monitor = app
        .get_webview_window(WINDOW_LABEL)
        .and_then(|window| window.current_monitor().ok().flatten())
        .or(app.primary_monitor()?)
        .or_else(|| app.available_monitors().ok()?.into_iter().next())
        .context("no display is available")?;
    let area = monitor.work_area();
    let scale = monitor.scale_factor();
    let height = height.max(minimum_height_for_content_scale(content_scale));
    Ok((
        width.clamp(MIN_WIDGET_WIDTH, area.size.width as f64 / scale),
        height.clamp(MIN_WIDGET_HEIGHT, area.size.height as f64 / scale),
    ))
}

fn minimum_height_for_content_scale(content_scale: u16) -> f64 {
    let scale = f64::from(content_scale.clamp(50, 200)) / 100.0;
    (CARD_BASE_HEIGHT * scale + WIDGET_VERTICAL_PADDING).ceil()
}

pub fn apply_configured_size(
    app: &AppHandle,
    width: f64,
    height: f64,
    content_scale: u16,
) -> Result<()> {
    if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
        apply_monitor_constraints(&window, minimum_height_for_content_scale(content_scale))?;
        window.set_size(LogicalSize::new(width, height))?;
    }
    Ok(())
}

pub async fn refresh(app: &AppHandle, core: &Arc<AppCore>) -> Result<()> {
    if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
        let url: tauri::Url = widget_url(core).await?.parse()?;
        if crate::widget::needs_navigation(&window, &url) {
            window.navigate(url)?;
        }
    }
    Ok(())
}

async fn ensure_window(app: &AppHandle, core: Arc<AppCore>) -> Result<WebviewWindow> {
    if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
        return Ok(window);
    }

    let config = core.config.read().await.clone();
    let url = widget_url(&core).await?;
    let (position, max_width, max_height) = resolved_geometry(
        app,
        config
            .notification_widget_x
            .zip(config.notification_widget_y),
        config.notification_widget_width,
    )?;
    let width = config
        .notification_widget_width
        .clamp(MIN_WIDGET_WIDTH, max_width);
    let minimum_height =
        minimum_height_for_content_scale(config.notification_widget_geometry.content_scale)
            .min(max_height);
    let height = config
        .notification_widget_height
        .max(minimum_height)
        .clamp(MIN_WIDGET_HEIGHT, max_height);
    let window = WebviewWindowBuilder::new(app, WINDOW_LABEL, WebviewUrl::External(url.parse()?))
        .title("Relay - TTS notifications")
        .inner_size(width, height)
        .min_inner_size(MIN_WIDGET_WIDTH, minimum_height)
        .max_inner_size(max_width, max_height)
        .resizable(!config.notification_widget_locked)
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

    window.set_position(position)?;
    apply_lock(&window, config.notification_widget_locked)?;
    apply_monitor_constraints(&window, minimum_height)?;
    watch_geometry(&window, core.clone());
    if width != config.notification_widget_width || height != config.notification_widget_height {
        core.update_config(|config| {
            config.notification_widget_width = width;
            config.notification_widget_height = height;
        })
        .await?;
    }
    Ok(window)
}

fn apply_lock(window: &WebviewWindow, locked: bool) -> Result<()> {
    window.set_resizable(!locked)?;
    window.set_focusable(!locked)?;
    window.set_ignore_cursor_events(locked)?;
    window.eval(format!(
        "window.setWidgetLocked?.({})",
        if locked { "true" } else { "false" }
    ))?;
    Ok(())
}

fn watch_geometry(window: &WebviewWindow, core: Arc<AppCore>) {
    let window = window.clone();
    window.clone().on_window_event(move |event| match event {
        tauri::WindowEvent::Moved(position) => {
            if let Some(minimum_height) = configured_minimum_height(&core) {
                let _ = apply_monitor_constraints(&window, minimum_height);
            }
            persist_position(core.clone(), *position);
        }
        tauri::WindowEvent::Resized(size) => {
            persist_size(core.clone(), &window, *size);
        }
        tauri::WindowEvent::ScaleFactorChanged { .. } => {
            if let Some(minimum_height) = configured_minimum_height(&core) {
                let _ = apply_monitor_constraints(&window, minimum_height);
            }
        }
        _ => {}
    });
}

fn persist_position(core: Arc<AppCore>, position: PhysicalPosition<i32>) {
    let generation = core
        .notification_widget_move_generation
        .fetch_add(1, Ordering::Relaxed)
        + 1;
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(400)).await;
        if core
            .notification_widget_move_generation
            .load(Ordering::Relaxed)
            != generation
        {
            return;
        }
        let _ = core
            .update_config(|config| {
                config.notification_widget_x = Some(position.x);
                config.notification_widget_y = Some(position.y);
            })
            .await;
    });
}

fn persist_size(core: Arc<AppCore>, window: &WebviewWindow, size: PhysicalSize<u32>) {
    let generation = core
        .notification_widget_resize_generation
        .fetch_add(1, Ordering::Relaxed)
        + 1;
    let scale = window.scale_factor().unwrap_or(1.0);
    let logical = size.to_logical::<f64>(scale);
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(400)).await;
        if core
            .notification_widget_resize_generation
            .load(Ordering::Relaxed)
            != generation
        {
            return;
        }
        let _ = core
            .update_config(|config| {
                config.notification_widget_width = logical.width;
                config.notification_widget_height = logical.height;
            })
            .await;
    });
}

fn resolved_geometry(
    app: &AppHandle,
    saved: Option<(i32, i32)>,
    width: f64,
) -> Result<(PhysicalPosition<i32>, f64, f64)> {
    let monitors = app.available_monitors()?;
    let saved_monitor = saved.and_then(|(x, y)| {
        monitors.iter().find(|monitor| {
            let area = monitor.work_area();
            x >= area.position.x
                && y >= area.position.y
                && x < area.position.x + area.size.width as i32
                && y < area.position.y + area.size.height as i32
        })
    });
    let has_saved_monitor = saved_monitor.is_some();
    let monitor = match saved_monitor {
        Some(monitor) => monitor.clone(),
        None => app
            .primary_monitor()?
            .or_else(|| monitors.into_iter().next())
            .context("no display is available")?,
    };
    let area = monitor.work_area();
    let scale = monitor.scale_factor();
    let max_width = area.size.width as f64 / scale;
    let max_height = area.size.height as f64 / scale;
    let position = if has_saved_monitor {
        let (x, y) = saved.expect("saved monitor requires a saved position");
        PhysicalPosition::new(x, y)
    } else {
        let physical_width = (width.min(max_width) * scale).round() as i32;
        PhysicalPosition::new(
            area.position.x + area.size.width as i32 - physical_width - DEFAULT_MARGIN,
            area.position.y + DEFAULT_MARGIN,
        )
    };
    Ok((position, max_width, max_height))
}

fn configured_minimum_height(core: &Arc<AppCore>) -> Option<f64> {
    core.config.try_read().ok().map(|config| {
        minimum_height_for_content_scale(config.notification_widget_geometry.content_scale)
    })
}

fn apply_monitor_constraints(window: &WebviewWindow, minimum_height: f64) -> Result<()> {
    let monitor = window
        .current_monitor()?
        .context("no display is available")?;
    let area = monitor.work_area();
    let scale = monitor.scale_factor();
    let max_height = area.size.height as f64 / scale;
    window.set_size_constraints(WindowSizeConstraints {
        min_width: Some(tauri::LogicalUnit::new(MIN_WIDGET_WIDTH).into()),
        min_height: Some(tauri::LogicalUnit::new(minimum_height.min(max_height)).into()),
        max_width: Some(tauri::LogicalUnit::new(area.size.width as f64 / scale).into()),
        max_height: Some(tauri::LogicalUnit::new(max_height).into()),
    })?;
    Ok(())
}

async fn update_visibility(core: &Arc<AppCore>, visible: bool) -> Result<()> {
    core.update_config(|config| config.notification_widget_visible = visible)
        .await
        .map(|_| ())
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
    use super::minimum_height_for_content_scale;

    #[test]
    fn notification_widget_defaults_remain_compact() {
        assert_eq!(
            (
                crate::config::DEFAULT_NOTIFICATION_WIDGET_WIDTH,
                crate::config::DEFAULT_NOTIFICATION_WIDGET_HEIGHT,
            ),
            (510.0, 130.0)
        );
    }

    #[test]
    fn notification_widget_height_keeps_scaled_content_visible() {
        assert_eq!(minimum_height_for_content_scale(100), 110.0);
        assert_eq!(minimum_height_for_content_scale(135), 143.0);
        assert_eq!(minimum_height_for_content_scale(200), 204.0);
    }
}
