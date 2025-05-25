from gi.repository import (
    GObject,
    Gio,
    Gtk,
    Pango,
)  # pyright: ignore[reportMissingModuleSource]

from zennote.dialogs import request_save_file
import pathlib
from typing import Any, Callable, cast, final

from zennote.themes import load_theme_from_file
from zennote.utils import Args, KwArgs
from highlighter import HLEvent, Highlighter


@Gtk.Template(resource_path="/ui/editor.ui")
class Editor(Gtk.TextView):
    """
    Represents a text editor widget with syntax highlighting capabilities.
    Inherits from Gtk.TextView and provides methods to open files, save files,
    and highlight text based on recognized names from a theme.
    """
    __gtype_name__ = "Editor"
    path: pathlib.Path | None = None
    _buffer: Gtk.TextBuffer = cast(
        Gtk.TextBuffer, Gtk.Template.Child("editor-text-buffer")
    )
    filename: GObject.Property = GObject.Property(type=str, default="Untitled")
    saved: GObject.Property = GObject.Property(type=bool, default=False)
    _recognized_names: list[str] = []

    def __init__(self, saved: bool = False, *_args: Any, **_kwargs: Any):
        super().__init__(*_args, **_kwargs)
        self.set_saved(saved)
        self._load_tags()
        self.highlighter = Highlighter(self._recognized_names)
        self.highlighter.set_language()

    def _load_tags(self):
        """
        loads a theme from a JSON file and adds the tags to the text buffer's tag table.
        The tags provide names to hightlight, as well as color and style.
        The tags are used by the highlighter to apply styles to the text in the editor.
        if loading the theme fails, it will simply return without doing anything.
        The tags are then added to the text buffer's tag table.
        """
        tagtable = self._buffer.get_tag_table()

        path = pathlib.Path(__file__).parent.resolve() / "theme.json"
        theme = load_theme_from_file(path)
        if not theme:
            return
        for name, tag in theme:
            tagtable.add(tag)
            self._recognized_names.append(name)

    @Gtk.Template.Callback()
    def _on_changed(self, *_args: Args, **_kwargs: KwArgs):
        """
        Internal callback for when the text in the editor changes.
        It sets the editor as unsaved and highlights the text.
        """
        self.set_saved(False)
        self.highlight()

    def get_filename(self) -> str:
        return cast(str, self.get_property("filename"))

    def open_file(self, path: pathlib.Path):
        """
        sets `self.path` to the given path, reads the file content and sets it to the editor's buffer.
        If the file does not exist or isn't a file, it will return without doing anything.
        """
        self.set_file(path)

        if not self.path:
            return
        if not self.path.exists(follow_symlinks=True):
            return
        if not self.path.is_file():
            return

        with self.path.open("r") as file:
            self._buffer.set_text(file.read())
            self.set_saved(True)

    def is_saved(self) -> bool:
        return cast(bool, self.get_property("saved"))

    def set_saved(self, v: bool):
        self.set_property("saved", v)

    def set_file(self, path: pathlib.Path | str):
        """sets the internal path and creates it and its parents, sets the title to the files name"""

        file_path = pathlib.Path(path) if isinstance(path, str) else path

        file_path.parent.mkdir(parents=True, exist_ok=True)
        file_path.touch(exist_ok=True)

        self.path = file_path

        self.set_property("filename", file_path.name)

    def write_to_file(self, cb: Callable[[bool], None] | None = None):
        """
        writes the current text in the editors buffer to the file specified by `self.path`.
        If `self.path` is not set, it will request a new file path using `request_new_file_path`.
        """
        if not self.path:
            self.request_new_file_path(cb)
            return

        try:
            self.path.parent.mkdir(parents=True, exist_ok=True)
            self.path.touch(exist_ok=True)

            with self.path.open("w") as file:
                start, end = self._buffer.get_bounds()
                text = self._buffer.get_text(start, end, include_hidden_chars=True)
                file.write(text)

            self.set_saved(True)

            if cb:
                cb(True)

        except Exception as e:
            print(e)
            if cb:
                cb(False)

    def get_text(self) -> str:
        """
        retrieves and returns the text from the editor's buffer
        """
        start, end = self._buffer.get_bounds()
        return self._buffer.get_text(start, end, include_hidden_chars=True)

    def request_new_file_path(self, cb: Callable[[bool], None] | None = None):
        """
        Opens a file chooser dialog to request a new file path. 
        If a file is selected, it sets the file path and writes the current text to that file.
        Callback `cb` is called with `True` if the file was saved successfully, or `False` if no file was selected or an error occurred.
        """
        def on_save(f: Gio.File | None):
            if not f:
                if cb:
                    cb(False)
                return

            path = f.get_path()

            if not path:
                if cb:
                    cb(False)
                return

            self.set_file(path)
            self.write_to_file()

            if cb:
                cb(True)

        request_save_file(on_save)

    def highlight(self):
        """
        Highlights the text in the editor using the highlighter.
        """
        text = self.get_text()

        events = self.highlighter.highlight(text)

        tag: str | None = None
        bounds = self._buffer.get_bounds()
        self._buffer.remove_all_tags(*bounds)

        for event in events:

            match event:
                case HLEvent.Start():
                    (tag,) = event
                case HLEvent.Source():
                    (start, end) = event
                    start_iter = self._buffer.get_iter_at_offset(start)
                    end_iter = self._buffer.get_iter_at_offset(end)

                    if tag:
                        self._buffer.apply_tag_by_name(tag, start_iter, end_iter)
                case HLEvent.End():
                    tag = None
                case _:
                    pass
