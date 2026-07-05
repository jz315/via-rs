pub use via_core as core;
pub use via_footprint_ir as footprint_ir;

pub mod parts {
    pub use via_parts::*;
    pub use via_parts_harmonic::{
        Dc005BarrelJack, Esp32S3N16R8, Header2, Mp1584BuckAdapter, SilentStepStickTmc2209V20,
        TerminalBlock5, Xh2p54Motor4, dc005_barrel_jack, esp32_s3_n16r8, generated_footprint_pads,
        generated_footprints, mp1584_buck_adapter, pin_header_1x02, silentstepstick_tmc2209_v20,
        terminal_block_1x05, write_generated_footprints, xh2p54_motor4,
    };
}

pub mod patterns {
    pub use via_patterns_harmonic::*;
    pub use via_patterns_motion::*;
}

pub mod prelude {
    pub use crate::{parts, patterns};
    pub use via_core::{
        Board, BoardRules, CheckProfile, Component, Design, Diagnostic, DiagnosticSeverity,
        ElectricalClass, Error, Exporter, FootprintPads, ModuleId, Net, NetHandle,
        NetlessPartHandle, ObjectRef, Part, PartSpec, PartSpecBuilder, PinRef, PinSpec, Result,
        Unit, Voltage, part, pin,
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
