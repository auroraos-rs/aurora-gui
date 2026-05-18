# demo

Demo application for [`aurora_egui`](../aurora_egui/).

A simple [egui](https://github.com/emilk/egui) app demonstrating window properties, cover page rendering, system font integration, and environment variable inspection. Runs on both Aurora OS devices and desktop Linux.

This is the reference example for the egui integration crate. Additional demo apps for other GUI libraries may be added to the workspace in the future.

## Pages

### Counter

The default page showing:
- A greeting label
- An increment/decrement counter
- The current egui pixel ratio
- Settings toggles (status bar, background visibility)

### Env Variables

A scrollable, alphabetically sorted list of all environment variables displayed in a monospace grid.

## Cover Page

When the app is backgrounded on Aurora OS, the cover page shows:
- The app title in large text
- The current counter value
- The current pixel ratio

Two cover actions are registered (`reset` and `add`) with Aurora theme icons.

## Build & Run

```bash
# Native desktop build
cargo run -p demo

# Release builds for Aurora OS
cross build --release -p demo --target aarch64-unknown-linux-gnu
cross build --release -p demo --target armv7-unknown-linux-gnueabihf
```

## RPM Packaging

The crate includes `[package.metadata.generate-rpm]` metadata. Build scripts at the workspace root produce signed RPMs:

```bash
./aarch64_build.sh
./arm_build.sh
```

## License

Apache 2.0
