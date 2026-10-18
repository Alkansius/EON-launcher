# EON Desktop

A modular Rust desktop-environment prototype for GNU/Linux. The initial version registers independent desktop modules and deliberately relies on an existing Wayland compositor; it is not a compositor and does not start a graphical session.

## Current state

The workspace contains separate crates for the module contract, shell host, application launcher, system settings, power controls, status indicators, notifications/calendar, and PipeWire audio integration. The initial `eon-shell` executable currently prints the registered modules. Graphical surfaces, desktop-protocol clients, settings backends, and PipeWire control are roadmap work, not implemented features.

## Architecture

- `eon-core`: shared module descriptor and compositor backend types.
- `eon-shell`: executable host that composes the feature crates.
- `eon-launcher`: application search and launch boundary.
- `eon-settings`: desktop/system preference boundary.
- `eon-power`: power and session action boundary.
- `eon-status`: clock, date, battery, network, and session indicators boundary.
- `eon-notifications`: notification and calendar boundary.
- `eon-audio`: audio device/routing boundary; intended to use PipeWire independently from shell UI and session management.

The initial target is to run as a Wayland client under an existing compositor such as Niri or LabWC. A graphical shell surface can later use the Wayland layer-shell protocol where supported. Compositor control, shell UI, and audio routing remain separate concerns.

## Prerequisites

- Rust toolchain (`rustc` and `cargo`), preferably installed through rustup.
- GNU/Linux for eventual Wayland and PipeWire integration.
- For graphical integration in a later milestone: a Wayland session/compositor and development libraries required by the selected UI toolkit, layer-shell protocol implementation, and PipeWire bindings.

The current prototype has no third-party Rust dependencies or native library requirements.

## Build and run

```sh
cargo check --workspace
cargo run -p eon-shell
```

The current executable is a CLI module-registry check. It does not launch a desktop, alter system configuration, or control power/audio devices.

## Roadmap

1. Choose a Rust UI toolkit and implement the single-window launcher/dashboard.
2. Add Wayland layer-shell surfaces and compositor/session detection while retaining Niri/LabWC as the initial compositor.
3. Implement app discovery through XDG desktop-entry conventions and settings/power integrations through Linux desktop services.
4. Add notification and calendar views plus system indicators.
5. Implement the isolated PipeWire audio adapter and validate it against a running PipeWire session.
6. Add packaging, session integration, and graphical end-to-end checks.
