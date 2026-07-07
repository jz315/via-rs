# via-kicad-footprints

KiCad official footprint cache and manifest helpers for VIA.

This crate does not bundle the full KiCad official footprint library, and it
does not bundle project-required `.kicad_mod` files. It provides:

- manifest data types for GitHub Release footprint bundles;
- cache path resolution and SHA256 validation;
- import helpers for a local KiCad installation;
- generic footprint lookup and copy helpers keyed by KiCad library/name.

KiCad official footprint files are licensed under
`CC-BY-SA-4.0 WITH KiCad-libraries-exception`; see
`KICAD_FOOTPRINT_LICENSE.md` and `THIRD_PARTY_NOTICES.md`.

Default cache path:

- Windows: `%LOCALAPPDATA%\via\kicad-footprints\<version>\`
- Other platforms: `$XDG_CACHE_HOME/via/kicad-footprints/<version>/` or
  `~/.cache/via/kicad-footprints/<version>/`

Set `VIA_KICAD_FOOTPRINTS_DIR` to override the cache directory.
