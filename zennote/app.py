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
    tab_bar: Adw.TabBar = Gtk.Template.Child("tab-bar")
    tab_view: Adw.TabView = Gtk.Template.Child("tab-view")

    opened_files: dict[pathlib.Path, tuple[Adw.TabPage, Editor]] = {}

    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        # Connect the tab view to the tab bar
        self.tab_bar.set_view(self.tab_view)
        self.open_file(path="/home/johannes/text.md")

    def open_file(self, path: str):

        editor = Editor()
        editor.open_file(path)
        page = self.tab_view.append(editor)

        name = pathlib.Path(path).name
        page.set_title(name)


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

    def write_to_file(self):
        if not self.path:
            return

        self.path.parent.mkdir(parents=True, exist_ok=True)
        self.path.touch(exist_ok=True)

        with self.path.open("w") as file:
            start, end = self.buffer.get_bounds()
            text = self.buffer.get_text(start, end, include_hidden_chars=True)
            file.write(text)


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

    def on_file_save(self, *args, **kwargs):
        page = self.tabview.tab_view.get_selected_page()

        if not page:
            return

        child: Editor = cast(Editor, page.get_child())

        child.write_to_file()


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
