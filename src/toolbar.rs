use gtk::gio;
use gtk::glib::object::ObjectExt;
use gtk::glib::{self, Object};
use gtk::prelude::GtkWindowExt;
use gtk::subclass::prelude::{ObjectImpl, ObjectSubclass, ObjectSubclassExt};
use gtk::CompositeTemplate;
use std::cell::{Ref, RefCell};
use std::path::PathBuf;
use std::time::Duration;

use crate::utils;

mod inner {

    use adw::subclass::{bin::BinImpl, prelude::ObjectImplExt};
    use gtk::{
        subclass::{prelude::*, widget::WidgetImpl, window::WindowImpl},
        PopoverMenuBar,
    };

    use super::*;
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/io/github/bracket/toolbar.ui")]
    pub struct EditorToolBar {
        #[template_child(id = "menubar")]
        pub menubar: TemplateChild<gtk::PopoverMenuBar>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EditorToolBar {
        const NAME: &'static str = "EditorToolBar";
        type Type = super::EditorToolBar;
        type ParentType = adw::Bin;

        fn new() -> Self {
            Self {
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

    impl ObjectImpl for EditorToolBar {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = &self.obj();

        }
    }
    impl BinImpl for EditorToolBar {}
    impl WidgetImpl for EditorToolBar {}
}

glib::wrapper! {
    pub struct EditorToolBar(ObjectSubclass<inner::EditorToolBar>)
    @extends gtk::Widget, adw::Bin,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl EditorToolBar {
    pub fn new() -> Self {
        let obj = Object::new();
        // inner::EditorToolBar::from_obj(&obj)
        //     .working_dir
        //     .replace(Some(dir));
        obj
    }
}
