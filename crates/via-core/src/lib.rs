pub mod design;
pub mod electrical;
pub mod error;
pub mod export;
pub mod footprint;
pub mod model;
pub mod rules;
pub mod spec;

pub use design::{CheckProfile, Design, NetHandle, NetlessPartHandle, Unit, Voltage};
pub use electrical::ElectricalClass;
pub use error::{Diagnostic, DiagnosticSeverity, Error, ObjectRef, Result};
pub use export::Exporter;
pub use footprint::FootprintPads;
pub use model::{Board, ModuleId, Net, Part, PinRef, PinSpec, pin};
pub use rules::BoardRules;
pub use spec::{BoardSpec, Component, DecouplerPins, PartSpec, PartSpecBuilder, PowerRail, part};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catches_unknown_pin() {
        let mut spec = BoardSpec::new("bad");
        let module = spec
            .add(Part::new("U1", "thing").footprint("Thing").pins(["A"]))
            .unwrap();

        spec.net("BROKEN")
            .connect_all([module.pin("A"), module.pin("B")]);

        let err = spec.build().unwrap_err();
        assert!(format!("{err}").contains("unknown pin B"));
        let Error::Validation(diagnostics) = err else {
            panic!("expected validation error");
        };
        assert_eq!(diagnostics[0].code(), Some("net.unknown_pin"));
        assert_eq!(diagnostics[0].object(), Some(&ObjectRef::pin("U1", "B")));
        assert_eq!(diagnostics[0].related(), &[ObjectRef::net("BROKEN")]);
    }

    #[test]
    fn validates_loaded_footprint_pads() {
        let mut spec = BoardSpec::new("footprint_check");
        let module = spec
            .add(
                Part::new("J1", "connector")
                    .footprint("Conn")
                    .pins(["A", "B"])
                    .pinmap([("A", "1"), ("B", "2")]),
            )
            .unwrap();
        spec.add_footprint_pads(FootprintPads::new("Conn", ["1", "2"]));
        spec.net("N")
            .connect_all([module.pin("A"), module.pin("B")]);

        spec.build().unwrap();
    }

    #[test]
    fn catches_unmodeled_footprint_pads() {
        let mut spec = BoardSpec::new("incomplete_symbol");
        let module = spec
            .add(
                Part::new("U1", "module")
                    .footprint("Module")
                    .pins(["A", "B"])
                    .pinmap([("A", "1"), ("B", "2")]),
            )
            .unwrap();
        spec.add_footprint_pads(FootprintPads::new("Module", ["1", "2", "3"]));
        spec.net("N")
            .connect_all([module.pin("A"), module.pin("B")]);

        let err = spec.build().unwrap_err();
        assert!(format!("{err}").contains("does not cover pads on footprint Module: 3"));
    }

    #[test]
    fn catches_physical_pad_on_multiple_nets() {
        let mut spec = BoardSpec::new("shorted_pad");
        let module = spec
            .add(
                Part::new("U1", "module")
                    .footprint("Module")
                    .pins(["A", "B", "C"])
                    .map_pin("A", "1")
                    .map_pin("B", "1")
                    .map_pin("C", "2"),
            )
            .unwrap();

        spec.net("NET_A")
            .connect_all([module.pin("A"), module.pin("C")]);
        spec.net("NET_B")
            .connect_all([module.pin("B"), module.pin("C")]);

        let err = spec.build().unwrap_err();
        assert!(format!("{err}").contains("physical pad U1.1 is connected to multiple nets"));
        let Error::Validation(diagnostics) = err else {
            panic!("expected validation error");
        };
        assert_eq!(
            diagnostics
                .iter()
                .find(|diagnostic| diagnostic.code() == Some("net.physical_pad_short"))
                .and_then(Diagnostic::object),
            Some(&ObjectRef::pad("U1", "1"))
        );
    }

    #[test]
    fn catches_electrical_class_mismatch() {
        let mut spec = BoardSpec::new("bad_power");
        let source = spec
            .add(
                Part::new("J1", "input")
                    .footprint("Input")
                    .pins(["12V", "GND"])
                    .power_pin("12V", "12V")
                    .ground_pin("GND"),
            )
            .unwrap();
        let logic = spec
            .add(
                Part::new("U1", "logic")
                    .footprint("Logic")
                    .pins(["3V3", "GND"])
                    .power_pin("3V3", "3V3")
                    .ground_pin("GND"),
            )
            .unwrap();

        spec.net("WRONG")
            .power("12V")
            .connect_all([source.pin("12V"), logic.pin("3V3")]);

        let err = spec.build().unwrap_err();
        assert!(format!("{err}").contains("net WRONG is power:12V"));
    }

    #[test]
    fn production_check_requires_source_and_verified_footprint() {
        let mut spec = BoardSpec::new("production_gate");
        let module = spec
            .add(
                Part::new("C1", "100uF")
                    .footprint("Cap")
                    .pins(["1", "2"])
                    .verify(),
            )
            .unwrap();
        spec.net("N")
            .connect_all([module.pin("1"), module.pin("2")]);
        let board = spec.build().unwrap();

        let err = board.check_production().unwrap_err();
        let text = format!("{err}");
        assert!(text.contains("C1 footprint Cap still requires physical verification"));
        assert!(text.contains("C1 has no production source"));
    }

    #[test]
    fn production_check_accepts_sourced_verified_part() {
        let mut spec = BoardSpec::new("production_ok");
        let module = spec
            .add(
                Part::new("R1", "1k")
                    .footprint("R")
                    .pins(["1", "2"])
                    .lcsc("C21190"),
            )
            .unwrap();
        spec.net("N")
            .connect_all([module.pin("1"), module.pin("2")]);
        let board = spec.build().unwrap();

        board.check_production().unwrap();
    }

    #[test]
    fn board_spec_build_returns_checked_board() {
        let mut spec = BoardSpec::new("spec_ok");
        let module = spec
            .add(Part::new("R1", "1k").footprint("R").pins(["1", "2"]))
            .unwrap();
        spec.net("N")
            .connect_all([module.pin("1"), module.pin("2")]);

        let board = spec.build().unwrap();

        assert_eq!(board.name(), "spec_ok");
        assert!(board.module("R1").is_some());
    }

    #[test]
    fn board_spec_has_classed_net_helpers() {
        let mut spec = BoardSpec::new("net_helpers");
        let pwr = spec
            .add(
                Part::new("J1", "input")
                    .footprint("Input")
                    .pins(["12V", "GND"])
                    .power_pin("12V", "12V")
                    .ground_pin("GND"),
            )
            .unwrap();
        let load = spec
            .add(
                Part::new("U1", "load")
                    .footprint("Load")
                    .pins(["VIN", "GND"])
                    .power_pin("VIN", "12V")
                    .ground_pin("GND"),
            )
            .unwrap();

        spec.power("12V_IN", "12V")
            .connect_all([pwr.pin("12V"), load.pin("VIN")]);
        spec.ground("GND")
            .connect_all([pwr.pin("GND"), load.pin("GND")]);

        spec.build().unwrap();
    }

    #[test]
    fn board_spec_rail_decouples_to_ground() {
        let mut spec = BoardSpec::new("rail_helpers");
        let source = spec
            .add(
                Part::new("J1", "input")
                    .footprint("Input")
                    .pins(["VIN", "GND"])
                    .power_pin("VIN", "5V")
                    .ground_pin("GND"),
            )
            .unwrap();
        let load = spec
            .add(
                Part::new("U1", "load")
                    .footprint("Load")
                    .pins(["VIN", "GND"])
                    .power_pin("VIN", "5V")
                    .ground_pin("GND"),
            )
            .unwrap();
        let cap = spec
            .add(
                Part::new("C1", "100nF")
                    .footprint("C")
                    .pins(["1", "2"])
                    .pin_class("1", ElectricalClass::Passive)
                    .pin_class("2", ElectricalClass::Passive),
            )
            .unwrap();

        spec.ground("GND")
            .connect_all([source.pin("GND"), load.pin("GND")]);
        spec.rail("5V", "5V")
            .connect_all([source.pin("VIN"), load.pin("VIN")])
            .decouple((cap.pin("1"), cap.pin("2")));

        let board = spec.build().unwrap();

        assert_eq!(
            board
                .nets()
                .find(|net| net.name() == "5V")
                .unwrap()
                .connections()
                .len(),
            3
        );
        assert_eq!(
            board
                .nets()
                .find(|net| net.name() == "GND")
                .unwrap()
                .connections()
                .len(),
            3
        );
    }

    #[test]
    fn connect_all_accepts_iterators() {
        let mut spec = BoardSpec::new("iter_connections");
        let a = spec
            .add(Part::new("J1", "a").footprint("J").pins(["1"]))
            .unwrap();
        let b = spec
            .add(Part::new("J2", "b").footprint("J").pins(["1"]))
            .unwrap();
        let c = spec
            .add(Part::new("J3", "c").footprint("J").pins(["1"]))
            .unwrap();

        spec.net("CHAIN")
            .connect_all(vec![a.pin("1"), b.pin("1")])
            .connect_all(std::iter::once(c.pin("1")));

        let board = spec.build().unwrap();

        assert_eq!(
            board
                .nets()
                .find(|net| net.name() == "CHAIN")
                .unwrap()
                .connections()
                .len(),
            3
        );
    }

    #[test]
    fn board_spec_build_rejects_invalid_board() {
        let mut spec = BoardSpec::new("spec_bad");
        let module = spec
            .add(Part::new("J1", "connector").footprint("J").pins(["1"]))
            .unwrap();
        spec.net("BROKEN")
            .connect_all([module.pin("1"), module.pin("2")]);

        let err = spec.build().unwrap_err();

        assert!(format!("{err}").contains("unknown pin 2"));
    }
}
