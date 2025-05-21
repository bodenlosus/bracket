from gi.repository import (
    GObject,
    Gio,
    Gtk,
    Pango,
)  # pyright: ignore[reportMissingModuleSource]

from zennote.dialogs import request_save_file
import pathlib
from typing import Any, Callable, cast

from zennote.themes import load_theme_from_file
from zennote.utils import Args, KwArgs
from highlighter import HLEvent, Highlighter


@Gtk.Template(resource_path="/ui/editor.ui")
class Editor(Gtk.TextView):
    __gtype_name__ = "Editor"
    path: pathlib.Path | None = None
    buffer: Gtk.TextBuffer = Gtk.Template.Child("editor-text-buffer")
    filename: GObject.Property = GObject.Property(type=str, default="Untitled")
    saved: GObject.Property = GObject.Property(type=bool, default=False)
    recognized_names: list[str] = []

    def __init__(self, saved: bool = False, *_args: Any, **_kwargs: Any):
        super().__init__(*_args, **_kwargs)
        self.set_saved(saved)
        self._load_tags()
        self.highlighter = Highlighter(self.recognized_names)
        self.highlighter.set_language()

    def _load_tags(self):
        tagtable = self.buffer.get_tag_table()

        path = pathlib.Path(__file__).parent.resolve() / "tags.json"
        theme = load_theme_from_file(path)
        if not theme:
            return
        for name, tag in theme:
            tagtable.add(tag)
            self.recognized_names.append(name)

    @Gtk.Template.Callback()
    def _on_changed(self, *_args: Args, **_kwargs: KwArgs):
        self.set_saved(False)
        self.highlight()

    def get_filename(self) -> str:
        return cast(str, self.get_property("filename"))

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

    def is_saved(self) -> bool:
        return cast(bool, self.get_property("saved"))

    def set_saved(self, v: bool):
        match v:
            case True:
                print("saved")
            case False:
                print("unsaved")
        self.set_property("saved", v)

    def set_file(self, path: str):
        """sets the internal path and creates it and its parents, sets the title to the files name"""

        file_path = pathlib.Path(path).resolve()
        file_path.parent.mkdir(parents=True, exist_ok=True)
        file_path.touch(exist_ok=True)

        self.path = file_path

        self.set_property("filename", file_path.name)

    def write_to_file(self, cb: Callable[[bool], None] | None = None):
        if not self.path:
            self.request_new_file_path(cb)
            return

        try:
            self.path.parent.mkdir(parents=True, exist_ok=True)
            self.path.touch(exist_ok=True)

            with self.path.open("w") as file:
                start, end = self.buffer.get_bounds()
                text = self.buffer.get_text(start, end, include_hidden_chars=True)
                file.write(text)

            self.set_saved(True)

            if cb:
                cb(True)

        except Exception as e:
            print(e)
            if cb:
                cb(False)

    def get_text(self) -> str:
        start, end = self.buffer.get_bounds()
        return self.buffer.get_text(start, end, include_hidden_chars=True)

    def request_new_file_path(self, cb: Callable[[bool], None] | None = None):
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
        text = self.get_text()

        events = self.highlighter.highlight(text)

        tag: str | None = None
        bounds = self.buffer.get_bounds()
        self.buffer.remove_all_tags(*bounds)

        for event in events:

            match event:
                case HLEvent.Start():
                    (tag,) = event
                case HLEvent.Source():
                    (start, end) = event
                    start_iter = self.buffer.get_iter_at_offset(start)
                    end_iter = self.buffer.get_iter_at_offset(end)

                    if tag:
                        self.buffer.apply_tag_by_name(tag, start_iter, end_iter)

                case HLEvent.End():
                    tag = None
