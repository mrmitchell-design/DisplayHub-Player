# DisplayHub Player

Native Windows/Linux companion for DisplayHub.

## Version 0.1 goal

One installer, one pairing flow. The player connects to a DisplayHub server, displays a pairing code, is claimed from the DisplayHub admin interface, and then loads the assigned signage experience. AirPlay receiving will be integrated into the same application after pairing is connected end-to-end.

## Development

Requirements: Node.js 22+, Rust stable, and the Tauri 2 system prerequisites for your OS.

```bash
npm install
npm run tauri:dev
```

The current pairing screen is a UI prototype. The next milestone connects it to the DisplayHub server so the displayed code is server-issued and claimable from Screens.
