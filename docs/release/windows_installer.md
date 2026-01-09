# Windows Installer & Distribution

This doc reflects the repo's current Tauri configuration and observed build
outputs. It is intentionally scoped to Windows packaging.

## Build outputs (Windows)

Config source: `src-tauri/tauri.conf.json`
- `bundle.targets` is set to `["nsis"]` (NSIS installer only).
- `bundle.windows.webviewInstallMode` is `embedBootstrapper` to keep installs
  self-contained.

## NSIS installer branding

The installer uses custom branding images:
- Header image: `src-tauri/windows/installer_header.bmp` (150x57)
- Sidebar image: `src-tauri/windows/installer_sidebar.bmp` (164x314)

These are referenced in `tauri.conf.json` under `bundle.windows.nsis.headerImage`
and `bundle.windows.nsis.sidebarImage`.

To regenerate from the app icon:
```bash
node scripts/generate_installer_bitmaps.mjs
```

The script creates dark-themed BMPs with the Blankstop icon. Run it after
updating the source icon (`src-tauri/icons/icon.png`).

Observed outputs in this repo:
- There are no Windows release bundles checked in at this time.

Where to find artifacts after a Windows build:
- NSIS installer output: `src-tauri/target/release/bundle/nsis/`

Note: No Windows bundle outputs are checked in. Verify filenames in the output
folder after running `pnpm tauri build` on Windows.

## MSI (WiX) status
MSI is not built by default. If we switch back to `bundle.targets = "all"` or
add `"msi"` later, WiX may fail unless the **VBSCRIPT** optional Windows
feature is enabled on the build machine.

## Prerequisites / tooling
Only what this repo actually uses:
- Node.js + `pnpm` (frontend build + Tauri CLI invocation).
- Rust toolchain (required by `Cargo.toml` for Tauri/Rust backend).

## Install / run
- Install using the Windows bundle produced by `pnpm tauri build`.
- After install, launch **Blankstop**; it runs as a tray app with no main
  window on startup. Open settings via the tray menu.

## Uninstall
- Uninstall using standard Windows app removal for the installed bundle type
  (Apps & features / Settings → Apps), or via the bundle's uninstaller if it
  supplies one.

## Code signing
- Code signing is not configured in this repo yet.
- Placeholder: add signing config here once a certificate and signing pipeline
  exist.
