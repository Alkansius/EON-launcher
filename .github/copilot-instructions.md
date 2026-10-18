# Workspace Setup

- [x] Verify that `.github/copilot-instructions.md` exists.
- [x] Clarify project requirements: modular Rust desktop environment for GNU/Linux, based on Wayland and PipeWire.
- [x] Scaffold the Cargo workspace in the current directory.
- [x] Customize the project architecture and initial executable.
- [x] Install required extensions: none specified.
- [ ] Compile and validate the workspace: blocked because `cargo` is not installed.
- [x] Create and run a task: skipped because no concrete development task is needed yet.
- [x] Launch the project: no graphical session started; explicit user confirmation is required.
- [x] Ensure README and workspace instructions document the current project.

## Project Constraints

- Keep desktop features modular and expose clear boundaries between crates.
- Initially integrate with an existing Wayland compositor such as Niri or LabWC; do not claim to implement a compositor.
- Keep audio integration based on PipeWire and isolate it from shell UI and session management.
- Prefer Linux-native protocols and document system dependencies.
- Do not start a graphical session or alter system configuration without explicit confirmation.
