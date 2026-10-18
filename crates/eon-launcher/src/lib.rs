use eon_core::{DesktopModule, ModuleDescriptor};

pub struct LauncherModule;

impl DesktopModule for LauncherModule {
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor {
            id: "launcher",
            name: "Application launcher",
            responsibility: "Search and start installed desktop applications",
        }
    }
}
