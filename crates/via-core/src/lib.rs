pub mod design;
pub mod electrical;
pub mod error;
pub mod export;
pub mod footprint;
pub mod ir;
pub mod model;
pub mod rules;
mod spec;
pub mod symbol;

pub use design::{CheckProfile, Design, NetHandle, Unit, Voltage};
pub use electrical::ElectricalClass;
pub use error::{Diagnostic, DiagnosticSeverity, Error, ObjectRef, Result};
pub use export::Exporter;
pub use footprint::{Footprint, FootprintAsset, FootprintPads};
pub use ir::{BOARD_IR_SCHEMA, BOARD_IR_VERSION, BoardIr};
pub use model::{Board, ModuleId, PinRef, PinSpec, pin};
pub use rules::BoardRules;
pub use spec::{Component, DecouplerPins, PartSpec, PartSpecBuilder, part};
pub use symbol::{SymbolKind, SymbolSide, SymbolSpec, sym};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Part;

    #[test]
    fn catches_unknown_pin() {
        let mut design = Design::new("bad");
        let module = design
            .add(Part::new("U1", "thing").footprint("Thing").pins(["A"]))
            .unwrap();

        design
            .net("BROKEN")
            .connect_all(&mut design, [module.pin("A"), module.pin("B")]);

        let err = design.build().unwrap_err();
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
        let mut design = Design::new("footprint_check");
        let module = design
            .add(
                Part::new("J1", "connector")
                    .footprint("Conn")
                    .pins(["A", "B"])
                    .pinmap([("A", "1"), ("B", "2")]),
            )
            .unwrap();
        design.add_footprint_pads(FootprintPads::new("Conn", ["1", "2"]));
        design
            .net("N")
            .connect_all(&mut design, [module.pin("A"), module.pin("B")]);

        design.build().unwrap();
    }

    #[test]
    fn catches_unknown_footprint_when_catalog_is_loaded() {
        let mut design = Design::new("missing_footprint");
        design.add_footprint_pads(FootprintPads::new("Known", ["1", "2"]));
        let module = design
            .add(
                Part::new("J1", "connector")
                    .footprint("Missing")
                    .pins(["1", "2"]),
            )
            .unwrap();
        design
            .net("N")
            .connect_all(&mut design, [module.pin("1"), module.pin("2")]);

        let err = design.build().unwrap_err();
        assert!(format!("{err}").contains("references footprint Missing"));
    }

    #[test]
    fn catches_kicad_asset_aliases() {
        let mut design = Design::new("aliased_footprint");
        design.add_footprint_pads(
            FootprintPads::new("Local", ["1", "2"]).with_kicad_library("Fixture_Lib", "Remote"),
        );
        let module = design
            .add(
                Part::new("J1", "connector")
                    .footprint("Local")
                    .pins(["1", "2"]),
            )
            .unwrap();
        design
            .net("N")
            .connect_all(&mut design, [module.pin("1"), module.pin("2")]);

        let err = design.build().unwrap_err();
        assert!(format!("{err}").contains("aliasing is not supported"));
    }

    #[test]
    fn catches_unmodeled_footprint_pads() {
        let mut design = Design::new("incomplete_symbol");
        let module = design
            .add(
                Part::new("U1", "module")
                    .footprint("Module")
                    .pins(["A", "B"])
                    .pinmap([("A", "1"), ("B", "2")]),
            )
            .unwrap();
        design.add_footprint_pads(FootprintPads::new("Module", ["1", "2", "3"]));
        design
            .net("N")
            .connect_all(&mut design, [module.pin("A"), module.pin("B")]);

        let err = design.build().unwrap_err();
        assert!(format!("{err}").contains("does not cover pads on footprint Module: 3"));
    }

    #[test]
    fn catches_unknown_symbol_pin() {
        let mut design = Design::new("bad_symbol");
        let module = design
            .add(
                Part::new("U1", "module")
                    .footprint("Module")
                    .symbol(sym::module().left(["IN"]).right(["MISSING"]))
                    .pins(["IN", "OUT"]),
            )
            .unwrap();
        design
            .net("N")
            .connect_all(&mut design, [module.pin("IN"), module.pin("OUT")]);

        let err = design.build().unwrap_err();
        assert!(format!("{err}").contains("symbol references unknown logical pin MISSING"));
        let Error::Validation(diagnostics) = err else {
            panic!("expected validation error");
        };
        assert_eq!(
            diagnostics
                .iter()
                .find(|diagnostic| diagnostic.code() == Some("symbol.unknown_pin"))
                .and_then(Diagnostic::object),
            Some(&ObjectRef::pin("U1", "MISSING"))
        );
    }

    #[test]
    fn catches_physical_pad_on_multiple_nets() {
        let mut design = Design::new("shorted_pad");
        let module = design
            .add(
                Part::new("U1", "module")
                    .footprint("Module")
                    .pins(["A", "B", "C"])
                    .map_pin("A", "1")
                    .map_pin("B", "1")
                    .map_pin("C", "2"),
            )
            .unwrap();

        design
            .net("NET_A")
            .connect_all(&mut design, [module.pin("A"), module.pin("C")]);
        design
            .net("NET_B")
            .connect_all(&mut design, [module.pin("B"), module.pin("C")]);

        let err = design.build().unwrap_err();
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
        let mut design = Design::new("bad_power");
        let source = design
            .add(
                Part::new("J1", "input")
                    .footprint("Input")
                    .pins(["12V", "GND"])
                    .power_pin("12V", "12V")
                    .ground_pin("GND"),
            )
            .unwrap();
        let logic = design
            .add(
                Part::new("U1", "logic")
                    .footprint("Logic")
                    .pins(["3V3", "GND"])
                    .power_pin("3V3", "3V3")
                    .ground_pin("GND"),
            )
            .unwrap();

        design
            .power_domain("WRONG", "12V")
            .connect_all(&mut design, [source.pin("12V"), logic.pin("3V3")]);

        let err = design.build().unwrap_err();
        assert!(format!("{err}").contains("net WRONG is power:12V"));
    }

    #[test]
    fn production_check_requires_source_and_verified_footprint() {
        let mut design = Design::new("production_gate");
        let module = design
            .add(
                Part::new("C1", "100uF")
                    .footprint("Cap")
                    .pins(["1", "2"])
                    .verify(),
            )
            .unwrap();
        design
            .net("N")
            .connect_all(&mut design, [module.pin("1"), module.pin("2")]);
        let board = design.build().unwrap();

        let err = board.check_production().unwrap_err();
        let text = format!("{err}");
        assert!(text.contains("C1 footprint Cap still requires physical verification"));
        assert!(text.contains("C1 has no production source"));
    }

    #[test]
    fn production_check_accepts_sourced_verified_part() {
        let mut design = Design::new("production_ok");
        let module = design
            .add(
                Part::new("R1", "1k")
                    .footprint("R")
                    .pins(["1", "2"])
                    .lcsc("C21190"),
            )
            .unwrap();
        design
            .net("N")
            .connect_all(&mut design, [module.pin("1"), module.pin("2")]);
        let board = design.build().unwrap();

        board.check_production().unwrap();
    }

    #[test]
    fn design_build_returns_checked_board() {
        let mut design = Design::new("design_ok");
        let module = design
            .add(Part::new("R1", "1k").footprint("R").pins(["1", "2"]))
            .unwrap();
        design
            .net("N")
            .connect_all(&mut design, [module.pin("1"), module.pin("2")]);

        let board = design.build().unwrap();

        assert_eq!(board.name(), "design_ok");
        assert!(board.module("R1").is_some());
    }

    #[test]
    fn design_has_classed_net_helpers() {
        let mut design = Design::new("net_helpers");
        let pwr = design
            .add(
                Part::new("J1", "input")
                    .footprint("Input")
                    .pins(["12V", "GND"])
                    .power_pin("12V", "12V")
                    .ground_pin("GND"),
            )
            .unwrap();
        let load = design
            .add(
                Part::new("U1", "load")
                    .footprint("Load")
                    .pins(["VIN", "GND"])
                    .power_pin("VIN", "12V")
                    .ground_pin("GND"),
            )
            .unwrap();

        design
            .power_domain("12V_IN", "12V")
            .connect_all(&mut design, [pwr.pin("12V"), load.pin("VIN")]);
        design
            .ground("GND")
            .connect_all(&mut design, [pwr.pin("GND"), load.pin("GND")]);

        design.build().unwrap();
    }

    #[test]
    fn design_power_net_decouples_to_ground() {
        let mut design = Design::new("rail_helpers");
        let source = design
            .add(
                Part::new("J1", "input")
                    .footprint("Input")
                    .pins(["VIN", "GND"])
                    .power_pin("VIN", "5V")
                    .ground_pin("GND"),
            )
            .unwrap();
        let load = design
            .add(
                Part::new("U1", "load")
                    .footprint("Load")
                    .pins(["VIN", "GND"])
                    .power_pin("VIN", "5V")
                    .ground_pin("GND"),
            )
            .unwrap();
        let cap = design
            .add(
                Part::new("C1", "100nF")
                    .footprint("C")
                    .pins(["1", "2"])
                    .pin_class("1", ElectricalClass::Passive)
                    .pin_class("2", ElectricalClass::Passive),
            )
            .unwrap();

        design
            .ground("GND")
            .connect_all(&mut design, [source.pin("GND"), load.pin("GND")]);
        design
            .power_domain("5V", "5V")
            .connect_all(&mut design, [source.pin("VIN"), load.pin("VIN")])
            .decouple(&mut design, (cap.pin("1"), cap.pin("2")));

        let board = design.build().unwrap();

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
        let mut design = Design::new("iter_connections");
        let a = design
            .add(Part::new("J1", "a").footprint("J").pins(["1"]))
            .unwrap();
        let b = design
            .add(Part::new("J2", "b").footprint("J").pins(["1"]))
            .unwrap();
        let c = design
            .add(Part::new("J3", "c").footprint("J").pins(["1"]))
            .unwrap();

        design
            .net("CHAIN")
            .connect_all(&mut design, vec![a.pin("1"), b.pin("1")])
            .connect_all(&mut design, std::iter::once(c.pin("1")));

        let board = design.build().unwrap();

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
    fn board_ir_roundtrips_board_state() {
        let mut footprint = via_footprint_ir::FootprintIr::new("Conn_1x02");
        footprint.add_pad(via_footprint_ir::Pad::thru_hole(
            "1",
            via_footprint_ir::PadShape::Rect,
            via_footprint_ir::Point::new(0.0, 0.0),
            via_footprint_ir::Size::new(1.8, 1.8),
            1.0,
        ));
        footprint.add_pad(via_footprint_ir::Pad::thru_hole(
            "2",
            via_footprint_ir::PadShape::Circle,
            via_footprint_ir::Point::new(2.54, 0.0),
            via_footprint_ir::Size::new(1.8, 1.8),
            1.0,
        ));

        let mut design = Design::new("ir_roundtrip");
        design
            .rules_mut()
            .set_net_class_track_widths_mm([("logic:3V3".to_owned(), 0.25)]);
        design.add_footprint_pads(FootprintPads::from_ir(footprint));

        let module = design
            .add(
                part("J1", "Debug connector")
                    .footprint("Conn_1x02")
                    .symbol(sym::connector().left(["1"]).right(["2"]))
                    .pin(pin("1").logic("3V3").pad("1"))
                    .pin(pin("2").ground().pad("2"))
                    .lcsc("C123"),
            )
            .unwrap();
        design
            .net("SIG")
            .connect_all(&mut design, [module.pin("1"), module.pin("2")]);

        let board = design.build().unwrap();
        let roundtripped = Board::from_ir(board.to_ir()).unwrap();

        assert_eq!(roundtripped, board);
        assert_eq!(roundtripped.footprints().count(), 1);
        assert!(roundtripped.footprints().next().unwrap().ir().is_some());
    }

    #[test]
    fn board_ir_roundtrips_footprint_assets_and_accepts_v1() {
        let mut design = Design::new("asset_ir");
        design.add_footprint_pads(FootprintPads::kicad_library(
            "Fixture_Lib",
            "Fixture_Footprint",
            ["1", "2"],
        ));
        let module = design
            .add(
                part("J1", "fixture")
                    .footprint("Fixture_Footprint")
                    .pin(pin("1").passive())
                    .pin(pin("2").passive()),
            )
            .unwrap();
        design
            .net("N")
            .connect_all(&mut design, [module.pin("1"), module.pin("2")]);

        let board = design.build().unwrap();
        let ir = board.to_ir();
        assert_eq!(ir.version, BOARD_IR_VERSION);
        let roundtripped = Board::from_ir(ir.clone()).unwrap();
        let asset = roundtripped.footprints().next().unwrap().asset().unwrap();
        assert_eq!(
            asset,
            &FootprintAsset::KicadLibrary {
                library: "Fixture_Lib".to_owned(),
                name: "Fixture_Footprint".to_owned(),
            }
        );

        let mut v1_ir = ir;
        v1_ir.version = 1;
        for footprint in &mut v1_ir.board.footprints {
            footprint.asset = None;
        }
        let v1_board = Board::from_ir(v1_ir).unwrap();
        assert!(v1_board.footprints().next().unwrap().asset().is_none());
    }

    #[test]
    fn design_build_rejects_invalid_board() {
        let mut design = Design::new("design_bad");
        let module = design
            .add(Part::new("J1", "connector").footprint("J").pins(["1"]))
            .unwrap();
        design
            .net("BROKEN")
            .connect_all(&mut design, [module.pin("1"), module.pin("2")]);

        let err = design.build().unwrap_err();

        assert!(format!("{err}").contains("unknown pin 2"));
    }
}
