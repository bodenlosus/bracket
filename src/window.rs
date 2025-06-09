use adw::subclass::prelude::ObjectSubclassIsExt;
use gtk::gio;
use gtk::gio::prelude::{ActionExt, ActionGroupExt, ActionMapExt, FileExt};
use gtk::glib::object::ObjectExt;
use gtk::glib::{self, Object};
use gtk::prelude::GtkWindowExt;
use gtk::subclass::prelude::{ObjectImpl, ObjectSubclass, ObjectSubclassExt};
use gtk::CompositeTemplate;
use std::cell::{Ref, RefCell};
use std::error::Error;
use std::path::PathBuf;
use std::time::Duration;

use crate::tabview::EditorTabView;
use crate::utils::{self, Action};

mod inner {

    use std::{cell::Cell, env, path::PathBuf};

    use adw::subclass::{application_window::AdwApplicationWindowImpl, prelude::ObjectImplExt};
    use gtk::subclass::{prelude::*, widget::WidgetImpl, window::WindowImpl};

    use crate::toolbar::EditorToolBar;

    use super::*;
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/io/github/bracket/window.ui")]
    pub struct Window {
        #[template_child(id = "editor-tabview")]
        pub tabview: TemplateChild<EditorTabView>,
        #[template_child(id = "editor-toolbar")]
        pub toolbar: TemplateChild<EditorToolBar>,
        pub working_dir: Cell<Option<PathBuf>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "Window";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;

        fn new() -> Self {
            Self {
                working_dir: Cell::new(None),
                ..Default::default()
            }
        }
        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            obj.setup_actions();
        }
    }
    impl WidgetImpl for Window {}
    impl WindowImpl for Window {}
    impl ApplicationWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}
}

glib::wrapper! {
    pub struct Window(ObjectSubclass<inner::Window>)
    @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
    @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &adw::Application, working_dir: Option<PathBuf>) -> Self {
        let dir = working_dir.unwrap_or_else(utils::get_dir_with_fallback);
        let obj = Object::new();
        inner::Window::from_obj(&obj).working_dir.replace(Some(dir));
        obj.set_application(Some(app));
        obj
    }
    pub fn setup_actions(&self) {
        use utils::Action;

        self.add_simple_action(Action::NewFile, Self::on_new);
        self.add_simple_action(Action::OpenFile, Self::on_open);
        self.add_simple_action(Action::SaveFile, Self::on_save);
        self.add_simple_action(Action::SaveFileAs, Self::on_save_as);
        self.add_simple_action(Action::CloseFile, Self::on_close);
    }
    fn add_simple_action<T, F>(&self, action: T, handler: F)
    where
        gio::SimpleAction: From<T>,
        F: Fn(&Self) + 'static,
    {
        let action: gio::SimpleAction = action.into();
        action.connect_activate(glib::clone!(
            #[weak(rename_to=w)]
            self,
            move |_, _| handler(&w)
        ));
        self.add_action(&action);
    }
    fn on_new(&self) {
        self.imp().tabview.new_file(true);
    }
    fn on_open(&self) {
        let tabview = self.imp().tabview.clone();

        glib::MainContext::default().spawn_local(async move {
            let path = match utils::request_open_file().await {
                Ok(path) => path,
                Err(e) => {
                    eprintln!("error opening file {e}");
                    return;
                }
            };

            if let Err(e) = tabview.open_file(&path, true) {
                eprintln!("Error opening file: {e}")
            }
        });
        println!("a");
    }
    fn on_save(&self) {
        let tabview = self.imp().tabview.clone();
        if let Some(tabview) = tabview.get_active_editor() {
            tabview.save_file();
        }
    }
    fn on_save_as(&self) {
        let tabview = self.imp().tabview.clone();
        if let Some(tabview) = tabview.get_active_editor() {
            tabview.save_file_as();
        }
    }
    fn on_close(&self) {
        println!("a");
    }
}
