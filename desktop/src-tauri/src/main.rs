#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, LogicalSize, Manager, WindowEvent,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use tauri_plugin_notification::NotificationExt;

/// Makes the page behave like an app, not a web page: no right-click menu, no text selection, no zoom, no reload or
/// view-source shortcuts, no drag-and-drop navigation, no scrollbars. Runs in every page the window loads.
const APP_FEEL: &str = r#"(() => {
  window.__JARVIS_APP = true;
  const css = 'html,body{-webkit-user-select:none;user-select:none;cursor:default;overscroll-behavior:none}input,textarea,select{-webkit-user-select:text;user-select:text}::-webkit-scrollbar{display:none}';
  const add = () => { const s = document.createElement('style'); s.textContent = css; (document.head || document.documentElement).appendChild(s); };
  document.readyState === 'loading' ? document.addEventListener('DOMContentLoaded', add) : add();
  addEventListener('contextmenu', (e) => e.preventDefault());
  addEventListener('wheel', (e) => { if (e.ctrlKey) e.preventDefault(); }, { passive: false });
  addEventListener('dragover', (e) => e.preventDefault());
  addEventListener('drop', (e) => e.preventDefault());
  addEventListener('keydown', (e) => {
    const k = e.key.toLowerCase();
    if ((e.ctrlKey && ['+', '-', '=', '0', 'r', 'u', 's', 'p', 'f', 'g'].includes(k)) || ['F3', 'F5', 'F7', 'F12'].includes(e.key) || (e.ctrlKey && e.shiftKey && ['i', 'j', 'c'].includes(k))) e.preventDefault();
  }, true);
})();"#;

const FULL: (f64, f64) = (1020.0, 820.0);
const MINI: (f64, f64) = (260.0, 260.0);

/// Say yes to the microphone for our own page, without the prompt. Anything else (screen capture, camera, location...) keeps
/// WebView2's normal behaviour, so the screen-share picker still appears.
#[cfg(windows)]
fn allow_microphone(window: &tauri::WebviewWindow) {
    let _ = window.with_webview(|webview| unsafe {
        use webview2_com::Microsoft::Web::WebView2::Win32::{
            COREWEBVIEW2_PERMISSION_KIND, COREWEBVIEW2_PERMISSION_KIND_MICROPHONE, COREWEBVIEW2_PERMISSION_STATE_ALLOW,
        };
        use webview2_com::PermissionRequestedEventHandler;
        if let Ok(core) = webview.controller().CoreWebView2() {
            let mut token = Default::default();
            let _ = core.add_PermissionRequested(
                &PermissionRequestedEventHandler::create(Box::new(|_, args| {
                    if let Some(args) = args {
                        let mut kind = COREWEBVIEW2_PERMISSION_KIND::default();
                        args.PermissionKind(&mut kind)?;
                        if kind == COREWEBVIEW2_PERMISSION_KIND_MICROPHONE {
                            args.SetState(COREWEBVIEW2_PERMISSION_STATE_ALLOW)?;
                        }
                    }
                    Ok(())
                })),
                &mut token,
            );
        }
    });
}

#[cfg(not(windows))]
fn allow_microphone(_window: &tauri::WebviewWindow) {}

fn main_window(app: &AppHandle) -> Option<tauri::WebviewWindow> {
    app.get_webview_window("main")
}

fn show(app: &AppHandle) {
    if let Some(w) = main_window(app) {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

fn toggle(app: &AppHandle) {
    if let Some(w) = main_window(app) {
        if w.is_visible().unwrap_or(false) && w.is_focused().unwrap_or(false) {
            let _ = w.hide();
        } else {
            show(app);
        }
    }
}

/// The mini orb is the same window made small, undecorated and always on top: one page, one socket, one microphone.
fn set_mini(app: &AppHandle, on: bool) {
    if let Some(w) = main_window(app) {
        let (width, height) = if on { MINI } else { FULL };
        let _ = w.set_decorations(!on);
        let _ = w.set_always_on_top(on);
        let _ = w.set_size(LogicalSize::new(width, height));
        show(app);
    }
}

fn is_mini(app: &AppHandle) -> bool {
    main_window(app).map(|w| w.is_always_on_top().unwrap_or(false)).unwrap_or(false)
}

fn pause_listening(app: &AppHandle, paused: bool) {
    if let Some(w) = main_window(app) {
        let _ = w.eval(&format!("window.__orbPause && window.__orbPause({paused})"));
    }
}

/// Called by the page (hostname-restricted capability) when an alert arrives while the window is hidden.
#[tauri::command]
fn toast(app: AppHandle, title: String, body: String) {
    let title: String = title.chars().take(80).collect();
    let body: String = body.chars().take(300).collect();
    let _ = app.notification().builder().title(title).body(body).show();
}

#[tauri::command]
fn mini(app: AppHandle, on: bool) {
    set_mini(&app, on);
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| show(app)))
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec!["--minimized"])))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().with_handler(|app, shortcut, event| {
            if event.state() != ShortcutState::Pressed {
                return;
            }
            if shortcut.matches(Modifiers::CONTROL | Modifiers::ALT, Code::KeyJ) {
                toggle(app);
            } else if shortcut.matches(Modifiers::CONTROL | Modifiers::ALT, Code::KeyM) {
                set_mini(app, !is_mini(app));
            }
        }).build())
        .invoke_handler(tauri::generate_handler![toast, mini])
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::App("index.html".into()))
                .title("Jarvis AI")
                .inner_size(FULL.0, FULL.1)
                .min_inner_size(240.0, 240.0)
                .center()
                .theme(Some(tauri::Theme::Dark))
                .initialization_script(APP_FEEL)
                // Let the page receive dropped files itself (picture, document, script) instead of Windows handling the drop.
                .disable_drag_drop_handler()
                // Audio without a first click, so "Hey Jarvis" works from start-up. (The microphone is allowed in allow_microphone();
                // the blanket --use-fake-ui-for-media-stream flag would also skip the screen-share picker, so it is not used.)
                .additional_browser_args("--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection --autoplay-policy=no-user-gesture-required")
                // Only our page, the Cloudflare sign-in and the bundled "connecting" page may load in this window.
                .on_navigation(|url| {
                    let host = url.host_str().unwrap_or("");
                    matches!(url.scheme(), "tauri" | "about" | "blob" | "data")
                        || host == "tauri.localhost"
                        || host == "jarvis.example.com"
                        || host.ends_with(".cloudflareaccess.com")
                })
                .build()
                .map(|w| allow_microphone(&w))?;
            for key in [Code::KeyJ, Code::KeyM] {
                // A shortcut another program already owns is not fatal.
                let _ = handle.global_shortcut().register(Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), key));
            }

            let open = MenuItem::with_id(app, "open", "Open Jarvis AI  (Ctrl+Alt+J)", true, None::<&str>)?;
            let mini_item = CheckMenuItem::with_id(app, "mini", "Mini orb  (Ctrl+Alt+M)", true, false, None::<&str>)?;
            let pause = CheckMenuItem::with_id(app, "pause", "Pause listening", true, false, None::<&str>)?;
            // Start with Windows is on by default: the first run turns it on, and after that the tray tick decides.
            if let Ok(dir) = app.path().app_config_dir() {
                let marker = dir.join("autostart-default-set");
                if !marker.exists() {
                    let _ = app.autolaunch().enable();
                    let _ = std::fs::create_dir_all(&dir);
                    let _ = std::fs::write(&marker, "1");
                }
            }
            let autostart_on = app.autolaunch().is_enabled().unwrap_or(false);
            let auto = CheckMenuItem::with_id(app, "auto", "Start with Windows", true, autostart_on, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let sep = PredefinedMenuItem::separator(app)?;
            let menu = Menu::with_items(app, &[&open, &mini_item, &pause, &auto, &sep, &quit])?;

            let (mini_c, pause_c, auto_c) = (mini_item.clone(), pause.clone(), auto.clone());
            TrayIconBuilder::with_id("tray")
                .icon(app.default_window_icon().cloned().ok_or("no icon")?)
                .tooltip("Jarvis AI")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "open" => show(app),
                    "mini" => {
                        let on = mini_c.is_checked().unwrap_or(false);
                        set_mini(app, on);
                    }
                    "pause" => {
                        let paused = pause_c.is_checked().unwrap_or(false);
                        pause_listening(app, paused);
                        if let Some(t) = app.tray_by_id("tray") {
                            let _ = t.set_tooltip(Some(if paused { "Jarvis AI (not listening)" } else { "Jarvis AI" }));
                        }
                    }
                    "auto" => {
                        let on = auto_c.is_checked().unwrap_or(false);
                        let al = app.autolaunch();
                        let _ = if on { al.enable() } else { al.disable() };
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                        show(tray.app_handle());
                    }
                })
                .build(app)?;

            if std::env::args().any(|a| a == "--minimized") {
                if let Some(w) = main_window(&handle) {
                    let _ = w.hide();
                }
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            // The close button hides to the tray (listening continues); Quit in the tray menu really exits.
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Jarvis AI");
}
