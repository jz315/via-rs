use std::path::Path;

use via_core::FootprintPads;
use via_footprint::{FootprintWriteError, GeneratedFootprint};

pub fn generated_footprints() -> Vec<GeneratedFootprint> {
    use via_footprint::generators::{
        capacitor_0603, capacitor_0805, dc005_5p5x2p1_right_angle_drawing_verify,
        esp32_s3_n16r8_devboard_socket, mp1584_4wire_adapter,
        polarized_capacitor_radial_d6p3_p2p50_verify, resistor_0603, resistor_0805,
        silentstepstick_tmc2209_v20_socket, terminal_block_1x, tht_header_1x, xh_vertical_1x,
    };

    vec![
        esp32_s3_n16r8_devboard_socket(),
        silentstepstick_tmc2209_v20_socket(),
        mp1584_4wire_adapter(),
        xh_vertical_1x("XH2p54_1x04_Vertical_THT_VERIFY", 4)
            .pin_labels(
                ["A2", "A1", "B1", "B2"]
                    .into_iter()
                    .map(str::to_owned)
                    .collect::<Vec<_>>(),
            )
            .build(),
        terminal_block_1x("TerminalBlock_1x02_P5.08", 2).build(),
        dc005_5p5x2p1_right_angle_drawing_verify(
            "DC005_5p5x2p1_RightAngle_THT_Drawing_2_3_4_VERIFY",
        ),
        terminal_block_1x("TerminalBlock_1x05_P5.08", 5).build(),
        tht_header_1x("PinHeader_1x02_P2.54", 2).build(),
        tht_header_1x("PinHeader_1x08_P2.54", 8).build(),
        resistor_0603("R_0603_1608Metric"),
        resistor_0805("R_0805_2012Metric"),
        capacitor_0603("C_0603_1608Metric"),
        capacitor_0805("C_0805_2012Metric"),
        polarized_capacitor_radial_d6p3_p2p50_verify("CP_Radial_D6p3_P2p50_VERIFY"),
    ]
}

pub fn write_generated_footprints(
    pretty_dir: impl AsRef<Path>,
) -> std::result::Result<usize, FootprintWriteError> {
    let pretty_dir = pretty_dir.as_ref();
    std::fs::create_dir_all(pretty_dir)?;
    for entry in std::fs::read_dir(pretty_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("kicad_mod") {
            std::fs::remove_file(path)?;
        }
    }

    let mut count = 0;
    for footprint in generated_footprints() {
        footprint.write_kicad_mod(pretty_dir.join(format!("{}.kicad_mod", footprint.name())))?;
        count += 1;
    }

    Ok(count)
}

pub fn generated_footprint_pads() -> Vec<FootprintPads> {
    generated_footprints()
        .into_iter()
        .map(|footprint| {
            let ir = footprint.into_ir();
            FootprintPads::from_ir(ir)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_footprints_cover_current_generated_names() {
        let names = generated_footprints()
            .into_iter()
            .map(|footprint| footprint.name().to_owned())
            .collect::<std::collections::BTreeSet<_>>();

        for expected in [
            "ESP32-S3-N16R8_DevBoard_2x22_P2.54_Row25.40",
            "SilentStepStick_TMC2209_v20_CarrierSocket_2x8_Row12p70",
            "BuckModule_4Wire_MP1584_Adapter",
            "XH2p54_1x04_Vertical_THT_VERIFY",
            "TerminalBlock_1x02_P5.08",
            "DC005_5p5x2p1_RightAngle_THT_Drawing_2_3_4_VERIFY",
            "TerminalBlock_1x05_P5.08",
            "PinHeader_1x02_P2.54",
            "PinHeader_1x08_P2.54",
            "R_0603_1608Metric",
            "R_0805_2012Metric",
            "C_0603_1608Metric",
            "C_0805_2012Metric",
            "CP_Radial_D6p3_P2p50_VERIFY",
        ] {
            assert!(names.contains(expected), "{expected}");
        }
    }

    #[test]
    fn generated_footprint_pad_metadata_covers_passives() {
        for name in [
            "R_0603_1608Metric",
            "R_0805_2012Metric",
            "C_0603_1608Metric",
            "C_0805_2012Metric",
            "CP_Radial_D6p3_P2p50_VERIFY",
        ] {
            let pads = generated_footprint_pads()
                .into_iter()
                .find(|footprint| footprint.name() == name)
                .unwrap();
            assert!(pads.contains_pad("1"), "{name}");
            assert!(pads.contains_pad("2"), "{name}");
        }

        let dc005 = generated_footprint_pads()
            .into_iter()
            .find(|footprint| {
                footprint.name() == "DC005_5p5x2p1_RightAngle_THT_Drawing_2_3_4_VERIFY"
            })
            .unwrap();
        assert!(dc005.contains_pad("2"));
        assert!(dc005.contains_pad("3"));
        assert!(dc005.contains_pad("4"));
    }

    #[test]
    fn dc005_footprint_ir_captures_drawing_pins_and_slots() {
        let footprint = generated_footprints()
            .into_iter()
            .find(|footprint| {
                footprint.name() == "DC005_5p5x2p1_RightAngle_THT_Drawing_2_3_4_VERIFY"
            })
            .unwrap()
            .into_ir();

        let pad = |number: &str| {
            footprint
                .pads()
                .iter()
                .find(|pad| pad.number == number)
                .unwrap_or_else(|| panic!("missing DC005 pad {number}"))
        };

        let pad2 = pad("2");
        let pad3 = pad("3");
        let pad4 = pad("4");

        assert_eq!((pad2.at.x, pad2.at.y), (11.0, 4.7));
        assert_eq!((pad3.at.x, pad3.at.y), (7.5, 0.0));
        assert_eq!((pad4.at.x, pad4.at.y), (13.7, 0.0));

        let drill2 = pad2.drill.unwrap();
        let drill3 = pad3.drill.unwrap();
        let drill4 = pad4.drill.unwrap();

        assert_eq!((drill2.x, drill2.y), (3.2, 1.0));
        assert_eq!((drill3.x, drill3.y), (1.0, 3.2));
        assert_eq!((drill4.x, drill4.y), (1.0, 3.2));
    }
}
