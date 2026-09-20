"""Build a self-contained Typst archive and compile an installed-package example.

Run with the development dependency typst==0.15.0. Package contents come only
from the repository; installation verification uses a fresh local namespace.
"""
from pathlib import Path
import gzip
import hashlib
import io
import shutil
import tarfile
import tempfile
import tomllib

import typst

ROOT = Path(__file__).resolve().parents[1]
manifest = tomllib.loads((ROOT / "typst.toml").read_text())["package"]
name, version = manifest["name"], manifest["version"]
examples = {
    "cgrade": '#assert(api.to-de("ch", 5.5) == 1.7)\n#assert(api.from-de("fr", 2.5) == 15.0)\n#assert(api.is-pass("de", 4.3) == false)\n#assert(api.to-de("ch", 3.5) == none)\n#assert(api.convert("ch", "fr", 5.5) == 17.66)\n#assert(api.parse-grade("6,0") == 6.0)\n#assert(api.parse-grade("6,0!") == none)\n#assert(api.format-grade(5.25, 1) == "5.3")\n#assert(api.format-grade(-0.0004, 1) == "0.0")',
}

with tempfile.TemporaryDirectory(prefix=f"{name}-typst-") as tmp:
    stage = Path(tmp) / "source"
    stage.mkdir()
    for item in ("typst.toml", "README.md", "LICENSE", "LICENSES", "typst", "tables"):
        source = ROOT / item
        if source.is_dir():
            shutil.copytree(source, stage / item)
        elif source.is_file():
            shutil.copy2(source, stage / item)
        else:
            raise FileNotFoundError(source)
    archive_bytes = io.BytesIO()
    with gzip.GzipFile(fileobj=archive_bytes, mode="wb", mtime=0) as gz:
        with tarfile.open(fileobj=gz, mode="w") as archive:
            for path in sorted(stage.rglob("*")):
                if not path.is_file():
                    continue
                relative = path.relative_to(stage).as_posix()
                data = path.read_bytes()
                info = tarfile.TarInfo(relative)
                info.size, info.mode, info.mtime = len(data), 0o644, 0
                archive.addfile(info, io.BytesIO(data))
    packed = archive_bytes.getvalue()
    packages = Path(tmp) / "packages"
    installed = packages / "local" / name / version
    installed.mkdir(parents=True)
    with tarfile.open(fileobj=io.BytesIO(packed), mode="r:gz") as archive:
        archive.extractall(installed, filter="data")
    example = Path(tmp) / "main.typ"
    example.write_text(
        f'#import "@local/{name}:{version}" as api\n'
        + examples[name] + '\nPackage import verified.\n', encoding="utf-8"
    )
    pdf = typst.compile(str(example), root=tmp, package_path=str(packages))
    assert pdf.startswith(b"%PDF"), "Compiler did not produce a PDF"
    output = ROOT / "dist" / f"{name}-{version}-typst.tar.gz"
    output.parent.mkdir(exist_ok=True)
    output.write_bytes(packed)
    print(f"Typst installed import verified: {output.name} sha256={hashlib.sha256(packed).hexdigest()}")
