use eon_core::{DesktopModule, ModuleDescriptor};

mod ui;

fn main() {
    if std::env::args().any(|argument| argument == "--list-modules") {
        list_modules();
        return;
    }

    if let Err(error) = ui::run() {
        eprintln!("EON shell could not start: {error}");
    }
}

fn list_modules() {
    let modules: [&dyn DesktopModule; 6] = [
        &eon_launcher::LauncherModule,
        &eon_settings::SettingsModule,
        &eon_power::PowerModule,
        &eon_status::StatusModule,
        &eon_notifications::NotificationsModule,
        &eon_audio::AudioModule,
    ];

    println!("EON desktop environment modules:");

    for module in modules {
        print_module(module.descriptor());
    }
}

fn print_module(module: ModuleDescriptor) {
    println!("  {} ({}) - {}", module.name, module.id, module.responsibility);
}
