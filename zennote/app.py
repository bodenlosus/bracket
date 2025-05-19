from __future__ import annotations

import sys
import gi
import pathlib
import asyncio
from enum import Enum
from typing import Union, cast

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
gi.require_version("Panel", "1")
import gi.events
from gi.repository import Gtk, GLib, Adw, Panel, Gio, GObject


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
        self.open_file(path="/home/johannes/text.md")

    def open_file(self, path: str):

        editor = Editor()
        editor.open_file(path)
        page = self.view.append(editor)

        name = pathlib.Path(path).name
        page.set_title(name)

    def get_active_editor(self) -> Union[Editor, None]:
        page = self.view.get_selected_page()

        if not page:
            return None

        return cast(Editor, page.get_child())

    def new_file(self):
        editor = Editor()

        page = self.view.prepend(editor)
        page.set_title("Untitled")

        self.view.set_selected_page(page)


@Gtk.Template(resource_path="/ui/editor.ui")
class Editor(Gtk.TextView):
    __gtype_name__ = "Editor"
    path: Union[pathlib.Path, None] = None
    buffer: Gtk.TextBuffer = Gtk.Template.Child("editor-text-buffer")

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)

    def open_file(self, path: str):
        self.path = pathlib.Path(path).resolve()

        if not self.path.exists(follow_symlinks=True):
            return
        if not self.path.is_file():
            return

        with self.path.open("r") as file:
            self.buffer.set_text(file.read())

    def write_to_file(self, window: Gtk.Window):
        if not self.path:
            self.open_save_dialogue(window)
            return

        self.path.parent.mkdir(parents=True, exist_ok=True)
        self.path.touch(exist_ok=True)

        with self.path.open("w") as file:
            start, end = self.buffer.get_bounds()
            text = self.buffer.get_text(start, end, include_hidden_chars=True)
            file.write(text)

    def open_save_dialogue(self, window: Gtk.Window):

        d = Gtk.FileDialog()
        d.save()


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
        save = Gio.SimpleAction(name="save-current")
        save.connect("activate", self.on_file_save)

        self.add_action(save)

        new_file = Gio.SimpleAction(name="new-file")
        new_file.connect("activate", self.on_file_new)

        self.add_action(new_file)

    def on_file_save(self, *args, **kwargs):
        editor = self.tabview.get_active_editor()

        if not editor:
            return

        editor.write_to_file(self)

    def on_file_new(self, *args, **kwargs):
        self.tabview.new_file()


class ZenNote(Adw.Application):

    def __init__(self):
        super().__init__(application_id="io.github.johannes.zennote")
        self.set_resource_base_path("/")
        asyncio.set_event_loop_policy(gi.events.GLibEventLoopPolicy())

    def do_activate(self):
        GLib.set_application_name("zennote")

        window = Window(self)
        window.setup_actions()
        window.present()
