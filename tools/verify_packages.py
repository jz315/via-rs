#!/usr/bin/env python3
"""Package every workspace crate and compile the tarballs against one another."""

from __future__ import annotations

import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tomllib


ROOT = Path(__file__).resolve().parents[1]
PACKAGE_DIR = ROOT / "target" / "package"
VERIFY_DIR = PACKAGE_DIR / "verify-workspace"
VERIFY_TARGET = PACKAGE_DIR / "verify-target"


def run(*args: str, cwd: Path = ROOT, env: dict[str, str] | None = None) -> None:
    subprocess.run(args, cwd=cwd, env=env, check=True)


def workspace_packages() -> list[tuple[str, str]]:
    with (ROOT / "Cargo.toml").open("rb") as handle:
        workspace = tomllib.load(handle)
    workspace_version = workspace["workspace"]["package"]["version"]
    packages: list[tuple[str, str]] = []
    for member in workspace["workspace"]["members"]:
        with (ROOT / member / "Cargo.toml").open("rb") as handle:
            manifest = tomllib.load(handle)
        package = manifest["package"]
        version = package.get("version", workspace_version)
        if isinstance(version, dict):
            version = workspace_version
        if package.get("publish") is False:
            continue
        packages.append((package["name"], version))
    return packages


def extract_package(archive: Path) -> None:
    with tarfile.open(archive, "r:gz") as package:
        try:
            package.extractall(VERIFY_DIR, filter="data")
        except TypeError:
            package.extractall(VERIFY_DIR)


def write_patch_config(packages: list[tuple[str, str]]) -> None:
    config_dir = VERIFY_DIR / ".cargo"
    config_dir.mkdir(parents=True, exist_ok=True)
    lines = ["[patch.crates-io]"]
    for name, version in packages:
        path = (VERIFY_DIR / f"{name}-{version}").as_posix()
        lines.append(f"{json.dumps(name)} = {{ path = {json.dumps(path)} }}")
    (config_dir / "config.toml").write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> None:
    packages = workspace_packages()
    shutil.rmtree(VERIFY_DIR, ignore_errors=True)
    shutil.rmtree(VERIFY_TARGET, ignore_errors=True)
    package_command = ["cargo", "package", "--no-verify", "--allow-dirty", "--quiet"]
    for name, _version in packages:
        package_command.extend(["-p", name])
    run(*package_command)
    VERIFY_DIR.mkdir(parents=True)
    for name, version in packages:
        archive = PACKAGE_DIR / f"{name}-{version}.crate"
        if not archive.is_file():
            raise FileNotFoundError(f"cargo did not create {archive}")
        extract_package(archive)
    write_patch_config(packages)

    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(VERIFY_TARGET)
    for name, version in packages:
        manifest = VERIFY_DIR / f"{name}-{version}" / "Cargo.toml"
        run(
            "cargo",
            "check",
            "--quiet",
            "--manifest-path",
            str(manifest),
            "--all-targets",
            "--all-features",
            "--offline",
            cwd=VERIFY_DIR,
            env=env,
        )


if __name__ == "__main__":
    main()
