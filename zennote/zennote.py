
import sys
import gi
import pathlib
gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
gi.require_version("Panel", "1")

from gi.repository import Gio

def load_resources():
    """Load resources for the ZenNote application."""
    
    resource_path = pathlib.Path(__file__).parent.absolute() / "resources" / "ui.gresource"
    
    if not resource_path.exists():
        raise FileNotFoundError(f"Resource file not found: {resource_path}") 

    resource = Gio.Resource.load(resource_path.as_posix())
    resource._register()

def app():
    """Main entry point for the ZenNote application."""    
    load_resources()
    
    # has to be loaded after the resources are loaded
    # because the resources are used in the UI
    from zennote.app import ZenNote
     
    zennote = ZenNote()
    exit_status = zennote.run(sys.argv)
    sys.exit(exit_status)
    app = ZenNote()
    app.run() 

if __name__ == "__main__":
    app()
