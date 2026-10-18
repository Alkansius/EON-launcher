#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModuleDescriptor {
    pub id: &'static str,
    pub name: &'static str,
    pub responsibility: &'static str,
}

pub trait DesktopModule {
    fn descriptor(&self) -> ModuleDescriptor;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompositorBackend {
    Niri,
    Labwc,
}

impl CompositorBackend {
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Niri => "Niri",
            Self::Labwc => "LabWC",
        }
    }
}
