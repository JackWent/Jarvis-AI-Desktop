fn main() {
    // Only these two commands exist for the page to call; capabilities/remote.json grants them to our hostname alone.
    tauri_build::try_build(
        tauri_build::Attributes::new().app_manifest(tauri_build::AppManifest::new().commands(&["toast", "mini"])),
    )
    .expect("failed to run tauri-build");
}
