# eon-shell

Persistent Iced launcher hosted as a Wayland layer-shell surface through `iced_layershell`, with namespace `eon-launcher` for Niri layer rules. It starts as a narrow dock on the left and expands into the application grid from the dock button. Escape or launching an app returns the same surface to dock mode. Run it inside an existing Niri session; it does not implement a compositor or start a graphical session. Use `--list-modules` to inspect the registered EON modules without opening the UI.

The launcher uses translucent surfaces. Backdrop blur is not rendered by the app and depends on compositor-specific configuration.
