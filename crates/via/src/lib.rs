//! Rust-native circuit authoring facade for VIA.
//!
//! The prelude exposes the normal board-authoring API:
//!
//! ```rust
//! use via::prelude::*;
//!
//! fn board() -> Result<Board> {
//!     let mut design = Design::new("doc_example").rules(Rules::new());
//!     let signal = design.logic("SIGNAL", "3V3");
//!     let input = design.add(
//!         part("J1", "Input")
//!             .footprint(fp::testpad_1p0())
//!             .pin(pin("SIG").logic("3V3").pad("1")),
//!     )?;
//!     let output = design.add(
//!         part("J2", "Output")
//!             .footprint(fp::testpad_1p0())
//!             .pin(pin("SIG").logic("3V3").pad("1")),
//!     )?;
//!     design.connect(&signal, [input.pin("SIG"), output.pin("SIG")]);
//!     design.finish(ValidationProfile::Prototype)
//! }
//!
//! let checked = board()?;
//! assert_eq!(checked.name(), "doc_example");
//! # Ok::<(), via::core::Error>(())
//! ```

pub use via_core as core;
pub use via_footprint::fp;
pub use via_footprint_ir as footprint_ir;
pub use via_project as project;

pub mod design_ext;
pub mod units;

pub mod parts {
    pub use via_parts::*;
}

pub mod prelude {
    pub use crate::design_ext::{DesignExt, RailBuilder};
    pub use crate::units::{Capacitance, QuantityExt, RatedVoltage, Resistance};
    pub use crate::{fp, parts};
    pub use via_core::{
        Board, BoardRules, CheckProfile, Component, Design, Diagnostic, DiagnosticSeverity,
        ElectricalClass, Error, Exporter, Footprint, FootprintAsset, FootprintDefinition,
        FootprintPads, FootprintSource, ModuleId, NetHandle, ObjectRef, PartBuilder, PartId,
        PinRef, PinSpec, Result, SymbolKind, SymbolSide, SymbolSpec, Unit, ValidationProfile,
        ValidationReport, Voltage, part, pin, sym,
    };

    pub type Rules = BoardRules;
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn prelude_can_author_a_minimal_design() {
        let mut design = Design::new("prelude_minimal").rules(Rules::new());
        let signal = design.net("SIGNAL");
        let ground = design.ground("GND");

        let header = design
            .add(
                part("J1", "Header 1x02")
                    .footprint("Header_1x02")
                    .pin(pin("1").logic("3V3"))
                    .pin(pin("2").ground()),
            )
            .unwrap();
        let load = design
            .add(
                part("U1", "Load")
                    .footprint("Load_2Pin")
                    .pin(pin("IN").logic("3V3"))
                    .pin(pin("GND").ground()),
            )
            .unwrap();

        signal.connect_all(&mut design, [header.pin("1"), load.pin("IN")]);
        ground.connect_all(&mut design, [header.pin("2"), load.pin("GND")]);

        design.check(CheckProfile::Prototype).unwrap();
    }
}
