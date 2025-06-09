use crate::utils;
use adw::subclass::prelude::ObjectSubclassIsExt;
use gtk::gio;
use gtk::glib::object::ObjectExt;
use gtk::glib::{self, Object};
use gtk::prelude::{GtkWindowExt, TextBufferExt, TextViewExt};
use gtk::subclass::prelude::{ObjectImpl, ObjectSubclass, ObjectSubclassExt};
use gtk::template_callbacks;
use gtk::CompositeTemplate;
use std::cell::{Ref, RefCell};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{fs, io};

mod inner {

    use std::{cell::Cell, env, path::PathBuf, str::FromStr};

    use super::*;
    use adw::subclass::{bin::BinImpl, prelude::ObjectImplExt};
    use gtk::{
        glib::{derived_properties, Properties},
        subclass::{prelude::*, widget::WidgetImpl, window::WindowImpl},
    };
    #[derive(CompositeTemplate, Default, Properties, Debug)]
    #[properties(wrapper_type = super::Editor)]
    #[template(resource = "/io/github/bracket/editor.ui")]
    pub struct Editor {
        #[template_child(id = "editor-text-buffer")]
        pub buffer: TemplateChild<gtk::TextBuffer>,

        pub path: RefCell<Option<PathBuf>>,
        #[property(get, set)]
        pub filename: RefCell<String>,
        #[property(get, set, default = true)]
        pub saved: Cell<bool>,
    }
    #[template_callbacks]
    impl Editor {
        #[template_callback]
        fn on_changed(&self, buffer: &gtk::TextBuffer) {
            self.obj().set_saved(false);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Editor {
        const NAME: &'static str = "Editor";
        type Type = super::Editor;
        type ParentType = gtk::TextView;
        fn new() -> Self {
            Self {
                filename: RefCell::new("Untitled".to_string()),
                path: RefCell::new(None),
                ..Default::default()
            }
        }
        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[derived_properties]
    impl ObjectImpl for Editor {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_saved(true);
        }
    }
    impl TextViewImpl for Editor {}
    impl WidgetImpl for Editor {}
}

glib::wrapper! {
    pub struct Editor(ObjectSubclass<inner::Editor>)
    @extends gtk::TextView, gtk::Widget,
    @implements gtk::Accessible, gtk::AccessibleText, gtk::Buildable, gtk::ConstraintTarget, gtk::Scrollable;
}

impl Editor {
    pub fn new() -> Self {
        let obj = Object::new();
        // inner::Editor::from_obj(&obj)
        //     .working_dir
        //     .replace(Some(dir));
        obj
    }
    pub fn get_text(&self) -> glib::GString {
        let buffer = self.buffer();
        let (start, end) = buffer.bounds();
        buffer.text(&start, &end, false)
    }
    pub fn open_file_path<P>(&self, path: P) -> Result<(), io::Error>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let content = fs::read_to_string(path)?;
        self.buffer().set_text(&content);
        if let Some(name) = path.file_name().and_then(|f| f.to_str()) {
            self.set_filename(name);
        }

        Ok(())
    }
    pub fn set_file<P>(&self, path: P) -> Result<(), io::Error>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref().canonicalize()?;

        let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        self.set_filename(filename);
        self.imp().path.replace(Some(path));

        Ok(())
    }
    pub fn write_to_file(&self) -> Result<(), io::Error> {
        let path = self.imp().path.borrow();

        if let Some(path) = path.as_ref() {
            let content = self.get_text();
            fs::write(path, content)?;
        } else {
            todo!()
        }

        self.set_saved(true);
        Ok(())
    }
    pub fn save_file(&self) {
        let editor = self.clone();
        glib::MainContext::default().spawn_local(async move {
            editor.save_file_fut().await;
        });
    }

    pub async fn save_file_fut(&self) {
        let path = { self.imp().path.borrow().is_none() } ;
        if path {
            self.save_file_as_fut().await;
            return;
        }

        if let Err(e) = self.write_to_file() {
            eprintln!("Error saving file: {e}")
        }
    }

    pub fn save_file_as(&self) {
        let editor = self.clone();
        glib::MainContext::default().spawn_local(async move {
            editor.save_file_as_fut().await;
        });
    }
    async fn save_file_as_fut(&self) {
        let path = match utils::request_save_file().await {
            Ok(path) => path,
            Err(e) => {
                eprintln!("error opening file {e}");
                return;
            }
        };

        if let Err(e) = self.set_file(path) {
            eprintln!("Error saving file: {e}")
        }
        if let Err(e) = self.write_to_file() {
            eprintln!("Error saving file: {e}")
        }
    }
}
