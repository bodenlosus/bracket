import sys
import gi
import pathlib

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
gi.require_version("Panel", "1")
from gi.repository import Gtk, GLib, Adw, Panel, Gio

@Gtk.Template(resource_path="/ui/editor.ui")
class Editor(Gtk.TextView):
    __gtype_name__ = "Editor"
    def __init__(self):
        super().__init__()

@Gtk.Template(resource_path="/ui/toolbar.ui")
class EditorToolBar(Gtk.PopoverMenuBar):
    __gtype_name__ = "EditorToolBar"

    def __init__(self):
        super().__init__()


@Gtk.Template(resource_path="/ui/window.ui")
class Window(Adw.ApplicationWindow):
    __gtype_name__ = "Window"

    def __init__(self, app):
        super().__init__(application=app)
        self.get_id


class ZenNote(Adw.Application):
    def __init__(self):
        super().__init__(application_id="io.github.johannes.zennote")
        self.set_resource_base_path("/")

    def do_activate(self):
        GLib.set_application_name("zennote")

        window = Window(self)
        window.present()
