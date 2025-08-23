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

## Development

This is a [tauri](https://tauri.app/start/) app with Angular frontend. You will need to follow the instructions to set up your environment to develop tauri Applications.

### Running locally

After checkout:

```bash
npm i
cargo tauri dev
```
