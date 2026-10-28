"""Pack the registry's .hnmod: horizon-native.json + mixins/<id>/*.{rs,toml}.

Deterministic, mirroring the layout of published bundles: no directory
entries, sorted names, deflate, fixed 2026-01-01 timestamps, Unix 0644.
Refuses to pack when the version or the Horizon pin disagree between
horizon-native.json, horizon-mixin.toml, Cargo.toml and the sources.
Usage: python tools/pack_hnmod.py [OUT]  (default: dist/<id>-<version>.hnmod)
"""
import json
import pathlib
import sys
import tomllib
import zipfile

root = pathlib.Path(__file__).resolve().parent.parent
manifest = json.loads((root / "horizon-native.json").read_text(encoding="utf-8"))
mod = root / "mixins" / manifest["id"]

# The version and the pins must agree everywhere they are written.
mixin = tomllib.loads((mod / "horizon-mixin.toml").read_text(encoding="utf-8"))
cargo = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))
runtime = (mod / "runtime.rs").read_text(encoding="utf-8")
native = (mod / "native.rs").read_text(encoding="utf-8")
version = manifest["version"]
checks = {
    "horizon-mixin.toml [mod] id": mixin["mod"]["id"] == manifest["id"],
    "horizon-mixin.toml [mod] version": mixin["mod"]["version"] == version,
    "Cargo.toml version": cargo["package"]["version"] == version,
    "runtime.rs VERSION": f'pub const VERSION: &str = "{version}";' in runtime,
    "runtime.rs ID": f'pub const ID: &str = "{manifest["id"]}";' in runtime,
    "horizon-mixin.toml horizon_revision":
        mixin["compatibility"]["horizon_revision"] == manifest["horizon"],
    "native.rs header revision": manifest["horizon"] in native.split("\n", 1)[0],
}
wrong = [name for name, ok in checks.items() if not ok]
if wrong:
    sys.exit(f"horizon-native.json disagrees with: {', '.join(wrong)}")
names = ["horizon-native.json"] + [
    f"mixins/{manifest['id']}/{p.name}"
    for p in mod.iterdir()
    if p.is_file() and p.suffix in (".rs", ".toml")
]
out = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else (
    root / "dist" / f"{manifest['id']}-{manifest['version']}.hnmod"
)
out.parent.mkdir(parents=True, exist_ok=True)
with zipfile.ZipFile(out, "w") as z:
    for name in sorted(names):
        info = zipfile.ZipInfo(name, date_time=(2026, 1, 1, 0, 0, 0))
        info.compress_type = zipfile.ZIP_DEFLATED
        info.create_system = 3
        info.external_attr = 0o100644 << 16
        z.writestr(info, (root / name).read_bytes())
print(out)
