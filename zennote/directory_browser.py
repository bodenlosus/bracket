import pathlib
from typing import cast
import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")

from gi.repository import Gtk, GLib, Adw, Gio, GObject

from pathlib import Path

class DirectoryItem(GObject.Object):
    """Data model for directory/file items"""

    def __init__(self, path):
        super().__init__()
        self.path = Path(path)
        self.name = self.path.name
        self._is_dir: bool = self.path.is_dir()

    @property
    def display_name(self):
        return self.name

    @property
    def is_dir(self) -> bool:
        return self.path.is_dir()


@Gtk.Template(resource_path="/ui/directory_browser.ui")
class DirectoryBrowser(Gtk.ListView):
    """
    This class represents the file tree on the side of the main window
    """
    __gtype_name__: str = "DirectoryBrowser"

    # Gtk Selection Model for the ListView, indicates that one item can be selected at a time
    # The concept of ListViews in GTK takes time to wrap your head around
    selection_model = cast(Gtk.SingleSelection, Gtk.Template.Child("selection-model"))
    # store, basically a list of items to display in the ListView
    store = Gio.ListStore.new(DirectoryItem)
    # The TreeListModel is a model that allows for hierarchical data (like directories and files)
    tree_model: Gtk.TreeListModel | None = None
    # The root path for the directory browser, defaults to the current working directory
    root_path: Path | None = pathlib.Path.cwd()  # Default root path
    def __init__(self):
        super().__init__()
        self._setup_tree_model()

    def set_path(self, path: Path):
        """
        Set the root path for the directory browser
        This will reset the model and load the new directory contents
        """
        self.root_path = path
        self._setup_tree_model()

    def _setup_tree_model(self):
        """Setup the TreeListModel with root directory"""

        # create a new list store for the root item
        root_store = Gio.ListStore.new(DirectoryItem)
        # Create the root item for the model, representing the root directory
        root_item = DirectoryItem(self.root_path)
        # Append the root item to the store
        root_store.append(root_item)

        # Create new TreeListModel
        # will go through all files recursively and create child widgets for each directory and file
        self.tree_model = Gtk.TreeListModel.new(
            root_store,           # root model
            False,               # passthrough - don't pass model items directly
            False,                # autoexpand - expand items automatically
            self.create_child_model  # function to create child models
        )

        # retrieve root row from the model
        root_row = self.tree_model.get_child_row(0)

        # Expand root row (just for visual clarity)
        if root_row:
            root_row.set_expanded(True)

        # Set the TreeListModel as the model for the ListView
        self.selection_model.set_model(self.tree_model)

    def create_child_model(self, item: DirectoryItem):
        """
        Create child model for TreeListModel
        This function is called for each item to get its children
        """
        directory_item = item  # This is a DirectoryItem

        # Only directories can have children
        if not directory_item.is_dir:
            return None

        # Create a ListStore for this directory's children
        child_store = Gio.ListStore.new(DirectoryItem)

        try:
            # Get directory contents
            children = list(directory_item.path.iterdir())

            # Sort: directories first, then files, alphabetically
            children.sort(key=lambda p: (not p.is_dir(), p.name.lower()))

            # Add children to store
            for child_path in children:
                # Skip hidden files/directories
                if child_path.name.startswith('.'):
                    continue

                child_item = DirectoryItem(child_path)
                child_store.append(child_item)

        except (PermissionError, OSError) as e:
            # Create an error item for inaccessible directories
            # TODO: handle this properly, maybe with a notification
            error_item = DirectoryItem(directory_item.path / f"[Access Denied: {e}]")
            child_store.append(error_item)

        # Return None if no children, otherwise return the store
        return child_store if child_store.get_n_items() > 0 else None

    # Factorys are the constructors for the widgets in the ListView, in setup we define the base widget structure
    @Gtk.Template.Callback()
    def _on_factory_setup(self, factory, list_item):
        """Setup callback - create the widget structure"""

        # Create TreeExpander (handles indentation and expand/collapse)
        expander = Gtk.TreeExpander(hide_expander=True)



        # Create widgets for icon and name
        icon = Gtk.Image()
        name_label = Gtk.Label()
        name_label.set_xalign(0)
        name_label.set_hexpand(True)

        # container for icon and name
        box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        box.append(icon)
        box.append(name_label)

        # Set box as expander child
        expander.set_child(box)

        # Set expander as list item child
        list_item.set_child(expander)

    @Gtk.Template.Callback()
    def _on_activate(self, _, position, *args, **kwargs):
        """
        Callback for when an item in the ListView is activated (clicked)
        This will toggle the expansion of directories or open files
        """
        # retrieve the activated row
        tree_list_row = cast(Gtk.TreeListRow | None, self.selection_model.get_item(position))

        # if not retrieved, return
        if not tree_list_row:
            return

        # Get the DirectoryItem from the TreeListRow
        directory_item = cast(DirectoryItem, tree_list_row.get_item())

        # If the item is a directory, toggle its expansion
        if directory_item.is_dir:
            expanded = tree_list_row.get_expanded()
            tree_list_row.set_expanded(not expanded)

        # If the item is a file, open it
        # TODO: focus the file if it is already open instead of opening it again
        else:
            path = directory_item.path
            self.activate_action("win.file-path-open", GLib.Variant("s", str(path)))


    @Gtk.Template.Callback()
    def _on_factory_bind(self, _, list_item: Gtk.ListItem):
        """Bind callback - populate widgets with data"""

        # Get the TreeListRow (contains the item and tree info)
        tree_list_row = cast(Gtk.TreeListRow, list_item.get_item())

        # Get our DirectoryItem
        directory_item = cast(DirectoryItem, tree_list_row.get_item())

        # Get widgets
        expander = cast(Gtk.TreeExpander, list_item.get_child())
        box = cast(Gtk.Box, expander.get_child())

        icon = cast(Gtk.Image, box.get_first_child())
        name_label = cast(Gtk.Label, icon.get_next_sibling())

        # Set the TreeListRow for the expander
        expander.set_list_row(tree_list_row)

        # Set icon
        if directory_item.is_dir:
            icon.set_from_icon_name("folder-symbolic")
        else:
            icon.set_from_icon_name("text-x-generic-symbolic")

        # Set labels
        name_label.set_text(directory_item.display_name)
