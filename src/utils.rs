use std::{
    collections::HashMap,
    error::Error,
    fmt::{write, Display},
    fs::{self, OpenOptions},
    io,
    path::{Path, PathBuf},
    str::FromStr,
};

pub fn get_dir_with_fallback() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| std::env::home_dir().expect("Could not retrieve working dir"))
}

pub fn touch_file<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    OpenOptions::new()
        .create(true)
        .append(true) // Won't truncate existing files
        .open(path)?;
    Ok(())
}

use adw::prelude::{AlertDialogExt, AlertDialogExtManual};
use cascade::cascade;
use gtk::{gio::prelude::FileExt, glib::object::IsA, prelude::GtkApplicationExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    #[serde(rename = "win.new-file")]
    NewFile,
    #[serde(rename = "win.save-file")]
    SaveFile,
    #[serde(rename = "win.save-file-as")]
    SaveFileAs,
    #[serde(rename = "win.open-file")]
    OpenFile,
    #[serde(rename = "win.close-file")]
    CloseFile,
}

impl FromStr for Action {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "win.new-file" => Ok(Action::NewFile),
            "win.save-file" => Ok(Action::SaveFile),
            "win.save-file-as" => Ok(Action::SaveFileAs),
            "win.open-file" => Ok(Action::OpenFile),
            "win.close-file" => Ok(Action::CloseFile),
            _ => Err(format!("Invalid action: {}", s)),
        }
    }
}

impl Action {
    pub fn name(&self) -> &'static str {
        match self {
            Action::NewFile => "new-file",
            Action::SaveFile => "save-file",
            Action::SaveFileAs => "save-file-as",
            Action::OpenFile => "open-file",
            Action::CloseFile => "close-file",
        }
    }
    pub fn to_str(&self) -> &'static str {
        match self {
            Action::NewFile => "win.new-file",
            Action::SaveFile => "win.save-file",
            Action::SaveFileAs => "win.save-file-as",
            Action::OpenFile => "win.open-file",
            Action::CloseFile => "win.close-file",
        }
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.to_str();
        write!(f, "{}", s)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct KeyCombo(String);

impl KeyCombo {
    fn test_valid(combo: &str) -> bool {
        let Some((key, modifiers)) = gtk::accelerator_parse(combo) else {
            return false;
        };
        gtk::accelerator_valid(key, modifiers)
    }
    pub fn is_valid(&self) -> bool {
        Self::test_valid(&self.0)
    }
}

impl Display for KeyCombo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for KeyCombo {
    fn as_ref(&self) -> &str {
        &self.0.as_str()
    }
}

impl From<Action> for gtk::gio::SimpleAction {
    fn from(value: Action) -> Self {
        Self::new(value.name(), None)
    }
}

#[derive(Debug, Serialize, PartialEq, Deserialize)]
pub struct KeyboardShortcuts {
    #[serde(flatten)]
    pub shortcuts: HashMap<Action, KeyCombo>,
}

impl KeyboardShortcuts {
    pub fn from_file<P>(path: P) -> Result<Self, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let content = fs::read_to_string(path)?;
        let shortcuts = serde_json::from_str::<KeyboardShortcuts>(&content)?;
        Self::validate(&shortcuts.shortcuts)?;
        Ok(shortcuts)
    }
    fn validate(shortcuts: &HashMap<Action, KeyCombo>) -> Result<(), String> {
        for (action, combo) in shortcuts {
            if !combo.is_valid() {
                return Err(format!("Invalid Keybind: {}: {}", action, combo));
            }
        }
        Ok(())
    }
    pub fn is_valid(&self) -> Result<(), String> {
        Self::validate(&self.shortcuts)
    }

    pub fn apply_accels_to_app(&self, app: &adw::Application) -> () {
        self.shortcuts
            .iter()
            .for_each(|(a, k)| app.set_accels_for_action(&a.to_string(), &[k.as_ref()]))
    }
}

pub fn get_app_config_dir() -> Option<PathBuf> {
    let xdg_config_dir = std::env::var("XDG_CONFIG_HOME")
        .ok()
        .and_then(|s| PathBuf::from_str(&s).ok())
        .or_else(|| std::env::home_dir().map(|p| p.join(".config")));
    xdg_config_dir.map(|p| p.join("bracket"))
}

pub async fn request_open_file() -> Result<PathBuf, Box<dyn Error>> {
    let dialog = gtk::FileDialog::new();
    let path = dialog
        .open_future(None::<&adw::ApplicationWindow>)
        .await?
        .path()
        .ok_or("no path given for file")?;
    Ok(path)
}
pub async fn request_save_file() -> Result<PathBuf, Box<dyn Error>> {
    let dialog = gtk::FileDialog::new();
    let path = dialog
        .save_future(None::<&adw::ApplicationWindow>)
        .await?
        .path()
        .ok_or("no path given for file")?;
    Ok(path)
}

pub enum UnsavedResponses {
    Save,
    Discard,
    Cancel,
}

impl AsRef<str> for UnsavedResponses {
    fn as_ref(&self) -> &str {
        match self {
            &UnsavedResponses::Cancel => "cancel",
            &UnsavedResponses::Discard => "discard",
            &UnsavedResponses::Save => "save",
        }
    }
}

impl Display for UnsavedResponses {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            &UnsavedResponses::Cancel => "cancel",
            &UnsavedResponses::Discard => "discard",
            &UnsavedResponses::Save => "save",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseUnsavedResponseError;

impl FromStr for UnsavedResponses {
    type Err = ParseUnsavedResponseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "discard" => Ok(UnsavedResponses::Discard),
            "cancel" => Ok(UnsavedResponses::Cancel),
            "save" => Ok(UnsavedResponses::Save),
            &_ => Err(ParseUnsavedResponseError),
        }
    }
}

pub async fn prompt_unsaved_changes(
    name: Option<&str>,
    path: Option<&str>,
) -> Result<UnsavedResponses, ParseUnsavedResponseError>{
    let heading = name.map_or("Save changes?".to_string(), |n| {
        format!("Save changes to {}", n)
    });
    let body = path.map_or(
        "The file contains unsaved changes. Do you wish to save them?".to_string(),
        |p| {
            format!(
                "The file <b>{}</b> contains unsaved changes. Do you wish to save them?",
                p
            )
        },
    );

    let dialog = adw::AlertDialog::builder()
        .heading(heading)
        .body(body)
        .body_use_markup(true)
        .prefer_wide_layout(true)
        .default_response(UnsavedResponses::Save.as_ref())
        .close_response(UnsavedResponses::Cancel.as_ref())
        .build();

    dialog.add_responses(&[
        (UnsavedResponses::Cancel.as_ref(), "Cancel"),
        (UnsavedResponses::Discard.as_ref(), "Discard"),
        (UnsavedResponses::Save.as_ref(), "Save"),
    ]);
    dialog.set_response_appearance(UnsavedResponses::Discard.as_ref(), adw::ResponseAppearance::Destructive);
    dialog.set_response_appearance(UnsavedResponses::Save.as_ref(), adw::ResponseAppearance::Suggested);
    
    let window = gtk::Window::builder().decorated(false).build();

    let response = dialog.choose_future(&window).await;

    UnsavedResponses::from_str(&response)
}
