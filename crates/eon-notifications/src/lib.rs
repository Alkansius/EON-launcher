use eon_core::{DesktopModule, ModuleDescriptor};

pub struct NotificationsModule;

impl DesktopModule for NotificationsModule {
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor {
            id: "notifications",
            name: "Notifications and calendar",
            responsibility: "Collect desktop notifications and provide date/calendar views",
        }
    }
}
