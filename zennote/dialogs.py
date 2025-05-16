import gi
gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
gi.require_version("Panel", "1")

from typing import Any, Callable

from gi.repository import Gtk, GLib, Adw, Panel, Gio, GObject


def request_save_file(callback: Callable[[None | Gio.File], None]) -> None:
    def on_save(dialog: Gtk.FileDialog, result: Gio.AsyncResult, *args, **kwargs):
        try:
            file = dialog.save_finish(result)
            callback(file)
        except:
            print("Couldnt save file, maybe it hasnt been provided???")
            callback(None)

    d = Gtk.FileDialog()
    d.save(
        None,
        None,
        on_save,
    )

def request_open_file(callback: Callable[[None | Gio.File], None]) -> None:
    def on_open(dialog: Gtk.FileDialog, result: Gio.AsyncResult, *args, **kwargs):
        try:
            file = dialog.open_finish(result)
            callback(file)
        except:
            print("Couldnt open file, maybe it hasnt been provided???")
            callback(None)

    d = Gtk.FileDialog()
    d.open(
        None,
        None,
        on_open,
    )