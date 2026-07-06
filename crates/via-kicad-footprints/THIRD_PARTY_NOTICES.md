# Third-Party Notices

## KiCad official footprint library

- Upstream project: KiCad official footprint libraries
- Upstream repository: https://gitlab.com/kicad/libraries/kicad-footprints
- Intended asset bundle: `kicad-footprints-10.0.4.tar.zst`
- Intended manifest: `manifest.json`
- KiCad version used for the current cache/bundle target: `10.0.4`
- License: `CC-BY-SA-4.0 WITH KiCad-libraries-exception`

This crate does not vendor the official footprint files into the crates.io
package. Official KiCad `.kicad_mod` files are imported into a local cache with
`via footprints import` or fetched from a separate release asset with
`via footprints fetch`.

Project-specific official footprint selections and verification notes belong in
downstream project crates and generated project documentation.
