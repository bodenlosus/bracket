use adw::subclass::prelude::ObjectSubclassIsExt;
use gtk::gio;
use gtk::glib::object::{CastNone, ObjectExt};
use gtk::glib::{self, derived_properties, Object};
use gtk::prelude::{GtkWindowExt, WidgetExt};
use gtk::subclass::prelude::{ObjectImpl, ObjectSubclass, ObjectSubclassExt};
use gtk::template_callbacks;
use gtk::CompositeTemplate;
use std::cell::{Ref, RefCell};
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use crate::editor::Editor;
use crate::utils;

mod inner {

    use std::{cell::Cell, env, path::PathBuf};

    use adw::{
        subclass::{bin::BinImpl, prelude::ObjectImplExt},
        TabView,
    };
    use gtk::{
        glib::object::Cast,
        subclass::{prelude::*, widget::WidgetImpl, window::WindowImpl},
    };

    use crate::{editor, tabview, utils::UnsavedResponses};

    use super::*;
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/io/github/bracket/tabview.ui")]
    pub struct EditorTabView {
        #[template_child(id = "tabview")]
        pub tabview: TemplateChild<adw::TabView>,
        #[template_child(id = "tabbar")]
        pub tabbar: TemplateChild<adw::TabBar>,
    }
    #[template_callbacks]
    impl EditorTabView {
        #[template_callback]
        fn on_close(&self, page: &adw::TabPage, tabview: &adw::TabView) -> bool {
            let Ok(editor) = page.child().downcast::<Editor>() else {
                tabview.close_page_finish(page, false);
                return true;
            };

            if editor.saved() {
                println!("saved");
                tabview.close_page_finish(page, true);
                return true;
            }

            {
                let tabview = tabview.clone();
                let page = page.clone();
                let fut = async move {
                    let response = utils::prompt_unsaved_changes(None, None)
                        .await
                        .expect("failed to parse response of unsaved dialog.");

                    match response {
                        UnsavedResponses::Cancel => {
                            tabview.close_page_finish(&page, false);
                        },
                        UnsavedResponses::Discard => {
                            tabview.close_page_finish(&page, true);
                        },
                        UnsavedResponses::Save => {
                            if let Some(editor) = page.child().downcast_ref::<Editor>() {
                                editor.save_file_fut().await;
                            }
                            tabview.close_page_finish(&page, true);
                        }
                    }
                };
                glib::MainContext::default().spawn_local(fut);
            }


            return true;
        }
    }
    #[glib::object_subclass]
    impl ObjectSubclass for EditorTabView {
        const NAME: &'static str = "EditorTabView";
        type Type = super::EditorTabView;
        type ParentType = adw::Bin;

        fn new() -> Self {
            Self {
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
    impl ObjectImpl for EditorTabView {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            let imp = obj.imp();
            imp.tabbar.set_view(Some(&imp.tabview));
            obj.new_file(true);
        }
    }
    impl WidgetImpl for EditorTabView {}
    impl BinImpl for EditorTabView {}
}

glib::wrapper! {
    pub struct EditorTabView(ObjectSubclass<inner::EditorTabView>)
    @extends adw::Bin, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl EditorTabView {
    pub fn new() -> Self {
        let obj = Object::new();
        // inner::EditorTabView::from_obj(&obj)
        //     .working_dir
        //     .replace(Some(dir));
        obj
    }
    fn set_up_editor_bindings(&self, page: &adw::TabPage, editor: &Editor) {
        editor
            .bind_property("filename", page, "title")
            .bidirectional()
            .sync_create()
            .build();
    }
    pub fn new_file(&self, focus: bool) {
        let tabview = &self.imp().tabview;
        let editor = Editor::new();

        let page = tabview.prepend(&editor);

        self.set_up_editor_bindings(&page, &editor);

        if focus {
            tabview.set_selected_page(&page);
        }
    }
    pub fn open_file(&self, path: &PathBuf, focus: bool) -> Result<(), io::Error> {
        let tabview = &self.imp().tabview;
        let editor = Editor::new();

        editor.open_file_path(path)?;

        let page = tabview.prepend(&editor);

        self.set_up_editor_bindings(&page, &editor);

        tabview.set_selected_page(&page);

        if focus {
            tabview.set_selected_page(&page);
        }
        Ok(())
    }
    pub fn get_active_editor(&self) -> Option<Editor> {
        self.imp()
            .tabview
            .selected_page()
            .map(|p| p.child())
            .and_downcast::<Editor>()
    }
    pub fn close_active(&self) {
        let tabview = &self.imp().tabview;

        if let Some(selected) = tabview.selected_page() {
            tabview.close_page(&selected);
        }
    }
}
