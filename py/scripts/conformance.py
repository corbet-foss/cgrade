"""Run canonical vectors against either the source port or an installed wheel."""
import importlib
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
if "--installed" not in sys.argv:
    sys.path.insert(0, str(ROOT / "py"))
api = importlib.import_module("cgrade")

ARGUMENTS = {
    "available_scales": [],
    "is_supported": ["sys"],
    "scale": ["sys"],
    "is_pass": ["sys", "value"],
    "to_de": ["sys", "value"],
    "from_de": ["sys", "value"],
    "convert": ["from_sys", "to_sys", "value"],
    "bavarian_to_de": ["nmax", "nmin", "value"],
    "bavarian_from_de": ["nmax", "nmin", "value"],
    "parse_grade": ["text"],
    "format_grade": ["value", "decimals"],
}

RENAMED = {"convert": {"from_sys": "from", "to_sys": "to"}}

files = sorted((ROOT / "tests/vectors").glob("*.json"))
assert files, "No conformance vectors"
count = 0
for file in files:
    for vector in json.loads(file.read_text(encoding="utf-8")):
        name = vector["fn"]
        renamed = RENAMED.get(name, {})
        values = {key: vector.get(renamed.get(key, key)) for key in ARGUMENTS[name]}
        # "value" doubles as the German input of from_de/bavarian_from_de.
        fn = getattr(api, name)
        actual = fn(*(values[key] for key in ARGUMENTS[name]))
        if isinstance(actual, tuple):
            actual = list(actual)
        if actual != vector["expected"]:
            raise AssertionError(f"{file.name} :: {vector['name']}: {actual!r} != {vector['expected']!r}")
        count += 1
print(f"Python cgrade: {count} vectors passed across {len(files)} files")
