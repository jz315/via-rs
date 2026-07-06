# VIA Tutorial

This tutorial explains the generic `via` API. It deliberately avoids any
project-specific board, module, or measured connector. The point of `via-rs` is
to be a reusable library; real products should sit in downstream crates.

## 1. The Smallest Useful Board

A VIA board starts with a `Design`. A `Design` is mutable authoring state: you
add parts, define nets, connect pins, and then ask VIA to check the result.

```rust
use via::prelude::*;

pub fn board() -> Result<Board> {
    let mut d = Design::new("lesson_01").units(Unit::Mm);

    let v3v3 = d.rail("3V3").dc(3.3);
    let gnd = d.ground("GND");
    let sig = d.signal("SIG", "3V3");

    let j1 = d.add(
        part("J1", "Input connector")
            .footprint(fp::pin_1x03())
            .symbol(sym::connector().left(["SIG", "3V3", "GND"]))
            .pin(pin("SIG").logic("3V3").pad("1"))
            .pin(pin("3V3").power("3V3").pad("2"))
            .pin(pin("GND").ground().pad("3")),
    )?;

    let u1 = d.add(
        part("U1", "Demo load")
            .footprint(fp::pin_1x03())
            .symbol(sym::module().left(["IN"]).right(["VCC", "GND"]))
            .pin(pin("IN").logic("3V3").pad("1"))
            .pin(pin("VCC").power("3V3").pad("2"))
            .pin(pin("GND").ground().pad("3")),
    )?;

    d.connect(&sig, [j1.pin("SIG"), u1.pin("IN")]);
    d.connect(&v3v3, [j1.pin("3V3"), u1.pin("VCC")]);
    d.connect(&gnd, [j1.pin("GND"), u1.pin("GND")]);

    d.finish()
}
```

Three things are worth noticing.

First, the net names are declared before the physical connections. This keeps
intent readable. `v3v3`, `gnd`, and `sig` are handles, not magic strings being
scattered everywhere.

Second, every logical pin declares its electrical class. `power`, `ground`,
`logic`, and `passive` are not decoration; they are data the checker can use.

Third, every logical pin maps to a physical pad with `.pad("...")`. That is the
bridge between schematic intent and PCB reality.

## 2. Part, Symbol, Footprint, Pad

Beginners often mix four concepts together. VIA keeps them separate.

`Part` is the electrical object in your design: a resistor, connector, IC, or
module. It owns the reference designator, value/description, logical pins, and
metadata.

`Symbol` is the schematic drawing style. It says where logical pins are drawn on
the page. It does not create connections.

`Footprint` is the PCB object. It has pads, holes, silkscreen, fab outline, and
courtyard geometry.

`Pad` is a physical copper target inside a footprint. Pads are identified by
numbers or names such as `"1"`, `"2"`, `"A1"`, or `"GND"`.

The important mapping is:

```rust
pin("VCC").power("3V3").pad("2")
```

This means: the logical pin named `VCC` is a 3.3V power pin, and when exporting
to PCB tools it corresponds to physical pad `2`.

## 3. Passive Parts

Common passives are available through `via-parts` and re-exported by
`via::prelude::*`.

```rust
let r1 = d.add(parts::resistor("R1").value(1.kohm()).fp(fp::r0805()))?;
let c1 = d.add(parts::capacitor("C1").value(100.nf()).voltage(50.v()))?;
```

The default resistor/capacitor builders attach simple two-pin logical models.
You can override the footprint with `.fp(...)`.

Units such as `kohm`, `nf`, `uf`, and `v` are convenience methods. They are not
required, but they make code harder to misread.

## 4. Custom Parts

Use `part(...)` when the generic library does not already provide the component.

```rust
let u1 = d.add(
    part("U1", "I2C sensor")
        .footprint(fp::pin_1x04())
        .symbol(sym::module().left(["VCC", "GND"]).right(["SCL", "SDA"]))
        .pin(pin("VCC").power("3V3").pad("1"))
        .pin(pin("GND").ground().pad("2"))
        .pin(pin("SCL").logic("3V3").pad("3"))
        .pin(pin("SDA").logic("3V3").pad("4")),
)?;
```

If the component is used more than once, wrap it in a constructor and return a
typed handle.

```rust
#[derive(Debug, Clone)]
pub struct I2cSensor {
    id: ModuleId,
}

impl I2cSensor {
    pub fn vcc(&self) -> PinRef { self.id.pin("VCC") }
    pub fn gnd(&self) -> PinRef { self.id.pin("GND") }
    pub fn scl(&self) -> PinRef { self.id.pin("SCL") }
    pub fn sda(&self) -> PinRef { self.id.pin("SDA") }
}

pub fn i2c_sensor(refdes: &str) -> impl Component<Output = I2cSensor> {
    part(refdes, "I2C sensor")
        .footprint(fp::pin_1x04())
        .symbol(sym::module().left(["VCC", "GND"]).right(["SCL", "SDA"]))
        .pin(pin("VCC").power("3V3").pad("1"))
        .pin(pin("GND").ground().pad("2"))
        .pin(pin("SCL").logic("3V3").pad("3"))
        .pin(pin("SDA").logic("3V3").pad("4"))
        .handle(|id| I2cSensor { id })
}
```

Now board code reads like the circuit:

```rust
let sensor = d.add(i2c_sensor("U1"))?;
d.connect(&v3v3, [sensor.vcc()]);
d.connect(&gnd, [sensor.gnd()]);
d.connect(&scl, [sensor.scl()]);
d.connect(&sda, [sensor.sda()]);
```

## 5. Footprint Authoring

Use `via-footprint` for common generated footprints:

```rust
let fp = via_footprint::generators::tht_header_1x("Pin_1x06_P2.54", 6)
    .drill(1.0)
    .pad_diameter(1.7)
    .build();
```

Use `via-footprint-ir` only when the high-level builder cannot express the
geometry:

```rust
use via_footprint_ir::{FootprintIr, Pad, PadShape, Point, Size};

let mut fp = FootprintIr::new("Custom_Test_Footprint");
fp.add_pad(Pad::smd(
    "1",
    PadShape::Rect,
    Point::new(0.0, 0.0),
    Size::new(1.2, 1.6),
    ["F.Cu", "F.Paste", "F.Mask"],
));
```

Only put broadly reusable families in `via-footprint`. Measured product modules
belong in downstream part crates.

## 6. Checks

During authoring, use:

```rust
d.check(CheckProfile::Prototype)?;
```

Before using exported artifacts for real manufacturing work, use the stricter
production profile:

```rust
d.check(CheckProfile::Production)?;
```

Prototype checks answer: "Is the design structurally coherent?"

Production checks answer: "Have the risky assumptions been resolved?"

Generated or measured-later footprints should stay marked as verify-required
until the exact purchased part has been checked.

## 7. Export Workflow

For a real project, add a `via.toml` next to the downstream Cargo workspace and
point it at a design provider. The provider is just an external command that
prints stable `BoardIr` JSON to stdout. Replace the placeholder values below in
your project.

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
```

The provider binary can stay tiny:

```rust
use via::prelude::*;

fn main() -> Result<()> {
    let board = project_design::board()?;
    via::project::emit_ir(&board)
}
```

Then run the project workflow:

```powershell
cargo run -p via-cli -- build --out <board-ir-json>
cargo run -p via-cli -- check <design-name>
cargo run -p via-cli -- check-production <design-name>
cargo run -p via-cli -- inspect <design-name> --out <snapshot-json>
cargo run -p via-cli -- bom <design-name> --format csv --out <bom-csv>
cargo run -p via-cli -- export kicad
```

The intended loop is:

1. Write or update the Rust design.
2. Run checks.
3. Export schematic/netlist/snapshot.
4. Preview or place/rout in an editor.
5. Export a KiCad PCB draft.
6. Use KiCad for final DRC, Gerber generation, and manufacturing review.

VIA is allowed to produce drafts. It should not pretend a draft is ready to
order.

## 8. Downstream Libraries

A downstream project should depend on `via` and define its own part crate:

```text
my-board/
  Cargo.toml
  crates/
    my-parts/
    my-patterns/
  src/
    board.rs
```

That crate can define measured modules, exact footprint geometry, vendor
metadata, and reusable circuit patterns. The generic `via-rs` workspace should
not need to know those names.

That boundary is important. If a local module enters `via-footprint` too early,
every user sees it as a blessed generic footprint, even though it may only be
correct for one purchased part or one drawing. Keep generic code generic; keep
product assumptions close to the product.
