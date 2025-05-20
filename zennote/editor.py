from gi.repository import (
    GObject,
    Gio,
    Gtk,
    Pango,
)  # pyright: ignore[reportMissingModuleSource]

from zennote.dialogs import request_save_file
import pathlib
from typing import Any, Callable, cast

from zennote.utils import Args, KwArgs
from highlighter import

recognized_names = [
    "attribute",
    "comment",
    "constant",
    "constant.builtin",
    "constructor",
    "embedded",
    "function",
    "function.builtin",
    "keyword",
    "module",
    "number",
    "operator",
    "property",
    "property.builtin",
    "punctuation",
    "punctuation.bracket",
    "punctuation.delimiter",
    "punctuation.special",
    "string",
    "string.special",
    "tag",
    "type",
    "type.builtin",
    "variable",
    "variable.builtin",
    "variable.parameter",
];

class TagFormat:
    def __init__(
        self,
        foreground: str | None = None,
        background: str | None = None,
        underline: Pango.Underline | None = None,
        weight: Pango.Weight | None = None,
        style: Pango.Style | None = None,
    ):
        self.__inner = {
            "weight": weight,
            "foreground": foreground,
            "background": background,
            "underline": underline,
            "style": style,
        }

    def to_dict(self):
        res: dict[str, Pango.Weight | str | Pango.Underline | Pango.Style | None] = (
            dict()
        )

        for k, v in self.__inner.items():
            if not v:
                continue
            res[k] = v

        return res


c = {
    "00": "#1C1E26",
    "01": "#232530",
    "02": "#2E303E",
    "03": "#6F6F70",
    "04": "#9DA0A2",
    "05": "#CBCED0",
    "06": "#DCDFE4",
    "07": "#E3E6EE",
    "08": "#E93C58",
    "09": "#E58D7D",
    "0A": "#EFB993",
    "0B": "#EFAF8E",
    "0C": "#24A8B4",
    "0D": "#DF5273",
    "0E": "#B072D1",
    "0F": "#E4A382",
}



@Gtk.Template(resource_path="/ui/editor.ui")
class Editor(Gtk.TextView):
    __gtype_name__ = "Editor"
    path: pathlib.Path | None = None
    buffer: Gtk.TextBuffer = Gtk.Template.Child("editor-text-buffer")
    filename: GObject.Property = GObject.Property(type=str, default="Untitled")
    saved: GObject.Property = GObject.Property(type=bool, default=False)
    highlighter = HL(recognized_names)

    def __init__(self, saved: bool = False, *_args: Any, **_kwargs: Any):
        super().__init__(*_args, **_kwargs)
        self.set_saved(saved)
        self.highlighter.set_language()

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
                if cb: cb(False)
                return

            path = f.get_path()

            if not path:
                if cb: cb(False)
                return

            self.set_file(path)

            if cb: cb(True)

        request_save_file(on_save)

    def highlight(self):
        text = self.get_text()

        events = self.highlighter.highlight(text)

        for i in events:
            print(type(i))
