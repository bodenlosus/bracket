import json
import pathlib
import re
from typing import Any, Literal, get_args

NEW_FILE = "new-file"
SAVE_FILE = "save-file"
SAVE_FILE_AS = "save-file-as"
OPEN_FILE = "open-file"
CLOSE_FILE = "close-file"
# Define the actions that can be performed in the application on a window level


WINDOW_ACTIONS = [
    NEW_FILE,
    SAVE_FILE,
    SAVE_FILE_AS,
    OPEN_FILE,
    CLOSE_FILE
]

ACTIONS: dict[str, list[str]] = {
    "win": WINDOW_ACTIONS,
}

type KeyMap = dict[str, str]

# Parse a JSON string containing key bindings for actions in the application.
def parse_accels_json(config_string: str) -> KeyMap:
    """
    Parses a JSON string containing key bindings for actions in the application.
    input may look like this:
    {
        "win.new-file": "<Ctrl>n",
        "win.save-file": "<Ctrl>s",
        "app.open-file": "<Ctrl>o",
        "app.close-file": "<Ctrl>w"
    }
    The keys are in the format "scope.action" where scope is the context (e.g., "win" for window actions)
    """
    global ACTIONS

    # Parse the JSON string and validate its structure.
    # Return empty dictionary if parsing fails or if the structure is invalid.
    raw = None
    try:
        
        raw = json.loads(config_string)
    except json.JSONDecodeError as e:
        print(f"JSON decode error: {e}")
        return dict()

    # Validate that the parsed JSON is a dictionary.
    if not isinstance(raw, dict):
        return dict()

    # Define the allowed actions for each scope.
    # will look something like this:
    # {
    #     "win": {"new-file", "save-file", ...},
    # }
    allowed_actions: dict[str, set[str]] = {
        scope: {v for v in actions}
        for scope, actions in ACTIONS.items()
    }

    # Define a regex pattern to validate keybinds.
    # The pattern matches a sequence of angle-bracketed words followed by an alphanumeric character.
    # Example: "<Ctrl>Save" or "<Alt><Shift>Open"
    keybind_pattern = re.compile(r"^(<[A-Z][A-Za-z]*>)+[a-zA-Z0-9]$")

    res: KeyMap = dict()
    # Iterate through the raw JSON data and validate each key-value pair.
    # Each key should be in the format "scope.action" and each value should match the keybind pattern.
    # If any validation fails, print an error message and skip that key-value pair.
    for key, value in raw.items():
        key: str
        sep = key.split(".")

        if len(sep) != 2:
            print(f"invalid key format: {key}")
            continue

        (scope, action) = sep
        if not scope in allowed_actions:
            print(f"invalid scope: {scope} in {key}")

        if not action in allowed_actions[scope]:
            print(f"invalid action: {action} in {key}")
            continue

        elif not re.match(keybind_pattern, value):
            print(f"invalid keybind {value}")
            continue
        
        # If all validations pass, add the key-value pair to the result dictionary.
        res[key] = value

    # Return the validated keymap.
    # will look something like this:
    # {
    #     "win.new-file": "<Ctrl>n",
    #     "app.save-file": "<Ctrl>s",
    # }
    return res

# Load the keymap from a JSON file and parse it.
# TODO: Somehow specify the path to the JSON file externally instead of hardcoding it.
def load_accels_json() -> KeyMap:
    """
    Loads key bindings from a JSON file and parses them into a dictionary.
    Returns a KeyMap: A dictionary mapping action keys to their corresponding key bindings.
    If the file does not exist or cannot be parsed, returns an empty dictionary.
    """
    # Get the path to the JSON file containing key bindings. Hardcoded for simplicity.
    path = pathlib.Path(__file__).parent.resolve() / "keymap.json"
    
    # Check if the file exists. If not, print an error message and return an empty dictionary.
    if not path.is_file():
        print(f"file for keybindsings not found ${path}")
        return dict()

    # Open the file and read its contents, then parse the JSON string.
    with path.open("r") as f:
        return parse_accels_json(f.read())
