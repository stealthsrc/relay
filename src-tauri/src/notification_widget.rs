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

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationWidgetState {
    pub visible: bool,
    pub locked: bool,
}

fn configured_size(config: &crate::config::AppConfig) -> (f64, f64) {
    (
        config.notification_widget_width,
        config.notification_widget_height,
    )
}

fn saved_position(config: &crate::config::AppConfig) -> Option<(i32, i32)> {
    config
        .notification_widget_x
        .zip(config.notification_widget_y)
}

fn write_position(config: &mut crate::config::AppConfig, position: PhysicalPosition<i32>) {
    config.notification_widget_x = Some(position.x);
    config.notification_widget_y = Some(position.y);
    config.music_widget_x = Some(position.x);
    config.music_widget_y = Some(position.y);
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
        show_with_saved_placement(app, &core, &window).await?;
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
        show_with_saved_placement(app, &core, &window).await?;
        if !core.config.read().await.notification_widget_locked {
            let _ = window.set_focus();
        }
    } else if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
        // Capture live coords before hide — Windows may not emit a final Moved.
        flush_live_position(&window, &core).await?;
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
    if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
        // Unlock → move → lock is the normal edit session. Flush immediately on
        // lock so a debounced Moved write cannot be lost if the app quits next.
        if locked {
            flush_live_position(&window, &core).await?;
        }
        core.update_config(|config| config.notification_widget_locked = locked)
            .await?;
        apply_lock(&window, locked)?;
    } else {
        core.update_config(|config| config.notification_widget_locked = locked)
            .await?;
    }
    Ok(state(app, &core).await)
}

pub fn clamp_requested_size(
    app: &AppHandle,
    width: f64,
    height: f64,
    _content_scale: u16,
) -> Result<(f64, f64)> {
    let monitor = app
        .get_webview_window(WINDOW_LABEL)
        .and_then(|window| window.current_monitor().ok().flatten())
        .or(app.primary_monitor()?)
        .or_else(|| app.available_monitors().ok()?.into_iter().next())
        .context("no display is available")?;
    let area = monitor.work_area();
    let scale = monitor.scale_factor();
    // Honor the Overlay size fields down to MIN_WIDGET_HEIGHT (90). Do not
    // re-inflate from content-scale — that previously locked height around
    // 144–160 and made "Saved" 100px appear to do nothing.
    Ok((
        width.clamp(MIN_WIDGET_WIDTH, area.size.width as f64 / scale),
        height.clamp(MIN_WIDGET_HEIGHT, area.size.height as f64 / scale),
    ))
}

pub fn apply_configured_size(
    app: &AppHandle,
    core: &Arc<AppCore>,
    width: f64,
    height: f64,
    _content_scale: u16,
    preserve_right_edge: bool,
) -> Result<()> {
    if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
        apply_monitor_constraints(&window, MIN_WIDGET_HEIGHT)?;
        let scale = window.scale_factor().unwrap_or(1.0);
        if let (Ok(current_size), Ok(current_pos)) = (window.inner_size(), window.outer_position())
        {
            let current = current_size.to_logical::<f64>(scale);
            let position = next_position_for_size_change(
                current_pos,
                current.width,
                current.height,
                width,
                height,
                scale,
                preserve_right_edge,
            );
            // Never nudge inward from a docked edge — only rescue if the
            // resized window would be completely invisible.
            let position = ensure_partially_visible(app, position, width, height)?;
            if position != current_pos {
                window.set_position(position)?;
                // Panel resizes that move x (right-anchor) must persist; content-
                // mode keeps top-left and relies on Moved / lock flush.
                if preserve_right_edge {
                    persist_position(core.clone(), position);
                }
            }
        }
        window.set_size(LogicalSize::new(width, height))?;
    }
    Ok(())
}

/// Physical top-left for a size change. Panel edits may keep the right edge;
/// notification-only resizes otherwise keep the user's top-left.
fn next_position_for_size_change(
    current_pos: PhysicalPosition<i32>,
    current_width: f64,
    current_height: f64,
    next_width: f64,
    next_height: f64,
    scale: f64,
    preserve_right_edge: bool,
) -> PhysicalPosition<i32> {
    if preserve_right_edge {
        right_anchored_position(
            current_pos,
            current_width,
            current_height,
            next_width,
            next_height,
            scale,
        )
    } else {
        let _ = (
            current_width,
            current_height,
            next_width,
            next_height,
            scale,
        );
        current_pos
    }
}

/// Physical top-left that keeps the previous right edge when width changes.
fn right_anchored_position(
    current_pos: PhysicalPosition<i32>,
    current_width: f64,
    current_height: f64,
    next_width: f64,
    next_height: f64,
    scale: f64,
) -> PhysicalPosition<i32> {
    let _ = (current_height, next_height);
    let width_delta = ((next_width - current_width) * scale).round() as i32;
    PhysicalPosition::new(current_pos.x - width_delta, current_pos.y)
}

#[derive(Clone, Copy, Debug)]
struct MonitorBounds {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    scale: f64,
}

impl MonitorBounds {
    fn from_monitor(monitor: &tauri::Monitor) -> Self {
        // Use the full display rect (not work_area). Work-area clamping was
        // pulling edge-docked widgets inward on every restore.
        let size = monitor.size();
        let position = monitor.position();
        Self {
            x: position.x,
            y: position.y,
            width: size.width as i32,
            height: size.height as i32,
            scale: monitor.scale_factor(),
        }
    }

    fn contains_point(self, position: PhysicalPosition<i32>) -> bool {
        position.x >= self.x
            && position.y >= self.y
            && position.x < self.x + self.width
            && position.y < self.y + self.height
    }

    fn intersects_window(
        self,
        position: PhysicalPosition<i32>,
        physical_width: i32,
        physical_height: i32,
    ) -> bool {
        let right = position.x.saturating_add(physical_width);
        let bottom = position.y.saturating_add(physical_height);
        position.x < self.x + self.width
            && right > self.x
            && position.y < self.y + self.height
            && bottom > self.y
    }
}

/// Restore saved x/y exactly. Only move when the window would have zero
/// intersection with every monitor (e.g. display unplugged) — never inset from
/// work-area edges or keep the full frame inside the work area.
fn ensure_partially_visible(
    app: &AppHandle,
    position: PhysicalPosition<i32>,
    width: f64,
    height: f64,
) -> Result<PhysicalPosition<i32>> {
    let monitors = app.available_monitors()?;
    let bounds: Vec<MonitorBounds> = monitors.iter().map(MonitorBounds::from_monitor).collect();
    let primary = app
        .primary_monitor()?
        .map(|monitor| MonitorBounds::from_monitor(&monitor));
    Ok(preserve_or_rescue_position(
        position, width, height, &bounds, primary,
    ))
}

/// Pure placement policy used for notification restore and resize.
fn preserve_or_rescue_position(
    position: PhysicalPosition<i32>,
    width: f64,
    height: f64,
    monitors: &[MonitorBounds],
    primary: Option<MonitorBounds>,
) -> PhysicalPosition<i32> {
    if monitors.is_empty() {
        return position;
    }
    let scale_hint = monitors
        .iter()
        .find(|monitor| monitor.contains_point(position))
        .or(monitors.first())
        .map(|monitor| monitor.scale)
        .unwrap_or(1.0);
    let physical_width = (width * scale_hint).round().max(1.0) as i32;
    let physical_height = (height * scale_hint).round().max(1.0) as i32;
    if monitors
        .iter()
        .any(|monitor| monitor.intersects_window(position, physical_width, physical_height))
    {
        return position;
    }
    // Completely off-screen: park flush top-right on the primary/first monitor
    // with no decorative margin.
    let fallback = primary.unwrap_or(monitors[0]);
    let physical_width = (width * fallback.scale).round().max(1.0) as i32;
    PhysicalPosition::new(fallback.x + fallback.width - physical_width, fallback.y)
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
        // Do not rewrite saved x/y here; a refit can clamp against stale
        // geometry and wipe the user's placement across relaunch.
        apply_monitor_constraints(&window, MIN_WIDGET_HEIGHT)?;
        return Ok(window);
    }

    let config = core.config.read().await.clone();
    let (configured_width, configured_height) = configured_size(&config);
    let url = widget_url(&core).await?;
    let (position, max_width, max_height) =
        resolved_geometry(app, saved_position(&config), configured_width)?;
    let width = configured_width.clamp(MIN_WIDGET_WIDTH, max_width);
    let height = configured_height.clamp(MIN_WIDGET_HEIGHT, max_height);
    // Keep exact saved coords when still partially visible; only rescue if
    // the placement would be completely off every monitor.
    let position = ensure_partially_visible(app, position, width, height)?;
    let window = WebviewWindowBuilder::new(app, WINDOW_LABEL, WebviewUrl::External(url.parse()?))
        .title("Relay - Notifications")
        .inner_size(width, height)
        .min_inner_size(MIN_WIDGET_WIDTH, MIN_WIDGET_HEIGHT)
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
    apply_monitor_constraints(&window, MIN_WIDGET_HEIGHT)?;
    watch_geometry(&window, core.clone());
    // Persist only when rescue changed placement; never rewrite a valid edge dock.
    if saved_position(&config) != Some((position.x, position.y)) {
        core.update_config(|config| write_position(config, position))
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
        // Short debounce coalesces drag spam; lock/hide also flush immediately.
        tokio::time::sleep(std::time::Duration::from_millis(80)).await;
        if core
            .notification_widget_move_generation
            .load(Ordering::Relaxed)
            != generation
        {
            return;
        }
        let _ = core
            .update_config(|config| write_position(config, position))
            .await;
    });
}

async fn flush_live_position(window: &WebviewWindow, core: &Arc<AppCore>) -> Result<()> {
    let Ok(position) = window.outer_position() else {
        return Ok(());
    };
    // Invalidate in-flight debounced writes so they cannot overwrite this flush.
    core.notification_widget_move_generation
        .fetch_add(1, Ordering::Relaxed);
    core.update_config(|config| write_position(config, position))
        .await?;
    Ok(())
}

/// Windows often ignores SetWindowPos while the window is still hidden. Show
/// first, then re-apply the saved physical top-left from config.
async fn show_with_saved_placement(
    app: &AppHandle,
    core: &Arc<AppCore>,
    window: &WebviewWindow,
) -> Result<()> {
    let was_visible = window.is_visible().unwrap_or(false);
    let config = core.config.read().await.clone();
    let (width, height) = configured_size(&config);
    window.show()?;
    // Do not yank an in-progress drag back to the last flushed config when the
    // notification window is already on-screen.
    if was_visible {
        return Ok(());
    }
    if let Some((x, y)) = saved_position(&config) {
        let position = ensure_partially_visible(app, PhysicalPosition::new(x, y), width, height)?;
        // Invalidate stale debounced writes from before hide, then place.
        core.notification_widget_move_generation
            .fetch_add(1, Ordering::Relaxed);
        window.set_position(position)?;
        if position.x != x || position.y != y {
            core.update_config(|config| write_position(config, position))
                .await?;
        }
    }
    Ok(())
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
                config.music_widget_width = logical.width;
                config.music_widget_height = logical.height;
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
    // Match against the full display bounds so an edge-docked top-left that
    // sits in the taskbar strip (outside work_area) still resolves to that
    // monitor instead of falling back to the default inset position.
    let saved_monitor = saved.and_then(|(x, y)| {
        monitors.iter().find(|monitor| {
            MonitorBounds::from_monitor(monitor).contains_point(PhysicalPosition::new(x, y))
        })
    });
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
    let position = if let Some((x, y)) = saved {
        // Keep exact coords even if the owning monitor vanished — rescue runs
        // in ensure_partially_visible without inventing a default inset.
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

fn configured_minimum_height(_core: &Arc<AppCore>) -> Option<f64> {
    Some(MIN_WIDGET_HEIGHT)
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
        "http://{}:{}/notifications?secret={secret}&target=widget&locked={}&lang={}",
        crate::widget::youtube_embed_host(),
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
    use super::{
        MonitorBounds, configured_size, next_position_for_size_change, preserve_or_rescue_position,
        right_anchored_position, saved_position, write_position,
    };
    use crate::config::AppConfig;
    use tauri::PhysicalPosition;

    fn primary_1080p() -> MonitorBounds {
        MonitorBounds {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
            scale: 1.0,
        }
    }

    #[test]
    fn notification_widget_defaults_remain_compact() {
        assert_eq!(
            (
                crate::config::DEFAULT_NOTIFICATION_WIDGET_WIDTH,
                crate::config::DEFAULT_NOTIFICATION_WIDGET_HEIGHT,
            ),
            (400.0, 104.0)
        );
    }

    #[test]
    fn notification_widget_allows_compact_overlay_heights() {
        assert!(crate::config::MIN_WIDGET_HEIGHT <= 100.0);
        assert_eq!(crate::config::MIN_WIDGET_HEIGHT, 90.0);
    }

    #[test]
    fn configured_size_uses_notification_dimensions() {
        let config = AppConfig {
            notification_widget_width: 540.0,
            notification_widget_height: 112.0,
            music_widget_width: 980.0,
            music_widget_height: 360.0,
            ..AppConfig::default()
        };
        assert_eq!(configured_size(&config), (540.0, 112.0));
    }

    #[test]
    fn right_anchored_resize_keeps_the_right_edge() {
        let next = right_anchored_position(
            PhysicalPosition::new(1400, 40),
            530.0,
            160.0,
            980.0,
            180.0,
            1.0,
        );
        assert_eq!(next, PhysicalPosition::new(950, 40));
    }

    #[test]
    fn height_resize_keeps_the_top_edge_fixed() {
        let next = right_anchored_position(
            PhysicalPosition::new(1400, 40),
            980.0,
            160.0,
            980.0,
            280.0,
            1.0,
        );
        assert_eq!(next, PhysicalPosition::new(1400, 40));
    }

    #[test]
    fn non_anchored_resize_keeps_exact_top_left() {
        let saved = PhysicalPosition::new(2020, 0);
        let next = next_position_for_size_change(saved, 460.0, 90.0, 540.0, 360.0, 1.0, false);
        assert_eq!(next, saved);
    }

    #[test]
    fn panel_resize_still_right_anchors_when_requested() {
        let next = next_position_for_size_change(
            PhysicalPosition::new(2020, 0),
            460.0,
            90.0,
            540.0,
            360.0,
            1.0,
            true,
        );
        assert_eq!(next, PhysicalPosition::new(1940, 0));
    }

    #[test]
    fn restore_keeps_flush_right_edge_dock() {
        let width = 980.0;
        let height = 180.0;
        let monitor = primary_1080p();
        let x = monitor.width - (width as i32);
        let saved = PhysicalPosition::new(x, 12);
        let restored = preserve_or_rescue_position(saved, width, height, &[monitor], Some(monitor));
        assert_eq!(restored, saved);
    }

    #[test]
    fn restore_does_not_nudge_when_frame_extends_past_work_area_inset() {
        // Simulate docking flush to the true screen right while an 8px work-area
        // inset exists on that edge — old clamp_position_to_area shifted left.
        let monitor = primary_1080p();
        let width = 980.0;
        let height = 180.0;
        let x = monitor.width - (width as i32);
        let saved = PhysicalPosition::new(x, 20);
        let restored = preserve_or_rescue_position(saved, width, height, &[monitor], Some(monitor));
        assert_eq!(restored, saved);
        assert_eq!(restored.x + width as i32, monitor.width);
    }

    #[test]
    fn restore_rescues_only_when_completely_off_every_monitor() {
        let monitor = primary_1080p();
        let rescued = preserve_or_rescue_position(
            PhysicalPosition::new(8000, 4000),
            980.0,
            180.0,
            &[monitor],
            Some(monitor),
        );
        assert_eq!(rescued, PhysicalPosition::new(monitor.width - 980, 0));
    }

    #[test]
    fn write_position_synchronizes_the_shared_widget_dock() {
        let mut config = AppConfig::default();
        config.notification_widget_x = Some(100);
        config.notification_widget_y = Some(20);
        config.music_widget_x = Some(880);
        config.music_widget_y = Some(40);

        write_position(&mut config, PhysicalPosition::new(2100, 12));
        assert_eq!(config.notification_widget_x, Some(2100));
        assert_eq!(config.notification_widget_y, Some(12));
        assert_eq!(config.music_widget_x, Some(2100));
        assert_eq!(config.music_widget_y, Some(12));
    }

    #[test]
    fn saved_position_uses_the_shared_widget_dock() {
        let mut config = AppConfig::default();
        config.notification_widget_x = Some(100);
        config.notification_widget_y = Some(20);
        config.music_widget_x = Some(880);
        config.music_widget_y = Some(40);
        assert_eq!(saved_position(&config), Some((100, 20)));
    }

    #[test]
    fn persist_position_keys_round_trip_through_config_store() {
        use crate::config::ConfigStore;

        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("config.json");
        let store = ConfigStore::new(path.clone());
        let mut config = AppConfig::default();
        config.notification_widget_x = Some(2020);
        config.notification_widget_y = Some(48);
        config.music_widget_x = Some(1880);
        config.music_widget_y = Some(64);
        store.save(&config).unwrap();

        let loaded = ConfigStore::new(path.clone()).load().unwrap();
        assert_eq!(loaded.notification_widget_x, Some(2020));
        assert_eq!(loaded.notification_widget_y, Some(48));
        assert_eq!(loaded.music_widget_x, Some(1880));
        assert_eq!(loaded.music_widget_y, Some(64));

        let persisted: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(persisted["notificationWidgetX"], 2020);
        assert_eq!(persisted["notificationWidgetY"], 48);
        assert_eq!(persisted["musicWidgetX"], 1880);
        assert_eq!(persisted["musicWidgetY"], 64);
    }
}
