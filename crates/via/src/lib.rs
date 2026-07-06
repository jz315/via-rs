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
        ElectricalClass, Error, Exporter, Footprint, FootprintAsset, FootprintPads, ModuleId,
        NetHandle, ObjectRef, PinRef, PinSpec, Result, SymbolKind, SymbolSide, SymbolSpec, Unit,
        Voltage, part, pin, sym,
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
