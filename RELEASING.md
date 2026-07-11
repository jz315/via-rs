# Releasing via

All public workspace crates share one release version because they exchange
public `via-core` types and use exact internal dependency versions.

Before publishing:

```powershell
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
python tools/verify_packages.py
```

The official KiCad footprint cache is distributed separately from crates.io.
The default CLI installer expects this release asset shape:

```text
tag:   kicad-footprints-<version>
asset: kicad-footprints-<version>.tar.zst
asset: kicad-footprints-<version>.tar.zst.sha256
```

For the current default:

```powershell
via footprints install --version 10.0.4
```

Publish or refresh the asset with the manual GitHub Actions workflow
`KiCad Footprint Bundle`. It downloads the matching upstream KiCad
`kicad-footprints` tag, imports it into a VIA cache, invokes the maintainer-only
`xtask` bundle command, and uploads both the release asset and its SHA-256
sidecar. Use the workflow `force` input only when deliberately replacing an
existing asset.

For a local release rehearsal:

```powershell
cargo run -p xtask -- footprints bundle --version 10.0.4 --out target\kicad-footprints-10.0.4.tar.zst
```

Publish dependency leaves first and wait for each group to become visible in
the crates.io index before continuing:

```powershell
cargo publish -p via-footprint-ir
cargo publish -p via-kicad-sexp
cargo publish -p via-kicad-footprints

cargo publish -p via-core

cargo publish -p via-footprint
cargo publish -p via-project
cargo publish -p via-kicad
cargo publish -p via-lceda-pro

cargo publish -p via-parts
cargo publish -p via-pcb
cargo publish -p via-pcb-cli
```

`via-examples` is not published. After the dependency crates are indexed, run
`cargo package -p via-pcb-cli` once more as the final registry-backed check.
