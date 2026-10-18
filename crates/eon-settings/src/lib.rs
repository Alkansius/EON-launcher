use eon_core::{DesktopModule, ModuleDescriptor};

pub struct SettingsModule;

impl DesktopModule for SettingsModule {
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor {
            id: "settings",
            name: "System settings",
            responsibility: "Expose desktop and supported system preferences",
        }
    }
}
