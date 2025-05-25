from __future__ import annotations
from collections.abc import Sequence
from os import wait
import pathlib
from typing import override

import gi
from typing import Callable
from zennote.tabview import EditorTabView
from zennote.actions import load_accels_json, WindowActions
from zennote.utils import Args, KwArgs
from zennote.directory_browser import DirectoryBrowser


gi.require_version("Gtk", "4.0")
from typing import cast

gi.require_version("Adw", "1")
gi.require_version("Panel", "1")

# pyright: reportMissingModuleSource=false
from gi.repository import Gtk, GLib, Adw, Gio, Gdk


@Gtk.Template(resource_path="/ui/toolbar.ui")
class EditorToolBar(Gtk.PopoverMenuBar):
    __gtype_name__: str = "EditorToolBar"

    def __init__(self):
        super().__init__()


@Gtk.Template(resource_path="/ui/window.ui")
class Window(Adw.ApplicationWindow):
    __gtype_name__: str = "Window"
    tabview: EditorTabView = cast(EditorTabView, Gtk.Template.Child("editor-tabview"))
    toolbar: EditorToolBar = cast(EditorToolBar, Gtk.Template.Child("editor-toolbar"))
    directory_browser: DirectoryBrowser = cast(DirectoryBrowser, Gtk.Template.Child("dir-browser"))
    working_dir: pathlib.Path | None = pathlib.Path.cwd()

    def __init__(self, app: Adw.Application):
        super().__init__(application=app)

    def setup_actions(self):
        actions: dict[WindowActions, tuple[Callable[[], None]]] = {
            "save-file": (self._on_file_save,),
            "new-file": (self._on_file_new,),
            "save-file-as": (self._on_file_save_as,),
            "open-file": (self._on_file_open,),
            "close-file": (self._on_file_close,),
        }

        for (
            name,
            (callback,),
        ) in actions.items():
            action = Gio.SimpleAction(name=name)
            action.connect("activate", callback)
            self.add_action(action)
        
        file_path_open = Gio.SimpleAction(name="file-path-open", parameter_type=GLib.VariantType.new("s"))
        file_path_open.connect("activate", self._on_file_path_open)
        self.add_action(file_path_open)
        

    def _on_file_close(self, *_args: Args, **_kwargs: KwArgs) -> None:
        self.tabview.close_active()

    def _on_file_save(self, *_args: Args, **_kwargs: KwArgs):
        editor = self.tabview.get_active_editor()

        if not editor:
            return

        editor.write_to_file()
    
    def _on_file_path_open(self, _, path: GLib.VariantType, *_args: Args, **_kwargs: KwArgs):
        str_path, _ = path.dup_string()
        
        print(str_path)
        
        fpath = pathlib.Path(str_path)
        
        self.tabview.open_file(fpath)
        
        
        
    def _on_file_save_as(self, *_args: Args, **_kwargs: KwArgs):
        editor = self.tabview.get_active_editor()

        if not editor:
            return

        editor.request_new_file_path(lambda x: None)

        editor.write_to_file()

    def set_working_dir(self, path: pathlib.Path):
        self.working_dir = path
        self.directory_browser.set_path(path)

    def _on_file_new(self, *_args: Args, **_kwargs: KwArgs):
        self.tabview.new_file()

    def _on_file_open(self, *_args: Args, **_kwargs: KwArgs):
        self.tabview.open_file_with_dialog()


class App(Adw.Application):

    directories: list[pathlib.Path] = []
    files: list[pathlib.Path] = []
    active_window: Gtk.Window | None = None

    def __init__(self):
        super().__init__(
            application_id="io.github.johannes.zennote",
            flags=Gio.ApplicationFlags.HANDLES_OPEN,
        )
        self.set_resource_base_path("/")
        self._load_css()

    def _load_css(self, *_args: Args, **_kwargs: KwArgs) -> None:
        print(_args, _kwargs)
        print("Loading CSS")
        css_provider = Gtk.CssProvider()
        css_provider.load_from_resource("/styles/style.css")
        
        display = Gdk.Display.get_default()
        if not display:
            raise RuntimeError("No default display found")
        
        Gtk.StyleContext.add_provider_for_display(
            display,
            css_provider,
            Gtk.STYLE_PROVIDER_PRIORITY_USER,
        )


    @override
    def do_activate(self):
        
        self.load_accels()
        GLib.set_application_name("zennote")

        if not self.directories:
            self.directories.append(pathlib.Path.cwd())
        
        for dir in self.directories:
            window = Window(self)
            window.setup_actions()

            for file in self.files:

                window.tabview.open_file(file)
            
            window.set_working_dir(dir)
            window.present()

        # self.active_window = window

    def load_accels(self):
        for action, accel in load_accels_json().items():
            print(action, accel)
            self.set_accels_for_action(action, (accel,))

    @override
    def do_open(self, files: Sequence[Gio.File], hint: str, *args, **kwargs) -> None:
        for file in files:
            path = file.get_path()

            if not path:
                continue

            path = pathlib.Path(path)

            if path.is_file():
                self.files.append(path)

            elif path.is_dir():
                self.directories.append(path)

        self.activate()
