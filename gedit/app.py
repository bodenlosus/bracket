from json.decoder import NaN

import sys
import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
gi.require_version("Panel", "1")

from gi.repository import Gtk, GLib, Adw, Panel, Gio

class GEdit(Adw.Application):
    def __init__(self):
        super().__init__(application_id="io.github.johannes.gedit")

    def do_activate(self):

        GLib.set_application_name("GEdit")
        resource = Gio.resource_load("resources")
        Gio.resources_register(resource)
        window = Window(self)
        window.present()

def app():
    gedit = GEdit()
    exit_status = gedit.run(sys.argv)
    sys.exit(exit_status)

if __name__ == "__main__":
    app()

class Window(Adw.ApplicationWindow):
    def __init__(self, application: Adw.Application):
        
        builder = Gtk.Builder.new_from_resource("/io/github/johannes/gedit/ui/window.ui")

class Editor(Gtk.TextView):
    def __init__(self) -> None:
        super().__init__()
