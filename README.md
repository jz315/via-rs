# via

`via` is an experiment in code-to-PCB tooling: describe circuits directly in
Rust, validate the design intent, and export reviewable KiCad artifacts.
Today it can export schematic, netlist, report, and footprint artifacts; it does
also export a KiCad PCB draft from a VIA layout JSON.

The goal is not to replace KiCad layout. The goal is to make electronics design
diffable, reusable, and CI-checkable before KiCad takes over for schematic
review, footprint verification, manual placement, routing, DRC, and Gerbers.

Start with [TUTORIAL.md](TUTORIAL.md) for the current `Design` / `part(...)` /
`pin(...)` workflow.

## Workspace Shape

- `via`: user-facing facade crate and `via::prelude::*` entry point.
- `via-core`: board, part, pin, net, footprint-pad facts, and validation.
- `via-parts`: generic reusable parts such as resistors and capacitors.
- `via-footprint`: high-level `.kicad_mod` footprint generators.
- `via-footprint-ir`: advanced low-level Footprint IR for custom generators.
- `via-kicad`: KiCad footprint parsing and netlist export.
- `via-lceda-pro`: LCEDA Pro `.epro2` export from the same board model.
- `via-parts-harmonic`: reusable measured module definitions from this project.
- `via-patterns-harmonic`: reusable project-level input power and switch patterns.
- `via-patterns-motion`: reusable higher-level motion-control circuit patterns.
- `via-examples`: project/example boards built from the public library crates.
- `via-cli`: thin check/export wrapper for examples and CI.

Part libraries are intentionally normal Rust crates. A third party should be
able to publish `via-parts-foo` without changing `via-core`.

Footprint generators are intentionally separate from part libraries. A part
crate can bind to a KiCad/library footprint by name, or call `via-footprint` to
generate a deterministic `.kicad_mod`. Normal users should use generator
builders; advanced users can depend on `via-footprint-ir` when they really need
to construct custom pads, graphics, and text directly.

```rust
use via_footprint::generators::tht_header_2x;

let fp = tht_header_2x("Socket_2x08", 8)
    .row_spacing(12.7)
    .try_build()?;

fp.write_kicad_mod("Socket_2x08.kicad_mod")?;
```

```rust
use via_footprint_ir::{FootprintIr, Pad, PadShape, Point, Size};

let mut fp = FootprintIr::new("Custom_Module");
fp.add_pad(Pad::thru_hole(
    "1",
    PadShape::Rect,
    Point::new(0.0, 0.0),
    Size::new(1.8, 1.8),
    1.0,
));
fp.write_kicad_mod("Custom_Module.kicad_mod")?;
```

Generated footprints carry KiCad properties such as `VIA_GENERATOR`,
`VIA_SOURCE`, `VIA_VERIFY`, `VIA_VERIFICATION_STATUS`, and `VIA_NOTES`. Treat
`VIA_VERIFY=true` as a fabrication gate: the footprint is useful for review and
layout, but it still needs physical measurement against the exact purchased
part before ordering boards. `generated` means produced by code; `measured` or
manual footprints remain outside the generic generator layer until their drawing
and measurement process is captured.

The harmonic parts crate currently generates these reusable footprints:

- `ESP32-S3-N16R8_DevBoard_2x22_P2.54_Row25.40`
- `SilentStepStick_TMC2209_v20_CarrierSocket_2x8_Row12p70`
- `BuckModule_4Wire_MP1584_Adapter`
- `XH2p54_1x04_Vertical_THT_VERIFY`
- `TerminalBlock_1x02_P5.08`
- `DC005_5p5x2p1_RightAngle_THT_Drawing_2_3_4_VERIFY`
- `TerminalBlock_1x05_P5.08`
- `PinHeader_1x02_P2.54`
- `PinHeader_1x08_P2.54`
- `R_0603_1608Metric`
- `R_0805_2012Metric`
- `C_0603_1608Metric`
- `C_0805_2012Metric`
- `CP_Radial_D6p3_P2p50_VERIFY`

Drawing-based mechanical connector footprints such as the DC-005 barrel jack are
now generated through Rust Footprint IR, but remain explicit `VERIFY` footprints
until measured against the purchased part.

## Authoring Shape

New user-facing code should start from the `via` facade crate:

```rust
use via::prelude::*;

fn board() -> Result<Board> {
    let mut d = Design::new("demo_board")
        .rules(Rules::new())
        .units(Unit::Mm);

    let signal = d.logic("SIGNAL", "3V3");
    let v3v3 = d.power("3V3", Voltage::dc(3.3));
    let gnd = d.ground("GND");

    let header = d.add(
        part("J1", "External signal input")
            .footprint("Header_1x03")
            .pin(pin("SIG").logic("3V3"))
            .pin(pin("3V3").power("3V3"))
            .pin(pin("GND").ground()),
    )?;
    let load = d.add(
        part("U1", "Demo load")
            .footprint("Demo_Load_3Pin")
            .pin(pin("IN").logic("3V3"))
            .pin(pin("VCC").power("3V3"))
            .pin(pin("GND").ground()),
    )?;

    signal.connect_all(&mut d, [header.pin("SIG"), load.pin("IN")]);
    v3v3.connect_all(&mut d, [header.pin("3V3"), load.pin("VCC")]);
    gnd.connect_all(&mut d, [header.pin("GND"), load.pin("GND")]);

    d.check(CheckProfile::Prototype)?;
    d.build()
}
```

`Design` is the modern facade over the current checked board model. `NetHandle`
values such as `signal`, `v3v3`, and `gnd` are lightweight handles, so multiple
nets can be named first and connected later without fighting Rust's mutable
borrow rules.

Custom typed parts use the same builder. The builder captures logical pins,
pad mappings, and electrical classes in one place, while the returned handle
keeps user code readable:

```rust
use via::prelude::*;

#[derive(Debug, Clone)]
pub struct Sensor {
    id: ModuleId,
}

impl Sensor {
    pub fn vcc(&self) -> PinRef {
        self.id.pin("VCC")
    }

    pub fn gnd(&self) -> PinRef {
        self.id.pin("GND")
    }

    pub fn scl(&self) -> PinRef {
        self.id.pin("SCL")
    }

    pub fn sda(&self) -> PinRef {
        self.id.pin("SDA")
    }
}

pub fn sensor(refdes: &str) -> impl Component<Output = Sensor> {
    part(refdes, "I2C sensor module")
        .footprint("Sensor_1x04")
        .pin(pin("VCC").power("3V3").pad("1"))
        .pin(pin("GND").ground().pad("2"))
        .pin(pin("SCL").logic("3V3").pad("3"))
        .pin(pin("SDA").logic("3V3").pad("4"))
        .handle(|id| Sensor { id })
}
```

Lower-level crates can still use `BoardSpec` directly when implementing parts,
patterns, or exporters:

```rust
use via_core::{Board, BoardSpec};
use via_kicad::write_netlist;
use via_parts_harmonic::{capacitor_0805, esp32_s3_n16r8, generated_footprint_pads};
use via_patterns_harmonic::DcBuckInputStageSpec;
use via_patterns_motion::{Tmc2209UartAxisPins, Tmc2209UartAxisSpec};

fn board() -> via_core::Result<Board> {
    let mut b = BoardSpec::new("polar_adjuster_v0");
    for footprint in generated_footprint_pads() {
        b.add_footprint_pads(footprint);
    }

    let esp32 = b.add(esp32_s3_n16r8("U1"))?;
    let x = b.add(
        Tmc2209UartAxisSpec::new("X")
            .driver("U2")
            .motor_connector("J2")
            .uart_resistor("R1")
            .pins(Tmc2209UartAxisPins::new(
                esp32.gpio4(),
                esp32.gpio5(),
                esp32.gpio6(),
                esp32.gpio7(),
                esp32.gpio15(),
            )),
    )?;
    b.add(
        DcBuckInputStageSpec::new()
            .input_loads([x.vmot()])
            .output_loads([esp32.power_5v()]),
    )?;
    let c3v3 = b.add(capacitor_0805("C1", "100nF 50V 3V3 local"))?;

    b.ground("GND").connect_all([esp32.ground(), x.ground()]);
    b.rail("3V3", "3V3")
        .connect_all([esp32.power_3v3(), x.vio()])
        .decouple(&c3v3);

    b.build()
}

fn main() -> via_core::Result<()> {
    let board = board()?;
    write_netlist(&board, "polar_adjuster_v0.net")?;
    Ok(())
}
```

`Board` is the checked, read-only result model. Normal user code should author
with `Design`, add reusable parts and patterns through `Design::add`, and call
`check()` or `build()` as the validation handoff. Lower-level part and pattern
crates may still use `BoardSpec` directly. `connect_all` accepts arrays,
vectors, or other `PinRef` iterators, and both `NetHandle::decouple(...)` and
`BoardSpec::rail(...).decouple(...)` wire a two-pin capacitor to a named rail
plus ground.

## Commands

The core product is the Rust library API. The CLI is a thin convenience wrapper
for export and CI checks.

Export the built-in polar-adjuster example:

```powershell
cargo run -p via-cli -- export --example polar-adjuster --out ..\..\..\electronics\generated\via\polar_adjuster_v0
```

Export an LCEDA Pro package for import experiments:

```powershell
cargo run -p via-cli -- export-lceda-pro --example polar-adjuster --out ..\..\..\electronics\generated\lceda_pro\polar_adjuster_v0.epro2
```

The LCEDA Pro backend writes a schematic-first `.epro2` zip matching Pro
exports: `project2.json`, `IMAGE/`, and one `.epru` record stream. It emits
`SYMBOL`, `DEVICE`, `BOARD`, `SCH`, and `SCH_PAGE` documents with refdes,
values, footprint names, logical pins, and net-labeled schematic wiring directly
from `via-core`.

This also writes generated KiCad footprints to:

```text
electronics/generated/via/polar_adjuster_v0/via_generated.pretty/
```

The schematic export does not write board placement, routing, DRC output,
Gerbers, or a fabrication-ready PCB. Use `export-pcb` with a VIA layout JSON for
a KiCad PCB draft, then use KiCad/JLCEDA as the final manufacturing gate.

Validate the built-in demo against the local KiCad footprint library:

```powershell
cargo run -p via-cli -- check --example polar-adjuster
```

Emit machine-readable diagnostics for CI:

```powershell
cargo run -p via-cli -- check --example polar-adjuster --json
```

Emit the VSCode/editor snapshot contract:

```powershell
cargo run -p via-cli -- snapshot --example polar-adjuster --out snapshot.json
```

The snapshot is V3 JSON generated through typed serde structs. It includes
footprint geometry, board rules, production diagnostics, a `source_hash`, and
per-module/net/footprint/rule signatures for layout update checks.

Run the stricter fabrication gate:

```powershell
cargo run -p via-cli -- check-production --example polar-adjuster
```

`check-production` includes the normal structural checks and additionally
fails parts that still require physical footprint verification or have no MPN /
supplier part number attached to the source model.

Verify the generated schematic with KiCad CLI on Windows:

```powershell
& "C:\Program Files\KiCad\10.0\bin\kicad-cli.exe" sch export pdf `
  ..\..\..\electronics\generated\via\polar_adjuster_v0\polar_adjuster_v0.kicad_sch `
  -o ..\..\..\electronics\generated\via\polar_adjuster_v0\polar_adjuster_v0_via_schematic_probe.pdf
```

Run checks:

```powershell
cargo test
cargo clippy --all-targets -- -D warnings
```

## Roadmap

See [ROADMAP.md](ROADMAP.md) for the long-term plan.

## Scope

Current MVP:

- Rust API for boards, modules, pins, pin-to-pad maps, nets, and validation.
- Workspace split between core model, KiCad backend, reusable parts, reusable
  board patterns, examples, and CLI.
- `via-footprint` crate for high-level generated KiCad footprints.
- `via-footprint-ir` crate for advanced custom footprint construction.
- Reusable typed part constructors as a separate `via-parts-harmonic` crate.
- Reusable project-level power/switch patterns as a separate
  `via-patterns-harmonic` crate.
- Reusable TMC2209 UART stepper-axis pattern as a separate
  `via-patterns-motion` crate.
- Typed electrical intent for power rails, ground, logic nets, and motor phases.
- KiCad footprint pad parsing for `.kicad_mod` files.
- Footprint pad validation for mapped logical pins.
- KiCad-style netlist export using physical footprint pad numbers.
- LCEDA Pro schematic package export with auditable component, pin, and net data.
- Physical pad conflict detection across nets.
- Production diagnostics for unverified footprints and missing sourcing data.
- JSON check summaries for CI integration.
- A real `polar_adjuster_v0` example based on this workspace.

Not in scope yet:

- Autorouting.
- Claiming a PCB is fabrication-ready.
- Replacing hand footprint measurement or KiCad DRC.
