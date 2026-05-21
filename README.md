# aurora-gui

A GUI toolkit for [Aurora OS](https://auroraos.ru/).

This workspace provides crates for building GUI apps that run natively on Aurora OS (Sailfish-derived Linux) and also work on desktop Linux for development. While the current focus is on [egui](https://github.com/emilk/egui) support, the workspace is structured to accommodate additional GUI libraries in the future (e.g. `aurora_iced`).

## Workspace Crates

| Crate | Purpose |
|-------|---------|
| [`aurora_app`](aurora_app/) | GUI-agnostic window property helpers for Aurora OS (Wayland generic properties, cover window linkage) |
| [`aurora_egui`](aurora_egui/) | egui integration — runs an egui app with GL rendering, cover page, and system font support |
| [`demo`](demo/) | Example egui application demonstrating all features |

## Quick Start

```bash
# Native dev build
cargo build -p demo

# Cross-compile for Aurora OS devices
cross build --release -p demo --target aarch64-unknown-linux-gnu
cross build --release -p demo --target armv7-unknown-linux-gnueabihf

# Run tests
cargo test --workspace
```

## Dependencies & Patches

The workspace patches `winit` and `glutin` with custom forks required for Aurora OS compatibility:

- `winit` → [`lmaxyz/winit@rm_maliit`](https://github.com/lmaxyz/winit/tree/rm_maliit)
- `glutin` → [`lmaxyz/glutin@aurora_device_fix`](https://github.com/lmaxyz/glutin/tree/aurora_device_fix)

Do not upgrade these to crates.io versions without verifying Aurora OS compatibility.

## RPM Packaging

The `demo` crate includes `[package.metadata.generate-rpm]` in its `Cargo.toml`. To build a signed RPM:

```bash
# Requires Aurora Platform SDK (PSDK_DIR environment variable)
./aarch64_build.sh
./arm_build.sh
```

Output RPMs are placed in `./RPMS/`.

## License

Apache 2.0
