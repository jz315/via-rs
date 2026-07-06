# via Roadmap

`via` should become a clean Rust-native electronics authoring library: typed
design intent in Rust, boring generated artifacts, and strong validation before
the design reaches KiCad or another manufacturing tool.

## Principles

- Keep the generic library project-neutral.
- Use normal Rust crates for parts, patterns, and product libraries.
- Prefer explicit APIs over macros until repetition proves the need.
- Keep symbol style, footprint geometry, and electrical connectivity separate.
- Generated files must be stable enough to review and diff.
- Exporters must clearly state what they can and cannot guarantee.

## Phase 1: Core Model

- Stable `Design` authoring API.
- Checked `Board` result model.
- Modules, logical pins, physical pads, nets, electrical classes, and rules.
- Diagnostics for duplicate references, missing pins, one-ended nets, missing
  footprints, and invalid pin-to-pad maps.

Completion bar:

- `cargo test --workspace` passes.
- A generic example can be checked and exported without project-specific crates.

## Phase 2: Footprint System

- Keep `via-footprint` focused on common generated footprints.
- Keep advanced pad geometry in `via-footprint-ir`.
- Support external KiCad footprint names as an explicit escape hatch.
- Provide clear metadata for generated and verify-required footprints.
- Make common footprints easy to attach from `fp::*` aliases.

Completion bar:

- Generated footprints round-trip through the KiCad parser.
- Downstream part crates can embed generated footprint geometry without manual
  registration.

## Phase 3: Parts And Patterns

- Keep `via-parts` generic: passives, simple connectors, generic semiconductors.
- Let downstream crates define measured modules and product-specific patterns.
- Make typed handles ergonomic so user code connects semantic pins, not strings.

Completion bar:

- A downstream crate can define a module, attach footprint geometry, return a
  typed handle, and export through the standard backends without modifying
  `via`.

## Phase 4: Exporters

- KiCad schematic, netlist, footprint library, and PCB draft export.
- LCEDA Pro schematic package export.
- JSON snapshot export for editor integrations.
- Stable ordering and low churn in generated files.

Completion bar:

- KiCad can open exported artifacts.
- Snapshot consumers get the same footprint geometry and pin maps as exporters.

## Phase 5: Checks

- Prototype checks for structural correctness.
- Production checks for unverified footprints, missing sourcing metadata, and
  risky electrical intent.
- Machine-readable diagnostics for CI.
- Optional policy controls for warning/error gates.

Completion bar:

- A CI job can fail on meaningful design mistakes before a human opens KiCad.

## Phase 6: Editor Workflow

- VSCode/editor snapshot view.
- Real footprint display from the same geometry used by exporters.
- Placement, routing draft, DRC, save/reopen, and KiCad PCB draft export.
- No fake UI actions: every visible control should change state or be removed.

Completion bar:

- A small board can go from Rust design snapshot to manual placement/routing
  draft and then into KiCad for final review.

## Later

- BOM and sourcing metadata.
- Design diff reports.
- Import comparison against existing KiCad artifacts.
- More generic footprints and package families.
- Optional macros once the builder API is stable.

## Non-Goals

- Replacing KiCad as the manufacturing sign-off tool.
- Hiding footprint measurement behind generic guesses.
- Shipping project-specific hardware definitions in the generic facade.
- Autorouting before the manual/editable workflow is reliable.
