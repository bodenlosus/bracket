
import sys
import gi
import pathlib
gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
gi.require_version("Panel", "1")

from gi.repository import Gio

def load_resources():
    """Load resources for the ZenNote application."""

    # Define the path to the resource file
    # This assumes the resources are located in a 'resources' directory relative to this file
    # not cleanest solution, but works for now
    # TODO: use a build system to copy the resources to the right place
    resource_path = pathlib.Path(__file__).parent.absolute() / "resources" / "ui.gresource"

    # Check if the resource file exists
    if not resource_path.exists():
        raise FileNotFoundError(f"Resource file not found: {resource_path}")

    # Load the resource file
    resource = Gio.Resource.load(resource_path.as_posix())

    # Register the resource with the Gio.Resource module
    resource._register() # type: ignore

def app():
    """Main entry point for the ZenNote application."""
    load_resources()

    # has to be loaded after the resources are loaded
    # because the resources are used in the UI
    from zennote.app import App
    
    # run the application
    zennote = App()
    exit_status = zennote.run(sys.argv)
    # Exit the application with the status code returned by the run method
    sys.exit(exit_status)

# I doubt that there is a way to call the file directly and make it work (since the project is basically a library), but one may try
if __name__ == "__main__":
    app()
