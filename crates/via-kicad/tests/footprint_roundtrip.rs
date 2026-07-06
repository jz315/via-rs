use via_footprint::generators::{soic8, terminal_block_1x, tht_header_1x, xh_vertical_1x};
use via_kicad::footprint_pads_from_kicad_mod;

#[test]
fn parses_generated_footprint_pads() {
    let footprint = tht_header_1x("PinHeader_1x04_P2.54_Via", 4).build();
    let out = std::env::temp_dir().join("PinHeader_1x04_P2.54_Via.kicad_mod");

    footprint.write_kicad_mod(&out).unwrap();
    let pads = footprint_pads_from_kicad_mod(&out).unwrap();
    let _ = std::fs::remove_file(out);

    assert_eq!(pads.name(), "PinHeader_1x04_P2.54_Via");
    assert!(pads.contains_pad("1"));
    assert!(pads.contains_pad("2"));
    assert!(pads.contains_pad("3"));
    assert!(pads.contains_pad("4"));
}

#[test]
fn parses_generic_generated_footprints() {
    let generated = [
        (terminal_block_1x("TB_1x05_P5.08", 5).build(), 5),
        (xh_vertical_1x("XH_1x04_P2.54_VERIFY", 4).build(), 4),
        (soic8(), 8),
    ];

    for (footprint, expected_pads) in generated {
        let out = std::env::temp_dir().join(format!("{}.kicad_mod", footprint.name()));
        footprint.write_kicad_mod(&out).unwrap();
        let pads = footprint_pads_from_kicad_mod(&out).unwrap();
        let _ = std::fs::remove_file(out);

        assert_eq!(pads.name(), footprint.name());
        assert_eq!(pads.pads().len(), expected_pads, "{}", footprint.name());
        assert!(pads.contains_pad("1"));
        assert!(pads.contains_pad(&expected_pads.to_string()));
    }
}
