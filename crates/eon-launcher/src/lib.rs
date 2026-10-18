use eon_core::{DesktopModule, ModuleDescriptor};
use freedesktop_desktop_entry::{default_paths, get_languages_from_env, DesktopEntry, Iter};
use serde::{Deserialize, Serialize};
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};

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

#[derive(Clone, Debug)]
pub struct Application {
    pub id: String,
    pub name: String,
    pub icon: Option<PathBuf>,
    command: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct LauncherPreferences {
    pub pinned: Vec<String>,
    pub application_order: Vec<String>,
}

impl LauncherPreferences {
    pub fn load() -> Self {
        preferences_path()
            .map(|path| Self::load_from(&path))
            .unwrap_or_default()
    }

    pub fn load_from(path: &Path) -> Self {
        std::fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> io::Result<()> {
        let path = preferences_path().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "user config directory is unavailable")
        })?;
        self.save_to(&path)
    }

    pub fn save_to(&self, path: &Path) -> io::Result<()> {
        let parent = path.parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "config path has no parent directory")
        })?;
        std::fs::create_dir_all(parent)?;

        let mut temporary_path = path.as_os_str().to_os_string();
        temporary_path.push(".tmp");
        let temporary_path = PathBuf::from(temporary_path);
        let contents = serde_json::to_vec_pretty(self)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

        std::fs::write(&temporary_path, contents)?;
        std::fs::rename(temporary_path, path)
    }

    pub fn retain_installed(&mut self, applications: &[Application]) {
        self.pinned
            .retain(|id| applications.iter().any(|application| &application.id == id));
        self.application_order
            .retain(|id| applications.iter().any(|application| &application.id == id));
        for application in applications {
            if !self
                .application_order
                .iter()
                .any(|id| id == &application.id)
            {
                self.application_order.push(application.id.clone());
            }
        }
    }

    pub fn is_pinned(&self, id: &str) -> bool {
        self.pinned.iter().any(|pinned| pinned == id)
    }

    pub fn toggle_pinned(&mut self, id: &str) {
        if let Some(position) = self.pinned.iter().position(|pinned| pinned == id) {
            self.pinned.remove(position);
        } else {
            self.pinned.push(id.to_owned());
        }
    }

    pub fn pin_before(&mut self, source: &str, target: &str) {
        if source == target {
            return;
        }

        self.pinned.retain(|id| id != source);
        let target_index = self
            .pinned
            .iter()
            .position(|id| id == target)
            .unwrap_or(self.pinned.len());
        self.pinned.insert(target_index, source.to_owned());
    }

    pub fn reorder_before(&mut self, source: &str, target: &str) {
        if source == target {
            return;
        }

        self.application_order.retain(|id| id != source);
        let target_index = self
            .application_order
            .iter()
            .position(|id| id == target)
            .unwrap_or(self.application_order.len());
        self.application_order
            .insert(target_index, source.to_owned());
    }

    pub fn sort_applications(&self, applications: &mut [Application]) {
        applications.sort_by_key(|application| {
            self.application_order
                .iter()
                .position(|id| id == &application.id)
                .map(|position| (0, position, String::new()))
                .unwrap_or_else(|| (1, usize::MAX, application.name.to_lowercase()))
        });
    }
}

fn preferences_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("org", "eon", "launcher")
        .map(|directories| directories.config_dir().join("preferences.json"))
}

impl Application {
    pub fn launch(&self) -> io::Result<Child> {
        let (program, arguments) = self
            .command
            .split_first()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing Exec command"))?;

        Command::new(program).args(arguments).spawn()
    }
}

pub fn discover_applications() -> Vec<Application> {
    let locales = get_languages_from_env();
    let mut applications = Iter::new(default_paths())
        .entries(Some(&locales))
        .filter_map(|entry| application_from_entry(entry, &locales))
        .collect::<Vec<_>>();

    applications.sort_by_key(|application| application.name.to_lowercase());
    applications.dedup_by(|left, right| left.id == right.id);
    applications
}

fn application_from_entry(entry: DesktopEntry, locales: &[String]) -> Option<Application> {
    if entry.hidden()
        || entry.no_display()
        || entry.terminal()
        || entry.type_() != Some("Application")
    {
        return None;
    }

    let name = entry.name(locales)?.into_owned();
    let command = entry.parse_exec().ok()?;
    if command.is_empty() {
        return None;
    }

    let icon = entry.icon().and_then(|icon_name| {
        freedesktop_icons::lookup(icon_name)
            .with_theme("Fluent")
            .with_size(48)
            .with_cache()
            .find()
    });

    Some(Application {
        id: entry.id().to_owned(),
        name,
        icon,
        command,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_entry(contents: &str) -> DesktopEntry {
        let locales = Vec::<String>::new();
        DesktopEntry::from_str("sample.desktop", contents, Some(&locales))
            .expect("desktop entry should parse")
    }

    #[test]
    fn discovers_launchable_application_entry() {
        let entry = parse_entry(
            "[Desktop Entry]\nType=Application\nName=Sample Editor\nExec=/bin/true\nIcon=sample-editor\n",
        );

        let application = application_from_entry(entry, &[]).expect("application should load");

        assert_eq!(application.name, "Sample Editor");
        assert_eq!(application.command, ["/bin/true"]);
    }

    #[test]
    fn skips_hidden_terminal_and_non_application_entries() {
        let entries = [
            "[Desktop Entry]\nType=Application\nName=Hidden\nExec=/bin/true\nHidden=true\n",
            "[Desktop Entry]\nType=Application\nName=Terminal\nExec=/bin/true\nTerminal=true\n",
            "[Desktop Entry]\nType=Link\nName=Website\nURL=https://example.com\n",
        ];

        for contents in entries {
            assert!(application_from_entry(parse_entry(contents), &[]).is_none());
        }
    }

    #[test]
    fn skips_entries_without_an_exec_command() {
        let entry = parse_entry("[Desktop Entry]\nType=Application\nName=No command\n");

        assert!(application_from_entry(entry, &[]).is_none());
    }

    #[test]
    fn pinning_toggles_and_preserves_user_order() {
        let mut preferences = LauncherPreferences::default();
        preferences.toggle_pinned("browser");
        preferences.toggle_pinned("editor");
        preferences.toggle_pinned("browser");

        assert_eq!(preferences.pinned, ["editor"]);
        assert!(!preferences.is_pinned("browser"));
    }

    #[test]
    fn moving_an_application_before_another_updates_grid_order() {
        let mut preferences = LauncherPreferences {
            pinned: Vec::new(),
            application_order: vec!["browser".into(), "editor".into(), "files".into()],
        };
        preferences.reorder_before("files", "editor");

        assert_eq!(
            preferences.application_order,
            ["browser", "files", "editor"]
        );
    }

    #[test]
    fn preferences_round_trip_through_config_file() {
        let path = std::env::temp_dir().join(format!(
            "eon-launcher-preferences-{}.json",
            std::process::id()
        ));
        let preferences = LauncherPreferences {
            pinned: vec!["browser".into()],
            application_order: vec!["editor".into(), "browser".into()],
        };

        preferences.save_to(&path).expect("preferences should save");
        assert_eq!(LauncherPreferences::load_from(&path), preferences);
        std::fs::remove_file(path).expect("temporary preferences should be removed");
    }

    #[test]
    fn retained_application_order_can_be_reordered_and_sorted() {
        let mut applications = vec![
            Application {
                id: "browser".into(),
                name: "Browser".into(),
                icon: None,
                command: vec!["/bin/true".into()],
            },
            Application {
                id: "editor".into(),
                name: "Editor".into(),
                icon: None,
                command: vec!["/bin/true".into()],
            },
            Application {
                id: "files".into(),
                name: "Files".into(),
                icon: None,
                command: vec!["/bin/true".into()],
            },
        ];
        let mut preferences = LauncherPreferences::default();
        preferences.retain_installed(&applications);
        preferences.reorder_before("files", "editor");
        preferences.sort_applications(&mut applications);

        assert_eq!(
            applications
                .iter()
                .map(|application| application.id.as_str())
                .collect::<Vec<_>>(),
            ["browser", "files", "editor"]
        );
    }

    #[test]
    fn moving_a_pin_before_another_updates_dock_order() {
        let mut preferences = LauncherPreferences {
            pinned: vec!["browser".into(), "files".into(), "editor".into()],
            application_order: Vec::new(),
        };
        preferences.pin_before("editor", "browser");

        assert_eq!(preferences.pinned, ["editor", "browser", "files"]);
    }
}
