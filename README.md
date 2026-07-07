<p align="center">
  <img src="assets/via-logo.svg" alt="via" width="520">
</p>

<p align="center">
  <a href="Cargo.toml"><img alt="version" src="https://img.shields.io/badge/version-0.1.1-f05a28?style=for-the-badge"></a>
  <a href="Cargo.toml"><img alt="Rust 2024" src="https://img.shields.io/badge/Rust-2024-4b4f56?style=for-the-badge&logo=rust&logoColor=white"></a>
  <a href="README.md#cli"><img alt="KiCad export" src="https://img.shields.io/badge/KiCad-export-314cb6?style=for-the-badge"></a>
  <a href="README.md#cli"><img alt="LCEDA Pro export" src="https://img.shields.io/badge/LCEDA%20Pro-export-00a3a3?style=for-the-badge"></a>
  <a href="TUTORIAL.md"><img alt="Tutorial" src="https://img.shields.io/badge/docs-tutorial-0a7f42?style=for-the-badge"></a>
  <a href="LICENSE"><img alt="License MPL-2.0" src="https://img.shields.io/badge/license-MPL--2.0-6a35ff?style=for-the-badge"></a>
</p>

# via

## Design circuit boards with Rust

`via` is a Rust-native circuit authoring toolkit. It lets you describe a board
as normal Rust data, validate pin maps and nets, and export reviewable KiCad /
LCEDA artifacts.

## Why via

- Reusable board modules instead of copy-pasted schematic fragments.
- Typed nets, rails, pins, footprints, and electrical classes.
- Validation before export, including pin maps, pad bindings, and connectivity.
- KiCad and LCEDA Pro export paths for review, hand layout, and iteration.
- Rust tests for hardware design assumptions.

## Get Started

Add `via-pcb` to your Rust project:

```toml
[dependencies]
via = { package = "via-pcb", version = "0.1.1" }
```

The crates.io package is `via-pcb`, while the Rust crate name is still `via`:

```rust
use via::prelude::*;

pub fn board() -> Result<Board> {
    let mut d = Design::new("demo")
        .rules(Rules::new())
        .units(Unit::Mm);

    let signal = d.signal("SIGNAL", "3V3");
    let v3v3 = d.rail("3V3").dc(3.3);
    let ground = d.ground("GND");

    let j1 = d.add(
        part("J1", "3-pin header")
            .footprint(fp::pin_1x03())
            .symbol(sym::connector().left(["SIG", "3V3", "GND"]))
            .pin(pin("SIG").logic("3V3").pad("1"))
            .pin(pin("3V3").power("3V3").pad("2"))
            .pin(pin("GND").ground().pad("3")),
    )?;

    d.connect(&signal, [j1.pin("SIG")]);
    d.connect(&v3v3, [j1.pin("3V3")]);
    d.connect(&ground, [j1.pin("GND")]);

    d.check(CheckProfile::Prototype)?;
    d.finish()
}
```

For exporter workflows, install the CLI package. The package name is
`via-pcb-cli`, and the installed command is `via`:

```powershell
cargo install via-pcb-cli
via check <design-name>
via export kicad <design-name>
```

## Alternative Dependency Sources

Use the Git repository directly:

```toml
[dependencies]
via = { package = "via-pcb", git = "https://github.com/jz315/via-rs.git" }
```

For local development, use a path dependency:

```toml
[dependencies]
via = { package = "via-pcb", path = "../via-rs/crates/via" }
```

## Workspace

- `via-pcb`: user-facing package; its Rust crate name is `via` and it provides
  `via::prelude::*`.
- `via-core`: boards, modules, pins, nets, footprints, rules, and diagnostics.
- `via-parts`: generic reusable parts such as resistors and capacitors.
- `via-footprint`: high-level generated footprint builders.
- `via-footprint-ir`: low-level footprint geometry IR for custom generators.
- `via-kicad`: KiCad netlist, footprint, schematic, and PCB helpers.
- `via-lceda-pro`: LCEDA Pro package export.
- `via-project`: `via.toml` project loading and external design providers.
- `via-examples`: generic examples for tests and documentation snippets.
- `via-pcb-cli` (`crates/via-cli`): project-oriented command-line wrapper for
  checks, snapshots, BOMs, and export. It installs the `via` binary.

## Minimal Example

```rust
use via::prelude::*;

pub fn board() -> Result<Board> {
    let mut d = Design::new("modern_api_minimal")
        .rules(Rules::new())
        .units(Unit::Mm);

    let signal = d.signal("SIGNAL", "3V3");
    let v3v3 = d.rail("3V3").dc(3.3);
    let ground = d.ground("GND");

    let input = d.add(
        part("J1", "External 3.3V signal input")
            .footprint(fp::pin_1x03())
            .symbol(sym::connector().left(["SIG", "3V3", "GND"]))
            .pin(pin("SIG").logic("3V3").pad("1"))
            .pin(pin("3V3").power("3V3").pad("2"))
            .pin(pin("GND").ground().pad("3")),
    )?;

    let load = d.add(
        part("U1", "Demo load")
            .footprint(fp::pin_1x03())
            .symbol(sym::module().left(["IN"]).right(["VCC", "GND"]))
            .pin(pin("IN").logic("3V3").pad("1"))
            .pin(pin("VCC").power("3V3").pad("2"))
            .pin(pin("GND").ground().pad("3")),
    )?;

    d.connect(&signal, [input.pin("SIG"), load.pin("IN")]);
    d.connect(&v3v3, [input.pin("3V3"), load.pin("VCC")]);
    d.connect(&ground, [input.pin("GND"), load.pin("GND")]);

    d.check(CheckProfile::Prototype)?;
    d.finish()
}
```

## Main Ideas

`Part` is the electrical object in the schematic model. It owns logical pins,
electrical classes, symbol placement hints, and the footprint reference.

`Symbol` is only the schematic drawing style. It decides where logical pins are
shown on the generated schematic. It does not create nets or hidden pins.

`Footprint` is the PCB-side physical model. It names or embeds pads and
geometry. A logical pin may map to a physical pad with `.pad("1")`.

`Design` is the authoring surface. It creates nets, adds components, connects
pins, runs checks, and produces a checked `Board`.

`Board` is the read-only result used by exporters, snapshots, and tests.

## Footprints

Normal users should prefer parts that already carry footprints:

```rust
use via::prelude::*;

let r1 = design.add(parts::resistor("R1").value(1.kohm()).fp(fp::r0805()))?;
let c1 = design.add(parts::capacitor("C1").value(100.nf()).voltage(50.v()))?;
```

For custom parts, attach either an embedded generated footprint or an external
KiCad footprint name:

```rust
let j1 = design.add(
    part("J1", "Debug header")
        .footprint(fp::pin_1x04())
        .symbol(sym::connector().left(["1", "2"]).right(["3", "4"]))
        .pins(["1", "2", "3", "4"]),
)?;

let u1 = design.add(
    part("U1", "Vendor module")
        .footprint("Vendor_Module_From_KiCad_Library")
        .pins(["VIN", "GND", "OUT"]),
)?;
```

`via-footprint` contains only common, generic footprint builders:

- SMD passives: `R_0402`, `R_0603`, `R_0805`, `R_1206`, and matching
  capacitors.
- Radial capacitors: generic verify builders for D5.0/P2.0, D6.3/P2.50,
  D8.0/P3.5, and D10.0/P5.0. Downstream projects can bind specific KiCad
  official footprints through explicit footprint asset metadata.
- Pin headers and sockets: `Pin_1x*`, `Pin_2x*`, `Socket_2x08_R12.7`.
- Board connectors: terminal blocks, XH, PH.
- Debug and mechanical footprints: test pads, fiducials, mounting holes.
- Small semiconductors: LED, SOD, SOT, SOIC, TSSOP.

Measured dev boards, purchased modules, and product-specific connector drawings
belong in downstream part crates.

## CLI

A downstream project is driven by `via.toml`, not by compiled-in examples.
The CLI reads a design provider, receives stable `BoardIr` JSON, then runs
checks or exporters. Replace the placeholder values below in your project.

```toml
[project]
name = "<project-name>"
version = "<project-version>"
default-design = "<design-name>"

[designs."<design-name>"]
provider = "cargo"
package = "<provider-package>"
command = "<provider-command>"

[outputs.kicad]
dir = "<kicad-output-dir>"
project = "<kicad-project-name>"
footprint-library-name = "<kicad-footprint-library-name>"
footprint-library-path = "<kicad-footprint-library-path>"
footprint-output-dir = "<generated-footprint-output-dir>"

[kicad-footprints]
version = "10.0.4"
source = "github-release"
```

The provider command prints the board IR:

```rust
use via::prelude::*;

fn main() -> Result<()> {
    let board = project_design::board()?;
    via::project::emit_ir(&board)
}
```

Project commands:

```powershell
cargo install via-pcb-cli
via ir <design-name> --out <board-ir-json>
via check <design-name>
via check <design-name> --production
via snapshot <design-name> --out <snapshot-json>
via bom <design-name> --format csv --out <bom-csv>
via footprints status
via footprints import --version 10.0.4 --from "<KiCad footprints dir>"
via footprints fetch --url "<cache-bundle-url>" # experimental
via export kicad <design-name>
via export lceda-pro <design-name> --out <lceda-package>
via export pcb <design-name> --layout <layout-json> --out <kicad-pcb> # experimental
```

## Contribution

```powershell
git clone https://github.com/jz315/via-rs.git
cd via-rs
cargo fmt --check
cargo test --workspace
```


## License

`via-pcb` is licensed under the Mozilla Public License 2.0.

Official KiCad footprint assets imported or fetched through
`via-kicad-footprints` remain under
`CC-BY-SA-4.0 WITH KiCad-libraries-exception`; see that crate's third-party
notices.
