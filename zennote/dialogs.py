import gi

from zennote.utils import Args, KwArgs
gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")

from typing import Callable, Literal, cast

from gi.repository import Gtk, Adw, Gio # pyright: ignore[reportMissingModuleSource]

type UnsavedResponse = Literal["discard", "save", "cancel"]
def unsaved_dialog(callback: Callable[[UnsavedResponse,], None], filename: str | None = None):
    """
    Opens a new dialog asking the user if they want to save changes to a file.
    the callback will be called with the response from the dialog since dialogs are async in GTK.
    """
    # body with the message to display
    body = f"File <b>{filename}</b> contains unsaved changes. Do you wish to save them?"

    # creates the dialog
    dialog = Adw.AlertDialog(heading="Save Changes?", body=body, body_use_markup=True, title="Zen Note")

    # makes it wide, so it looks better
    dialog.set_property("prefer-wide-layout", True)

    # defines the response type for the actions
    resp: dict[UnsavedResponse, tuple[str, Adw.ResponseAppearance, ]]  = {
        "discard": ("Discard", Adw.ResponseAppearance.DESTRUCTIVE),
        "cancel": ("Cancel", Adw.ResponseAppearance.DEFAULT),
        "save": ("Save File", Adw.ResponseAppearance.SUGGESTED),
    }

    # adds the responses to the dialog
    for id, (label, appearance) in resp.items():
        dialog.add_response(id, label)
        dialog.set_response_appearance(id, appearance)

    # sets the default response to "save" so that pressing enter will save the file
    # and the close response to "cancel" so that pressing escape will cancel the dialog
    dialog.set_default_response("save")
    dialog.set_close_response("cancel")

    # connects the callback to the dialog, so that it will be called when the user chooses an option
    def on_chosen(dialog: Adw.AlertDialog, res: Gio.AsyncResult, *_args: Args, **_kwargs: KwArgs):
        choice = dialog.choose_finish(res)
        callback(cast(UnsavedResponse, choice))

    # shows the dialog and connects the callback to it
    dialog.choose(callback=on_chosen)


def request_save_file(callback: Callable[[None | Gio.File], None]) -> None:
    """
    Requests a new file path from the user and calls the callback with the file path.
    The callback will be called with None if the user cancels the dialog.
    """
    # callback is called with the file path or None if the user cancels the dialog
    def on_save(dialog: Gtk.FileDialog, result: Gio.AsyncResult, *args, **kwargs):
        # try-except block to handle the case where the user cancels the dialog
        try:
            # dialog.save_finish(result) will return the file path if the user chooses a file
            file = dialog.save_finish(result)
            callback(file)
        except:
            # if the user cancels the dialog, the save_finish will print to the console
            # TODO: proper error handling: maybe some sort of notification in red...
            print("Couldnt save file, maybe it hasnt been provided???")
            callback(None)

    # creates the file dialog and adds the callback to it
    d = Gtk.FileDialog()
    d.save(
        None,
        None,
        on_save,
    )

# You could generalize this function to request any file operation, but since its a very simple function it adds unnecessary complexity.
def request_open_file(callback: Callable[[None | Gio.File], None]) -> None:
    """
    Requests a file path to open from the user and calls the callback with the file path.
    The callback will be called with None if the user cancels the dialog.
    """
    # callback is called with the file path or None if the user cancels the dialog
    def on_open(dialog: Gtk.FileDialog, result: Gio.AsyncResult, *args, **kwargs):
        # try-except block to handle the case where the user cancels the dialog
        try:
            # dialog.open_finish(result) will return the file path if the user chooses a file
            file = dialog.open_finish(result)
            callback(file)
        except:
            # if the user cancels the dialog, the open_finish will print to the console
            # TODO: again: proper error handling
            print("Couldnt open file, maybe it hasnt been provided???")
            callback(None)

    # open the dialog
    d = Gtk.FileDialog()
    d.open(
        None,
        None,
        on_open,
    )
