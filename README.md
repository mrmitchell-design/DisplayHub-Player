# DisplayHub Player

Native Windows/Linux companion for DisplayHub.

## Install

Normal users should install a packaged build from the GitHub Actions release artifacts. They do **not** need Node.js, Rust or Git.

- Windows: run the generated NSIS `.exe` installer.
- Ubuntu/Debian: install the generated `.deb` package.
- Linux portable: use the generated AppImage when available.

Open DisplayHub Player, enter the DisplayHub server address, then claim the six-character code from **DisplayHub → Screens**.

## Development

Requirements: Node.js 22+, Rust stable, and Tauri 2 system prerequisites.

```bash
npm install
npm run tauri:dev
```

## Release builds

The GitHub Actions workflow builds Windows and Linux installers automatically. It can be run manually for testing; tags beginning with `v` also publish a GitHub Release.
