from gi.repository import (
    Adw,
    GObject,
    Gdk,
    Gio,
    Gtk,
)  # pyright: ignore[reportMissingModuleSource]
from zennote.dialogs import UnsavedResponse, request_open_file, unsaved_dialog
from zennote.editor import Editor

import pathlib
from typing import cast

from zennote.utils import Args, KwArgs


@Gtk.Template(resource_path="/ui/tabview.ui")
# kann nicht von Adw.ToolbarView verbt werden maaaan
class EditorTabView(Adw.Bin):
    __gtype_name__: str = "EditorTabView"
    bar: Adw.TabBar = cast(Adw.TabBar, Gtk.Template.Child("tab-bar"))
    view: Adw.TabView = cast(Adw.TabView, Gtk.Template.Child("tab-view"))

    opened_files: dict[pathlib.Path, tuple[Adw.TabPage, Editor]] = {}

    def __init__(self, **_kwargs: KwArgs):
        super().__init__()
        # Connect the tab view to the tab bar
        self.bar.set_view(self.view)
        self.new_file()

    def open_file(self, path: pathlib.Path):
        """
        opens a path to a file in the tabview.
        path may not be a valid file, in which case it will simply not open
        """

        editor = Editor()
        editor.open_file(path)
        page = self.view.append(editor)

        name = pathlib.Path(path).name
        page.set_title(name)
        self._setup_editor_bindings(page, editor)
        self.view.set_selected_page(page)

    def open_file_with_dialog(self):
        def on_open(f: Gio.File | None):
            if not f:
                return
            path = f.get_path()
            if path:
                self.open_file(pathlib.Path(path))

        request_open_file(on_open)

    def get_active_editor(self) -> Editor | None:
        page = self.view.get_selected_page()

        if not page:
            return None

        return cast(Editor, page.get_child())

    def _setup_editor_bindings(self, page: Adw.TabPage, editor: Editor):
        editor.bind_property(
            "filename",
            page,
            "title",
            GObject.BindingFlags.BIDIRECTIONAL | GObject.BindingFlags.SYNC_CREATE,
        )

    @Gtk.Template.Callback()
    def _on_close(
        self, view: Adw.TabView, page: Adw.TabPage, *_args: Args, **_kwargs: KwArgs
    ):
        editor: Editor = cast(Editor, page.get_child())

        if editor.is_saved():
            view.close_page_finish(page, True)
            return Gdk.EVENT_STOP

        def cb(res: UnsavedResponse):
            match res:
                case "save":

                    def cb(close: bool):
                        view.close_page_finish(page, close)

                    editor.write_to_file(cb)

                case "cancel":
                    view.close_page_finish(page, False)
                case "discard":
                    view.close_page_finish(page, True)

        print("file unsaved")
        unsaved_dialog(cb, editor.get_filename())

        return Gdk.EVENT_STOP

    # Für Kontextmenü - Todo
    @Gtk.Template.Callback()  # pyright: ignore[reportAny,]
    def _on_context(self, *_args: Args, **_kwargs: KwArgs):
        print(_args)
        print(_kwargs)

    def new_file(self) -> Editor:
        editor = Editor(saved=True)

        page = self.view.prepend(editor)

        self._setup_editor_bindings(page, editor)

        self.view.set_selected_page(page)

        return editor

    def close_active(self):
        page = self.view.get_selected_page()

        if page:
            self.view.close_page(page)
