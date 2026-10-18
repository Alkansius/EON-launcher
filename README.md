# EON Desktop

A modular Rust desktop-environment prototype for GNU/Linux. The initial version registers independent desktop modules and deliberately relies on an existing Wayland compositor; it is not a compositor and does not start a graphical session.

## Current state

The workspace contains separate crates for the module contract, shell host, application launcher, system settings, power controls, status indicators, notifications/calendar, and PipeWire audio integration. `eon-shell` now opens an Iced launcher as a Wayland layer-shell overlay, and `eon-launcher` discovers and launches XDG desktop applications. The other service crates remain integration boundaries rather than fully implemented controls.

## Architecture

- `eon-core`: shared module descriptor and compositor backend types.
- `eon-shell`: executable host that composes the feature crates.
- `eon-launcher`: application search and launch boundary.
- `eon-settings`: desktop/system preference boundary.
- `eon-power`: power and session action boundary.
- `eon-status`: clock, date, battery, network, and session indicators boundary.
- `eon-notifications`: notification and calendar boundary.
- `eon-audio`: audio device/routing boundary; intended to use PipeWire independently from shell UI and session management.

The initial target is to run as a Wayland client under an existing compositor such as Niri or LabWC. The launcher uses the `wlr-layer-shell` protocol through `iced_layershell`; it does not implement a compositor or start a graphical session. The UI requests a transparent surface and draws translucent panels. Actual backdrop blur is compositor-controlled and must be enabled/configured by the user where supported. Compositor control, shell UI, and audio routing remain separate concerns.

## Niri setup

Run EON from an existing Niri session. To start the persistent left dock and enable a blur rule without replacing your Niri config, add this top-level include to `~/.config/niri/config.kdl`:

```kdl
include "/absolute/path/to/EON-launcher/config/niri/eon.kdl"
```

The included profile starts EON as a persistent dock and configures Niri's background blur for the `eon-launcher` layer. Change the path to the checkout location. Use the grid button at the bottom of the dock to open the launcher; Escape or launching an app returns to the dock. Reload Niri's config after adding the include. Background effects require a Niri version that supports `background-effect` layer rules and a compatible graphics stack.

## Prerequisites

- Rust toolchain (`rustc` and `cargo`), preferably installed through rustup.
- GNU/Linux development packages `pkg-config` and `libxkbcommon-dev` (package names vary by distribution).
- An existing Niri session with `wlr-layer-shell` support.
- For the graphical launcher: an existing Wayland session with `wlr-layer-shell` support. Niri and LabWC are target compositors; other compositors may not provide the required protocol.
- For actual background blur: compositor-specific blur support and configuration. The application itself does not capture or blur other windows.

Cargo resolves the Iced, layer-shell, desktop-entry, icon lookup, and clock dependencies. No system configuration is changed by building or launching the program.

## Build and run

```sh
cargo check --workspace
cargo run -p eon-shell
cargo run -p eon-shell -- --list-modules
```

The default command opens the graphical launcher in the current Wayland session. Press Escape to dismiss it; successfully launching an application closes it as well. `--list-modules` prints the workspace module registry without opening a window. Applications with a valid `Exec` command are launched directly without a shell; terminal-only and hidden entries are omitted. D-Bus-only entries are not launched yet. The shell does not start a graphical session or control power/audio devices.

Application icons are resolved from the installed `Fluent` XDG icon theme and rendered as SVG. Install the upstream [Fluent icon theme](https://github.com/vinceliuice/Fluent-icon-theme) into a user icon directory (default: `~/.local/share/icons`):

```sh
git clone https://github.com/vinceliuice/Fluent-icon-theme.git
cd Fluent-icon-theme
./install.sh
```

If an icon or theme is unavailable, the launcher displays the application's initial instead. Theme assets are not vendored into this repository.

The launcher saves pinned application IDs and grid order in `~/.config/eon/launcher/preferences.json` (or the active `XDG_CONFIG_HOME`). Use the star on an app tile to pin or unpin it, drag tiles to reorder the grid, drag a tile to the pinned panel to pin it, and drag a pinned app back onto the grid to unpin it. Changes are saved immediately.

## Roadmap

1. Add compositor/session detection while retaining Niri/LabWC as the initial compositor.
2. Add D-Bus activation for desktop entries without `Exec`.
3. Implement settings/power integrations through Linux desktop services.
4. Add notification and calendar views plus system indicators.
5. Implement the isolated PipeWire audio adapter and validate it against a running PipeWire session.
6. Add packaging, session integration, and graphical end-to-end checks.
