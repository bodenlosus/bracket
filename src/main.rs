mod window;
mod dialogs;
mod editor;
mod tabview;
mod toolbar;
mod utils;

use adw::prelude::*;
use gtk::gio;
use gtk::glib;
use std::fmt::format;
use std::path::Path;
const APP_ID: &'static str = "io.github.bracket";
const DATA_DIR: &'static str = "/home/johannes/bracket/data/";

fn main() -> () {
    load_resources();

    let app = adw::Application::new(Some(APP_ID), gio::ApplicationFlags::FLAGS_NONE);
    app.connect_startup(startup);
    app.connect_activate(build_ui);
    app.run();
}


fn build_ui(app: &adw::Application) {
    let window = window::Window::new(app, None);
    window.present();
}

fn startup(app: &adw::Application) {
    if let Err(e) = load_configs(app) {
        eprintln!("{e}");
    }
}
fn load_configs(app: &adw::Application) -> Result<(), Box<dyn std::error::Error>> {
    let config_dir =
        utils::get_app_config_dir().ok_or(std::io::Error::new(std::io::ErrorKind::NotFound, "config dir not found"))?;
    {
        let keybinds_path = &config_dir.join("keymap.json");
        if keybinds_path.is_file() {
            match utils::KeyboardShortcuts::from_file(keybinds_path) {
                Ok(s) => {
                    s.apply_accels_to_app(app);
                }
                Err(e) => eprintln!("Error reading config file for keybinds: {e}"),
            }
        } else {
            eprintln!("keybind file not found");
        }
    }
    Ok(())
}

fn load_resources() {
    let data_dir_path = Path::new(DATA_DIR);
    let resource_path = data_dir_path
        .join("io.github.bracket.gresource")
        .canonicalize()
        .expect("Resource path is not valid");

    let resource = gio::Resource::load(resource_path).expect("Could not load resource");
    gio::resources_register(&resource);
}
