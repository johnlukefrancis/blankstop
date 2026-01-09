# Windows Installer & Distribution

This doc reflects the repo's current Tauri configuration and observed build
outputs. It is intentionally scoped to Windows packaging.

## Build outputs (Windows)

Config source: `src-tauri/tauri.conf.json`
- `bundle.targets` is set to `"all"`.

Observed outputs in this repo:
- There are no Windows release bundles checked in at this time.

Where to find artifacts after a Windows build:
- `src-tauri/target/release/bundle/` (bundle-type subfolders created by Tauri)

Note: Because no Windows bundle outputs are present in this repo, the exact
bundle types and filenames must be read from the Windows build output directory
above. Do not assume a specific installer type without verifying the folder
contents after `pnpm tauri build` on Windows.

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
