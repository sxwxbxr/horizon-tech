#!/usr/bin/env python3
"""Packs the mod into the two files Horizon takes, byte-for-byte reproducible.

- `target/horizon_tech-<version>.hmod`: the content (horizon-mod.toml, assets/, data/):
  blocks, items, models, textures, recipes, loot, tags, menus. Goes in `horizon-mods/`.
- `target/horizon_tech-<version>.hnmod`: the native code (horizon-native.json and
  mixins/horizon_tech/), which the launcher compiles into the game. Goes in
  `horizon-native-mods/`.

Adapted from horizon-native-mod-template's packager (MIT).
"""
import hashlib
import json
from pathlib import Path
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parents[1]
GAME = "26.3"


def write_zip(out, files):
    out.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(out, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for name, content in sorted(files.items()):
            info = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            archive.writestr(info, content)
    return dict(file=str(out), size=out.stat().st_size, sha256=hashlib.sha256(out.read_bytes()).hexdigest())


def native(compat, mixin):
    mod = mixin["mod"]
    bundle = ROOT / "mixins" / mod["id"]
    if mixin.get("compatibility", {}).get("horizon_revision") != compat["horizon_revision"]:
        raise SystemExit("horizon-mixin.toml and compatibility.json pin different Horizon revisions")
    manifest = dict(schema=1, id=mod["id"], version=mod["version"], game=GAME,
                    horizon=compat["horizon_revision"], mixins=compat["horizon_mixins_revision"],
                    hooks=compat["hooks"])
    files = {"horizon-native.json": (json.dumps(manifest, indent=2, sort_keys=True) + "\n").encode()}
    for path in sorted(bundle.iterdir()):
        if path.is_file() and path.suffix in (".rs", ".toml"):
            files[path.relative_to(ROOT).as_posix()] = path.read_bytes()
    return write_zip(ROOT / "target" / f"{mod['id']}-{mod['version']}.hnmod", files)


def content(mod):
    manifest = tomllib.loads((ROOT / "horizon-mod.toml").read_text())
    if manifest["id"] != mod["id"] or manifest["version"] != mod["version"]:
        raise SystemExit("horizon-mod.toml and horizon-mixin.toml name different ids or versions")
    files = {"horizon-mod.toml": (ROOT / "horizon-mod.toml").read_bytes()}
    for top in ("assets", "data"):
        for path in sorted((ROOT / top).rglob("*")):
            if path.is_file():
                files[path.relative_to(ROOT).as_posix()] = path.read_bytes()
    return write_zip(ROOT / "target" / f"{mod['id']}-{mod['version']}.hmod", files)


def main():
    compat = json.loads((ROOT / "compatibility.json").read_text())
    manifests = sorted((ROOT / "mixins").glob("*/horizon-mixin.toml"))
    if len(manifests) != 1:
        raise SystemExit("keep one mixins/<id>/horizon-mixin.toml")
    mixin = tomllib.loads(manifests[0].read_text())
    if manifests[0].parent.name != mixin["mod"]["id"]:
        raise SystemExit("the bundle folder must be named after the mod id")
    print(json.dumps([content(mixin["mod"]), native(compat, mixin)], indent=2))


if __name__ == "__main__":
    main()
