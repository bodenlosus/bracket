from __future__ import annotations

import sys
import gi
import pathlib
import asyncio
from enum import Enum
from typing import Callable, Union, cast
from zennote.actions import load_accels_json, WindowActions
from zennote.dialogs import request_open_file, request_save_file

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
gi.require_version("Panel", "1")
import gi.events
from gi.repository import Gtk, GLib, Adw, Panel, Gio, GObject, GdkPixbuf


@Gtk.Template(resource_path="/ui/tabview.ui")
# kann nicht von Adw.ToolbarView verbt werden maaaan
class EditorTabView(Adw.Bin):
    __gtype_name__ = "EditorTabView"
    bar: Adw.TabBar = Gtk.Template.Child("tab-bar")
    view: Adw.TabView = Gtk.Template.Child("tab-view")

    opened_files: dict[pathlib.Path, tuple[Adw.TabPage, Editor]] = {}

    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        # Connect the tab view to the tab bar
        self.bar.set_view(self.view)
        self.new_file()

    def open_file(self, path: str):

        editor = Editor()
        editor.open_file(path)
        page = self.view.append(editor)

        name = pathlib.Path(path).name
        page.set_title(name)
        self._setup_editor_bindings(page, editor) 
        self.view.set_selected_page(page)
    
    def open_file_with_dialog(self):
        def on_open(f: Gio.File | None):
            if not f: return
            path = f.get_path()
            if path: 
                self.open_file(path)
        
        request_open_file(on_open)

    def get_active_editor(self) -> Union[Editor, None]:
        page = self.view.get_selected_page()

        if not page:
            return None

        return cast(Editor, page.get_child())
    
    def _setup_editor_bindings(self, page: Adw.TabPage, editor: Editor):
        editor.bind_property(
            "filename", page, "title", GObject.BindingFlags.BIDIRECTIONAL | GObject.BindingFlags.SYNC_CREATE
        )
        
        
    
    def new_file(self) -> Editor:
        editor = Editor()

        page = self.view.prepend(editor)

        self._setup_editor_bindings(page, editor)

        self.view.set_selected_page(page)
        
        r = Gio.File.new_for_path("/home/johannes/ZenNote/zennote/resources/icons/circle.svg")
        f = Gtk.Image.new_from_icon_name("edit-clear")
        print(f.get_gicon())        # print("loading:", page.get_loading()) 
        # page.set_icon()
        
        # print(f.get_gicon())
        page.set_icon(f.get_gicon())
        
        return editor


@Gtk.Template(resource_path="/ui/editor.ui")
class Editor(Gtk.TextView):
    __gtype_name__ = "Editor"
    path: Union[pathlib.Path, None] = None
    buffer: Gtk.TextBuffer = Gtk.Template.Child("editor-text-buffer")
    filename: GObject.Property = GObject.Property(type=str, default="Untitled")
    saved: GObject.Property = GObject.Property(type=bool, default=False)
    
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.buffer.connect("changed", self._on_changed)
    
    def _on_changed(self, *args, **kwargs):
        self.set_saved(False)        

    def open_file(self, path: str):
        self.set_file(path)

        if not self.path:
            return
        if not self.path.exists(follow_symlinks=True):
            return
        if not self.path.is_file():
            return

        with self.path.open("r") as file:
            self.buffer.set_text(file.read())
            self.set_saved(True)
    
    def set_saved(self, v: bool):
            self.set_property("saved", v)

    def set_file(self, path: str):
        """sets the internal path and creates it and its parents, sets the title to the files name"""

        path = pathlib.Path(path).resolve()
        path.parent.mkdir(parents=True, exist_ok=True)
        path.touch(exist_ok=True)

        self.path = path
        
        self.set_property("filename", path.name)

    def write_to_file(self):
        if not self.path:
            self.request_new_file_path()
            return

        self.path.parent.mkdir(parents=True, exist_ok=True)
        self.path.touch(exist_ok=True)

        with self.path.open("w") as file:
            start, end = self.buffer.get_bounds()
            text = self.buffer.get_text(start, end, include_hidden_chars=True)
            file.write(text)
            self.set_saved(True)

    def request_new_file_path(self):
        def on_save(f: Gio.File | None):
            if not f: return
            path = f.get_path()
            if path: self.set_file(path)
        
        request_save_file(on_save)
        
@Gtk.Template(resource_path="/ui/toolbar.ui")
class EditorToolBar(Gtk.PopoverMenuBar):
    __gtype_name__ = "EditorToolBar"

    def __init__(self):
        super().__init__()


@Gtk.Template(resource_path="/ui/window.ui")
class Window(Adw.ApplicationWindow):
    __gtype_name__ = "Window"
    tabview: EditorTabView = Gtk.Template.Child("editor-tabview")
    toolbar = Gtk.Template.Child("editor-toolbar")

    def __init__(self, app):
        super().__init__(application=app)

    def setup_actions(self):
        actions: dict[WindowActions, tuple[Callable]] = {
            "save-file": (self._on_file_save,),
            "new-file": (self._on_file_new,),
            "save-file-as": (self._on_file_save_as,),
            "open-file": (self._on_file_open,),
        }
        
        for name, (callback,), in actions.items():
            action = Gio.SimpleAction(name=name)
            action.connect("activate", callback)
            self.add_action(action)
            

    def _on_file_save(self, *args, **kwargs):
        editor = self.tabview.get_active_editor()

        if not editor:
            return

        editor.write_to_file()
    

    def _on_file_save_as(self, *args, **kwargs):
        editor = self.tabview.get_active_editor()

        if not editor:
            return
        
        editor.request_new_file_path()

        editor.write_to_file()

    def _on_file_new(self, *args, **kwargs):
        self.tabview.new_file()
    
    def _on_file_open(self, *args, **kwargs):
        self.tabview.open_file_with_dialog()
        
class ZenNote(Adw.Application):

    def __init__(self):
        super().__init__(application_id="io.github.johannes.zennote")
        self.set_resource_base_path("/")
        asyncio.set_event_loop_policy(gi.events.GLibEventLoopPolicy())
        
    def do_activate(self):
        GLib.set_application_name("zennote")
        window = Window(self)
        window.setup_actions()
        self.load_accels()
        window.present()
    
    def load_accels(self):
        for action, accel in load_accels_json().items():
            print(action, accel)
            self.set_accels_for_action(action, (accel,))