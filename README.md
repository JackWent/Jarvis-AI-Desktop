# Jarvis AI Desktop

A small [Tauri 2](https://tauri.app/) (Rust) app that wraps a self-hosted voice assistant web page in its own Windows window. It adds a tray icon, global hotkeys (Ctrl+Alt+J show or hide, Ctrl+Alt+M mini orb), start with Windows, toast notifications, a pause-listening switch and a "can't reach the server" page. The result is a 2.5 MB unsigned installer.

This repo is only the desktop shell. The assistant itself (the server and the web page it loads) is private and is not included, so the app does nothing useful until you point it at a page of your own. The URL is in three places: `desktop/dist/index.html`, `desktop/src-tauri/src/main.rs` and `desktop/src-tauri/capabilities/remote.json`. They all say `jarvis.example.com`.

The write-up of how it was built, and why, is on [jackwent.co.uk](https://jackwent.co.uk/).

## What's worth reading

- `desktop/src-tauri/src/main.rs`: the tray, hotkeys, autostart, single-instance, a navigation allowlist so the window can only load your page and the sign-in page, and a WebView2 permission handler that allows the microphone but still shows the screen-share picker.
- `desktop/src-tauri/capabilities/remote.json`: the remote page can call two app commands (`toast` and `mini`) and nothing else. No file, shell or network access is exposed to it.
- `desktop/src-tauri/tauri.conf.json`: window and bundle settings for the NSIS installer.

## Building

The installer is cross-built from Linux with `cargo-xwin` and NSIS, in CI, not on a Windows machine. In short: install the Tauri CLI, then run `cargo tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc` from `desktop/src-tauri`. The installer is unsigned, so Windows SmartScreen will say "unknown publisher".

## Licence

MIT. The icons are the author's own artwork; replace them with your own before publishing a build.
