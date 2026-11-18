// xore desktop — a thin native shell around https://xore.net
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_notification::NotificationExt;

const HOME: &str = "https://xore.net";

fn toast(app: &AppHandle, body: &str) {
    let _ = app
        .notification()
        .builder()
        .title("xore")
        .body(body)
        .show();
}

fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

/// Called by bridge.js whenever the site updates its unread count.
#[tauri::command]
fn xore_badge(app: AppHandle, count: u32, notify: bool) {
    let title = if count > 0 {
        format!("xore ({})", count)
    } else {
        "xore".to_string()
    };
    let tip = if count > 0 {
        format!("xore — {} unread", count)
    } else {
        "xore".to_string()
    };

    let mut focused = false;
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.set_title(&title);
        focused = w.is_focused().unwrap_or(false) && w.is_visible().unwrap_or(false);
    }
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_tooltip(Some(&tip));
    }

    if notify && count > 0 && !focused {
        let body = if count == 1 {
            "1 new message".to_string()
        } else {
            format!("{} new messages", count)
        };
        toast(&app, &body);
    }
}

/// Open anything that is not xore.net in the user's real browser.
#[tauri::command]
fn xore_open_external(url: String) {
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return;
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", &url])
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(&url).spawn();
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None::<Vec<&str>>,
        ))
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![xore_badge, xore_open_external])
        .on_window_event(|window, event| {
            // Closing the window parks the app in the tray instead of quitting,
            // so notifications keep arriving.
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .setup(|app| {
            WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::External(HOME.parse().expect("bad home url")),
            )
            .title("xore")
            .inner_size(1180.0, 820.0)
            .min_inner_size(400.0, 560.0)
            .initialization_script(include_str!("../bridge.js"))
            .build()?;

            let open_i = MenuItem::with_id(app, "open", "Open xore", true, None::<&str>)?;
            let auto_i = MenuItem::with_id(app, "autostart", "Start on login", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open_i, &auto_i, &quit_i])?;

            TrayIconBuilder::with_id("main")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("xore")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => show_main(app),
                    "autostart" => {
                        let m = app.autolaunch();
                        let on = m.is_enabled().unwrap_or(false);
                        let ok = if on { m.disable() } else { m.enable() };
                        if ok.is_ok() {
                            toast(
                                app,
                                if on {
                                    "xore will no longer start on login"
                                } else {
                                    "xore will start on login"
                                },
                            );
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running xore desktop");
}
