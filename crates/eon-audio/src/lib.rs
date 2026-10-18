use eon_core::{DesktopModule, ModuleDescriptor};

pub struct AudioModule;

impl DesktopModule for AudioModule {
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor {
            id: "audio",
            name: "Audio integration",
            responsibility: "Control audio devices and routing through PipeWire",
        }
    }
}
