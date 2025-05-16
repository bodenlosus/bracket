import json
import pathlib
import re
from typing import Any, Literal, get_args

WindowActions = Literal[
        "new-file",
        "save-file",
        "save-file-as",
        "open-file",
        "close-file",
]

ACTIONS: dict[str, Any] = {
    "win": WindowActions
}

type KeyMap = dict[str, str]


def parse_accels_json(config_string: str) -> KeyMap:
    global ACTIONS

    raw = json.loads(config_string)

    if not isinstance(raw, dict):
        return dict()

    # some sort of type black magic...
    allowed_actions: dict[str, set[str]] = {
        scope: {v for v in get_args(actions)}
        for scope, actions in ACTIONS.items()
    }

    keybind_pattern = re.compile(r"^(<[A-Z][A-Za-z]*>)+[a-zA-Z0-9]$")

    res: KeyMap = dict()

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

        res[key] = value

    return res

def load_accels_json() -> KeyMap:
    path = pathlib.Path(__file__).parent.resolve() / "keymap.json"
    if not path.is_file():
        print("file not found")
        return dict()

    with path.open("r") as f:
        return parse_accels_json(f.read())
