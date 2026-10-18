use eon_core::{DesktopModule, ModuleDescriptor};

pub struct PowerModule;

impl DesktopModule for PowerModule {
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor {
            id: "power",
            name: "Power controls",
            responsibility: "Request lock, suspend, reboot, and shutdown through system services",
        }
    }
}
