# via-kicad-footprints

KiCad official footprint cache and manifest helpers for VIA.

This crate does not bundle the full KiCad official footprint library, and it
does not bundle project-required `.kicad_mod` files. It provides:

- manifest data types for GitHub Release footprint bundles;
- cache path resolution and SHA256 validation;
- import helpers for a local KiCad installation;
- installer helpers for versioned release assets;
- bundle helpers for maintainers publishing a cache archive;
- generic footprint lookup and copy helpers keyed by KiCad library/name.

KiCad official footprint files are licensed under
`CC-BY-SA-4.0 WITH KiCad-libraries-exception`; see
`KICAD_FOOTPRINT_LICENSE.md` and `THIRD_PARTY_NOTICES.md`.

Default cache path:

- Windows: `%LOCALAPPDATA%\via\kicad-footprints\<version>\`
- Other platforms: `$XDG_CACHE_HOME/via/kicad-footprints/<version>/` or
  `~/.cache/via/kicad-footprints/<version>/`

Set `VIA_KICAD_FOOTPRINTS_DIR` to override the cache directory.

User install:

```powershell
via footprints install --version 10.0.4
```

The default release URL is:

```text
https://github.com/jz315/via-rs/releases/download/kicad-footprints-10.0.4/kicad-footprints-10.0.4.tar.zst
```

Maintainer bundle flow:

```powershell
via footprints import --version 10.0.4 --from "<KiCad kicad-footprints checkout>" --upstream-source "<source archive URL>"
cargo run -p xtask -- footprints bundle --version 10.0.4 --out kicad-footprints-10.0.4.tar.zst
```
