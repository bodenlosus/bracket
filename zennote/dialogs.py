import gi

from zennote.utils import Args, KwArgs
gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
gi.require_version("Panel", "1")

from typing import Callable, Literal, cast

from gi.repository import Gtk, Adw, Gio # pyright: ignore[reportMissingModuleSource]

type UnsavedResponse = Literal["discard", "save", "cancel"]
def unsaved_dialog(callback: Callable[[UnsavedResponse,], None], filename: str | None = None):

    body = f"File <b>{filename}</b> contains unsaved changes. Do you wish to save them?"

    d = Adw.AlertDialog(heading="Save Changes?", body=body, body_use_markup=True, title="Zen Note")

    d.set_property("prefer-wide-layout", True)

    resp: dict[UnsavedResponse, tuple[str, Adw.ResponseAppearance, ]]  = {
        "discard": ("Discard", Adw.ResponseAppearance.DESTRUCTIVE),
        "cancel": ("Cancel", Adw.ResponseAppearance.DEFAULT),
        "save": ("Save File", Adw.ResponseAppearance.SUGGESTED),
    }

    for id, (label, appearance) in resp.items():
        d.add_response(id, label)
        d.set_response_appearance(id, appearance)

    d.set_default_response("save")
    d.set_close_response("cancel")

    def on_chosen(dialog: Adw.AlertDialog, res: Gio.AsyncResult, *_args: Args, **_kwargs: KwArgs):
        choice = dialog.choose_finish(res)
        callback(cast(UnsavedResponse, choice))

    d.choose(callback=on_chosen)


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
