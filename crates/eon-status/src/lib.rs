use eon_core::{DesktopModule, ModuleDescriptor};

pub struct StatusModule;

impl DesktopModule for StatusModule {
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor {
            id: "status",
            name: "Status indicators",
            responsibility: "Present time, date, battery, network, and session state",
        }
    }
}
