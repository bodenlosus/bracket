from collections.abc import Generator
import pathlib
from gi.repository import Gtk, Pango
import json


def load_theme_from_file(
    path: pathlib.Path,
) -> Generator[tuple[str, Gtk.TextTag]] | None:
    """
    function to load a theme from a JSON file and returns the Gtk.TextTags used for styling in a generators
    if parsing fails it returns `None`
    """
    
    # if the file is not existent it returns
    if not path.is_file():
        return None

    # load the content - may fail so we do handle corresponding errors and  return
    content = None
    try:
        content = json.load(path.open("r"))

    except Exception as e:
        print(e)
        return None

    if not isinstance(content, dict):
        return None

    # loop over the json list where k is the tag name and v is a dict containing style info
    for k, v in content.items():
        # if the item is npt properly parsed skip it
        if not isinstance(v, dict) or not isinstance(k, str):
            continue

        # make a new tag
        tag = Gtk.TextTag.new(k)

        # retrieve style information
        color = v.get("color")
        font_style = v.get("font_style")
        font_weight = v.get("font_weight")

        # if the inforamtion is properly parsed apply it to the tag
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

        # yield the tag and the tag name
        yield k, tag
