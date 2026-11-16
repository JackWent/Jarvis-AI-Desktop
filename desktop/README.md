# Jarvis AI (Windows desktop app)

A small Tauri (Rust) shell around the Jarvis Orb page: the same page, in its own window, plus a tray icon, a global
hotkey (Ctrl+Alt+J show/hide, Ctrl+Alt+M mini orb), start with Windows, Windows toasts for alerts when the window is
hidden, a pause-listening switch and a "can't reach Jarvis" page. It loads https://jarvis.example.com, so page
changes reach the app at once; only these extras need a new installer.

Built in GitLab CI (`desktop` job, Rust + cargo-xwin, unsigned NSIS `Setup.exe`), never on the Jarvis VM.
The installer is unsigned: Windows SmartScreen says "unknown publisher", choose More info, Run anyway.

Security: the page may call only two app commands (`toast`, `mini`), granted to this hostname alone
(`src-tauri/capabilities/remote.json`). No file, shell or network access is exposed to the page.
