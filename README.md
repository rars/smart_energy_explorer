# Smart Energy Explorer

An energy usage dashboard. See video demo here: [https://www.youtube.com/watch?v=bXS-XeiCP00](https://www.youtube.com/watch?v=bXS-XeiCP00).

## Installation

### via App Stores

Available on:

- [Apple App Store](https://apps.apple.com/gb/app/smart-energy-explorer/id6746265942?mt=12)
- [Microsoft Windows Store](https://apps.microsoft.com/detail/9p1pkcf2f37l?hl=en-GB&gl=GB)

### via Homebrew on macOS

```bash
brew tap rars/homebrew-formulae
brew install --cask smart-energy-explorer
```

## Live electricity display

If you have a [Hildebrand Glow](https://shop.glowmarkt.com/products/display-and-cad-combined-for-smart-meter-customers) combined consumer access device (CAD) and in home display (IHD), this can be configured to publish electricity usage to an MQTT broker. Smart Energy Explorer can connect to the MQTT broker and will display that data within the app. To do this, set the MQTT settings under the settings section.

## Local AI access (MCP)

The app includes an experimental, read-only MCP server for local AI agents. It is disabled by default. Open **Settings → AI / MCP Access** and enable it before copying the generated configuration into an MCP-compatible AI client. The generated configuration uses `mcp-remote` so it works with clients that only support stdio MCP servers; Node.js/npm must be installed. The server listens only on the local computer at port `55168` and currently provides an `energy_summary` tool for querying electricity and gas usage over a date range.

The generated configuration supplies the bearer token through an environment variable rather than the `mcp-remote` command arguments. Treat it like a credential and revoke it with a factory reset if it is exposed.

## Development

This is a [tauri](https://tauri.app/start/) app with Angular frontend. You will need to follow the instructions to set up your environment to develop tauri Applications.

### Running locally

After checkout:

```bash
npm i
cargo tauri dev
```

### Publishing to the Mac App Store

One-time setup:

1. Install the pinned Ruby via [rbenv](https://github.com/rbenv/rbenv) (`rbenv install`, reads `.ruby-version`), then run `bundle install`.
2. Ensure `src-tauri/Entitlements.plist`, `src-tauri/Info.plist`, and the `.provisionprofile` are present locally (these are gitignored and not tracked in version control).
3. Generate an [App Store Connect API key](https://docs.fastlane.tools/app-store-connect-api/) and set `ASC_API_KEY_PATH` in your shell profile to point at its key JSON file.

To build and upload a new build to App Store Connect:

```bash
bundle exec fastlane mac release
```

This builds the Mac App Store `.app` via `cargo tauri build`, wraps it in a signed `.pkg` via `productbuild`, and uploads it to App Store Connect. It does **not** submit for review or touch metadata/screenshots — do that manually in App Store Connect once the build finishes processing.
