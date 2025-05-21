from collections.abc import Generator
import pathlib
from gi.repository import Gtk, Pango
import json


def load_theme_from_file(
    path: pathlib.Path,
) -> Generator[tuple[str, Gtk.TextTag]] | None:

    if not path.is_file():
        return None

    content = None
    try:
        content = json.load(path.open("r"))

    except Exception as e:
        print(e)
        return None

    if not isinstance(content, dict):
        return None

    for k, v in content.items():
        if not isinstance(v, dict) or not isinstance(k, str):
            continue

        tag = Gtk.TextTag.new(k)

        color = v.get("color")
        font_style = v.get("font_style")
        font_weight = v.get("font_weight")

        if isinstance(color, str):
            tag.set_property("foreground", color)
        if isinstance(font_weight, int):
            tag.set_property("weight", font_weight)
        if isinstance(font_style, str):
            styles = {
                "italic": Pango.Style.ITALIC,
                "oblique": Pango.Style.OBLIQUE,
            }
            style = styles.get(font_style)
            tag.set_property("style", style)

        yield k, tag
