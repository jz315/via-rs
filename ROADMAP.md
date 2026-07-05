# via Long-Term Roadmap

`via` should grow into a Rust-native circuit authoring toolkit that exports
trustworthy KiCad artifacts. The long-term goal is not to hide electronics
engineering behind magic. The goal is to make design intent explicit, typed,
reviewable, reusable, and testable.

## North Star

Hardware projects should be able to keep their electrical intent in normal Rust
code:

- Modules and connectors are reusable typed objects.
- Pins, nets, power rails, and pinmaps are checked before KiCad opens.
- Generated KiCad files are boring, inspectable, and compatible with normal
  manual schematic/layout workflows.
- CI can catch swapped pins, missing footprints, one-ended nets, and dangerous
  power mistakes before a board is fabricated.

KiCad remains the manufacturing backend for schematic review, footprint
inspection, manual placement, routing, DRC, Gerbers, and fab outputs.

## Design Principles

- Rust is the authoring language. Do not invent a new DSL until there is a hard,
  repeated reason.
- Prefer explicit APIs over clever macros in early versions.
- Generated files must be readable and stable enough for review.
- Validation should catch practical board bring-up mistakes, not just syntax
  mistakes.
- Local, measured module definitions beat generic library guesses.
- Every exporter must state what it can and cannot guarantee.
- The tool should help KiCad users, not trap them outside KiCad.

## Phase 0: Seed Prototype

Status: started.

Goals:

- Create a small Rust crate and CLI named `via`.
- Model boards, modules, pins, nets, and basic validation.
- Export a KiCad-style netlist.
- Use `polar_adjuster_v0` as the first real example.
- Split reusable code into workspace crates so part libraries can be shared.

Current deliverables:

- `Board`, `Part`, `ModuleId`, `PinRef`, and `Net`.
- Validation for duplicate modules, unknown pins, unknown modules, missing
  footprints, and one-ended nets.
- Workspace crates:
  - `via-core` for the circuit model,
  - `via-footprint` for high-level footprint generators,
  - `via-footprint-ir` for advanced low-level footprint construction,
  - `via-kicad` for KiCad import/export,
  - `via-parts-harmonic` for reusable measured module constructors,
  - `via-cli` for check/export commands.
- Built-in `polar_adjuster_v0` demo.
- Generated report and netlist under `electronics/generated/via/`.

Completion bar:

- `cargo fmt` passes.
- `cargo test` passes.
- `cargo run -p via-cli -- export --example polar-adjuster` exports non-empty
  artifacts.

## Phase 1: Real KiCad Netlist Compatibility

Goal: make exported netlists useful to KiCad and easy to diff.

Work:

- Verify the current netlist format against KiCad import expectations.
- Add stable component ordering and net ordering.
- Add library/source metadata.
- Add useful properties such as `VIA_VERIFY`, module family, purchase source,
  measured footprint revision, and pinmap revision.
- Add golden-file tests for generated netlists.

Non-goals:

- Do not generate a fabrication-ready PCB yet.
- Do not attempt autorouting.

Completion bar:

- KiCad can consume or inspect the exported netlist without hand editing.
- Golden tests make accidental exporter churn obvious.

## Phase 2: Pinmap And Footprint Verification

Goal: solve the real pain: module pin order and footprint mismatch.

Work:

- Add a `PinMap` type that maps logical pins to footprint pads.
- Parse KiCad `.kicad_mod` files enough to read pad names/numbers.
- Check that every mapped pin exists as a pad.
- Check that every required pad is intentionally mapped, ignored, or marked
  mechanical.
- Support `VERIFY` parts where exact physical pinout must be measured.
- Produce a pinmap report that is readable before layout.

Example checks:

- ESP32 `GPIO4` maps to the correct dev-board header pad.
- DC-005 drawing pins `2`, `3`, `4` are intentionally assigned and marked
  meter-verify.
- TMC2209 carrier socket row spacing and pin names match the selected footprint.

Completion bar:

- The polar adjuster example can validate all local footprints copied from the
  current KiCad project.
- A wrong pad name fails with a clear diagnostic.

## Phase 3: Typed Electrical Intent

Goal: move from strings toward useful electrical types without becoming heavy.

Work:

- Introduce typed nets: `PowerNet`, `GroundNet`, `LogicNet`, `MotorPhaseNet`,
  `SwitchNet`, and `ReservedNet`.
- Add voltage/current annotations.
- Add basic rules:
  - no accidental 12V-to-3V3 short,
  - no motor phase connected to GPIO,
  - no one-ended required control signals,
  - no power input without a ground return,
  - no unverified high-risk connector.
- Add escape hatches for prototypes and intentional oddities.

Completion bar:

- The tool catches at least three realistic mistakes in modified
  `polar_adjuster_v0` examples.
- Diagnostics explain the board-level risk, not just the code location.

## Phase 4: Module Library

Goal: let projects reuse measured modules safely.

Work:

- Create shareable Rust part-library crates, starting with this workspace:
  - ESP32-S3 N16R8 dev board,
  - SilentStepStick TMC2209 v2.0,
  - DC-005 barrel jack,
  - MP1584 buck adapter,
  - XH2.54 4P motor connector,
  - KF301-style terminal blocks.
- Keep library definitions in normal Rust.
- Keep project connection logic out of reusable part crates.
- Add metadata fields:
  - source URL or drawing,
  - measured date,
  - verified by,
  - footprint revision,
  - known caveats.
- Support project-local libraries without publishing a crate.

Completion bar:

- `examples/polar_adjuster_v0.rs` uses reusable module constructors instead of
  hand-written pin arrays everywhere.

## Phase 5: KiCad Schematic Export

Goal: generate a real KiCad schematic that humans can open and review.

Work:

- Export `.kicad_pro` and `.kicad_sch`.
- Place symbols in a simple deterministic grid.
- Emit labels for nets.
- Preserve stable UUIDs where practical.
- Support project-local symbol and footprint libraries.
- Keep schematic output intentionally plain.

Non-goals:

- Beautiful schematic layout.
- Complex sheet hierarchy in the first pass.

Completion bar:

- KiCad opens the generated project.
- ERC can run.
- The generated schematic is good enough for review, even if not pretty.

## Phase 5A: Footprint Generation Layer

Goal: avoid hand-writing common `.kicad_mod` files while keeping physical
footprint generation separate from circuit intent.

Work:

- Add `via-footprint` as the normal-user generator crate.
- Add `via-footprint-ir` as the advanced low-level IR crate.
- Keep pad/text/line primitives out of the `via-footprint` public facade except
  through the explicit `into_ir()` escape hatch.
- Export deterministic KiCad `.kicad_mod` files.
- Emit production review layers: reference/value text, Fab, SilkS, and
  Courtyard.
- Start with boring generators:
  - 1xN and 2xN through-hole headers,
  - terminal blocks,
  - rectangular module sockets,
  - simple adapter boards.
- Keep measured/nonstandard modules explicit and reviewable.

Non-goals:

- Do not put footprint geometry in `via-core`.
- Do not make normal users hand-assemble pads, points, or graphic primitives.
- Do not claim generated footprints are correct without datasheet/drawing or
  measurement review.

Completion bar:

- A generated footprint can be parsed back by `via-kicad`.
- `via-parts-*` crates can choose between a generated footprint and an external
  KiCad footprint by name.
- KiCad CLI can load/export the generated footprint library for visual review.

## Phase 6: Board Skeleton Export

Goal: help start layout without pretending to finish layout.

Work:

- Export an initial `.kicad_pcb` with footprints placed from hints.
- Add placement hints:
  - absolute position,
  - side,
  - rotation,
  - keepout group,
  - connector edge intent.
- Add board outline support.
- Add basic design-rule metadata.

Non-goals:

- Autorouting.
- Automatic manufacturability claims.

Completion bar:

- A generated PCB skeleton opens in KiCad with all footprints present and
  ratsnest connections visible.

## Phase 7: CI And Review Workflow

Goal: make `via` useful in real repositories.

Work:

- Add `via check`.
- Add `via export`.
- Add `via diff` for generated artifact summaries.
- Add machine-readable JSON diagnostics.
- Add GitHub Actions examples.
- Add a policy for failing on warnings vs errors.

Completion bar:

- A project can run `via check` in CI and get useful failure messages for pinmap,
  net, footprint, and electrical-intent mistakes.

## Phase 8: Ecosystem And Open Source Polish

Goal: make the project useful outside this workspace.

Work:

- Write project positioning clearly:
  - not an autorouter,
  - not a KiCad replacement,
  - Rust-native design intent for KiCad users.
- Add contribution guide.
- Add examples:
  - motor controller,
  - sensor breakout,
  - USB-powered microcontroller board,
  - module-carrier board.
- Add versioned export guarantees.
- Add fixture-based KiCad compatibility tests.
- Publish crate when the API is stable enough.

Completion bar:

- A new user can clone the repo, run one example, open output in KiCad, and
  understand what `via` guarantees.

## Later Ideas

- Generate HTML design review reports.
- Import selected KiCad schematic/netlist data for comparison.
- Support BOM export.
- Support JLCPCB/LCSC metadata as optional project-local data.
- Support SPICE or ERC hooks for specific circuit classes.
- Add optional macros once repeated boilerplate becomes painful.
- Add a package registry only if normal Rust crates are not enough.

## Things To Avoid

- Do not invent a new language early.
- Do not hide unsafe assumptions behind pretty output.
- Do not claim fabrication readiness without KiCad DRC and human footprint
  review.
- Do not chase autorouting before pinmaps, footprints, and schematic export are
  boringly reliable.
- Do not let generated files churn needlessly; stable diffs matter.
