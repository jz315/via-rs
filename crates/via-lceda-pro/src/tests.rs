use via_core::{BoardSpec, FootprintPads, Part};
use via_footprint_ir::{FootprintIr, GraphicLine, GraphicText, Pad, PadShape, Point, Size};

use crate::export::{render_epru, write_lceda_pro_project};
use crate::ids::footprint_uuid;

#[test]
fn writes_epro2_zip_with_schematic_record_stream() {
    let mut spec = BoardSpec::new("demo");
    spec.add_footprint_pads(two_pad_ir_footprint("R_0603"));
    let r1 = spec
        .add(
            Part::new("R1", "1k")
                .footprint("R_0603")
                .pins(["1", "2"])
                .logic_pin("1", "3V3")
                .logic_pin("2", "3V3"),
        )
        .unwrap();
    spec.net("SIG")
        .logic("3V3")
        .connect_all([r1.pin("1"), r1.pin("2")]);
    let board = spec.build().unwrap();

    let path = std::env::temp_dir().join("via_lceda_pro_demo.epro2");
    write_lceda_pro_project(&board, &path).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let _ = std::fs::remove_file(path);

    assert!(bytes.starts_with(b"PK\x03\x04"));
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("project2.json"));
    assert!(text.contains("IMAGE/"));
    assert!(text.contains("demo.epru"));
    assert!(text.contains("\"docType\":\"FOOTPRINT\""));
    assert!(text.contains("\"docType\":\"SYMBOL\""));
    assert!(text.contains("\"docType\":\"SCH\""));
    assert!(text.contains("\"docType\":\"SCH_PAGE\""));
    assert!(text.contains("\"docType\":\"PCB\""));
    assert!(text.contains("\"type\":\"PAD\""));
    assert!(text.contains(&format!("\"Footprint\":\"{}\"", footprint_uuid("R_0603"))));
    assert!(text.contains("\"key\":\"Footprint\""));
    assert!(text.contains("\"parentId\":\"c_R1\""));
    assert!(text.contains("\"WIRE\""));
    assert!(text.contains("SIG"));
    assert!(text.contains("\"PAD_NET\""));
}

#[test]
fn write_rejects_placeholder_footprint_geometry() {
    let mut spec = BoardSpec::new("placeholder");
    spec.add_footprint_pads(FootprintPads::new("Header_1x02", ["1", "2"]));
    let header = spec
        .add(
            Part::new("J1", "Header")
                .footprint("Header_1x02")
                .pins(["1", "2"]),
        )
        .unwrap();
    spec.net("N")
        .connect_all([header.pin("1"), header.pin("2")]);
    let board = spec.build().unwrap();

    let path = std::env::temp_dir().join("via_lceda_pro_placeholder.epro2");
    let error = write_lceda_pro_project(&board, &path).unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(error.to_string().contains("no geometry IR"));
    assert!(error.to_string().contains("55x55 placeholder"));
    let _ = std::fs::remove_file(path);
}

#[test]
fn schematic_page_contains_component_device_and_pin_wires() {
    let mut spec = BoardSpec::new("chain");
    let m = spec
        .add(
            Part::new("J1", "Header")
                .footprint("Header_1x02")
                .pins(["1", "2"]),
        )
        .unwrap();
    spec.net("N").connect_all([m.pin("1"), m.pin("2")]);
    let board = spec.build().unwrap();

    let epru = render_epru(&board);
    assert!(epru.contains("\"partId\":\"via_J1_Header.1\""));
    assert!(epru.contains("\"key\":\"Device\""));
    assert!(epru.contains("\"id\":\"c_J1_footprint\""));
    assert!(epru.contains(&format!("\"value\":\"{}\"", footprint_uuid("Header_1x02"))));
    assert!(epru.contains("\"type\":\"WIRE\""));
    assert!(epru.contains("\"type\":\"LINE\""));
    assert!(epru.contains("\"lineGroup\":\"w0_0\""));
    assert!(epru.contains("\"key\":\"NET\""));
    assert!(epru.contains("\"value\":\"N\""));
}

#[test]
fn pcb_document_contains_component_instances_and_pad_net_mappings() {
    let mut spec = BoardSpec::new("pcb_draft");
    spec.add_footprint_pads(FootprintPads::new("Header_1x02", ["1", "2"]));
    let header = spec
        .add(
            Part::new("J1", "Header")
                .footprint("Header_1x02")
                .pins(["1", "2"]),
        )
        .unwrap();
    spec.net("N")
        .connect_all([header.pin("1"), header.pin("2")]);
    let board = spec.build().unwrap();

    let epru = render_epru(&board);
    assert!(epru.contains("\"docType\":\"PCB\""));
    assert!(epru.contains("\"id\":\"pcb_J1\""));
    assert!(epru.contains("\"parentId\":\"pcb_J1\""));
    assert!(epru.contains(&format!("\"value\":\"{}\"", footprint_uuid("Header_1x02"))));
    assert!(epru.contains("\"type\":\"PAD_NET\""));
    assert!(epru.contains("\"padNet\":\"N\""));
    assert!(epru.contains("[\\\"NET\\\",\\\"N\\\"]"));
}

#[test]
fn footprint_ir_geometry_is_rendered_instead_of_placeholder_pads() {
    let mut footprint = FootprintIr::new("Real_1x03");
    footprint
        .add_pad(Pad::thru_hole(
            "1",
            PadShape::Rect,
            Point::new(0.0, 0.0),
            Size::new(1.8, 1.8),
            1.0,
        ))
        .add_pad(Pad::thru_hole(
            "2",
            PadShape::Circle,
            Point::new(2.54, 0.0),
            Size::new(1.8, 1.8),
            1.0,
        ))
        .add_pad(Pad::thru_hole_slot(
            "3",
            PadShape::Oval,
            Point::new(5.08, 0.0),
            Size::new(2.2, 4.4),
            1.0,
            3.0,
        ))
        .add_line(GraphicLine::new(
            Point::new(-1.0, -1.0),
            Point::new(3.54, -1.0),
            "F.SilkS",
            0.12,
        ))
        .add_text(GraphicText::reference(
            "REF**",
            Point::new(0.0, -2.0),
            "F.SilkS",
        ))
        .add_text(GraphicText::value(
            "Real_1x03",
            Point::new(0.0, 2.0),
            "F.Fab",
        ))
        .add_text(GraphicText::user(
            "pin note",
            Point::new(0.0, 0.0),
            "F.SilkS",
        ));
    let mut spec = BoardSpec::new("real_footprint");
    spec.add_footprint_pads(FootprintPads::from_ir(footprint));
    let header = spec
        .add(
            Part::new("J1", "Header")
                .footprint("Real_1x03")
                .pins(["1", "2", "3"]),
        )
        .unwrap();
    spec.net("N")
        .connect_all([header.pin("1"), header.pin("2")]);
    let board = spec.build().unwrap();

    let epru = render_epru(&board);
    assert!(epru.contains("\"docType\":\"FOOTPRINT\""));
    assert!(epru.contains("\"centerX\":100"));
    assert!(epru.contains(
        "\"defaultPad\":{\"padType\":\"RECT\",\"width\":70.8661,\"height\":70.8661,\"radius\":0}"
    ));
    assert!(
        epru.contains("\"hole\":{\"holeType\":\"ROUND\",\"width\":39.3701,\"height\":39.3701}")
    );
    assert!(
        epru.contains("\"hole\":{\"holeType\":\"SLOT\",\"width\":39.3701,\"height\":118.1102}")
    );
    assert!(
        epru.contains(
            "\"defaultPad\":{\"padType\":\"OVAL\",\"width\":86.6142,\"height\":173.2283}"
        )
    );
    assert!(epru.contains("\"type\":\"LINE\""));
    assert!(epru.contains("\"text\":\"pin note\""));
    assert!(epru.contains("\"fontFamily\":\"default\""));
    assert!(!epru.contains("\"text\":\"REF**\""));
    assert!(!epru.contains("\"text\":\"Real_1x03\""));
    assert!(
        !epru.contains(
            "\"defaultPad\":{\"padType\":\"RECT\",\"width\":55,\"height\":55,\"radius\":0}"
        )
    );
}

#[test]
fn symbol_pin_numbers_use_physical_pad_numbers() {
    let mut spec = BoardSpec::new("mapped");
    spec.add_footprint_pads(FootprintPads::new("Conn_1x04", ["1", "2", "3", "4"]));
    let connector = spec
        .add(
            Part::new("J1", "Motor")
                .footprint("Conn_1x04")
                .pins(["A2", "A1", "B1", "B2"])
                .pinmap([("A2", "1"), ("A1", "2"), ("B1", "3"), ("B2", "4")]),
        )
        .unwrap();
    spec.net("PHASE_A")
        .connect_all([connector.pin("A1"), connector.pin("A2")]);
    let board = spec.build().unwrap();

    let epru = render_epru(&board);
    assert!(epru.contains("\"key\":\"Pin Name\",\"value\":\"A1\""));
    assert!(epru.contains("\"key\":\"Pin Name\",\"value\":\"A2\""));
    assert!(epru.contains("\"key\":\"Pin Number\",\"value\":\"2\""));
    assert!(epru.contains("\"key\":\"Pin Number\",\"value\":\"1\""));
    assert!(!epru.contains("\"key\":\"Pin Number\",\"value\":\"A1\""));
    assert!(!epru.contains("\"key\":\"Pin Number\",\"value\":\"A2\""));
}

#[test]
fn multi_pad_logical_pin_expands_to_all_pad_numbers_and_net_wires() {
    let mut spec = BoardSpec::new("multi_pad");
    spec.add_footprint_pads(FootprintPads::new("Module", ["1", "2"]));
    spec.add_footprint_pads(FootprintPads::new("Header_1x01", ["1"]));
    let module = spec
        .add(
            Part::new("U1", "Module")
                .footprint("Module")
                .pins(["GND"])
                .map_pin_to_pads("GND", ["1", "2"]),
        )
        .unwrap();
    let header = spec
        .add(
            Part::new("J1", "Header")
                .footprint("Header_1x01")
                .pins(["1"]),
        )
        .unwrap();
    spec.net("GND")
        .connect_all([module.pin("GND"), header.pin("1")]);
    let board = spec.build().unwrap();

    let epru = render_epru(&board);
    assert!(epru.contains("\"key\":\"Pin Name\",\"value\":\"GND\""));
    assert!(epru.contains("\"key\":\"Pin Number\",\"value\":\"1\""));
    assert!(epru.contains("\"key\":\"Pin Number\",\"value\":\"2\""));
    assert!(!epru.contains("\"key\":\"Pin Number\",\"value\":\"GND\""));
    assert!(epru.contains("\"lineGroup\":\"w0_2\""));
}

fn two_pad_ir_footprint(name: &str) -> FootprintPads {
    let mut footprint = FootprintIr::new(name);
    footprint
        .add_pad(Pad::smd(
            "1",
            PadShape::Rect,
            Point::new(-0.8, 0.0),
            Size::new(0.8, 1.0),
        ))
        .add_pad(Pad::smd(
            "2",
            PadShape::Rect,
            Point::new(0.8, 0.0),
            Size::new(0.8, 1.0),
        ));
    FootprintPads::from_ir(footprint)
}
