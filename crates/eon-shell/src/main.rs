use eon_core::{DesktopModule, ModuleDescriptor};

fn main() {
    let modules: [&dyn DesktopModule; 6] = [
        &eon_launcher::LauncherModule,
        &eon_settings::SettingsModule,
        &eon_power::PowerModule,
        &eon_status::StatusModule,
        &eon_notifications::NotificationsModule,
        &eon_audio::AudioModule,
    ];

    println!("EON desktop environment | modular shell prototype");
    println!("Compositor: use an existing Wayland session (Niri or LabWC)");
    println!("\nModules:");

    for module in modules {
        print_module(module.descriptor());
    }
}

fn print_module(module: ModuleDescriptor) {
    println!("  {} ({}) - {}", module.name, module.id, module.responsibility);
}
