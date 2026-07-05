use crate::{FootprintMetadata, GeneratedFootprint};
use via_footprint_ir::{FootprintIr, GraphicText, Pad, PadShape, Point, Size};

use super::common::{Rect, add_production_outlines, add_reference_texts};
use super::headers::{RightRowOrder, tht_header_2x};

pub fn mp1584_4wire_adapter() -> GeneratedFootprint {
    let mut footprint = FootprintIr::new("BuckModule_4Wire_MP1584_Adapter")
        .description("Generated MP1584 4-wire adapter footprint; verify purchased module")
        .tag("via-generated")
        .tag("mp1584")
        .tag("verify");
    let pad_size = Size::new(2.2, 2.2);
    footprint
        .add_pad(Pad::thru_hole(
            "1",
            PadShape::Rect,
            Point::new(0.0, 0.0),
            pad_size,
            1.1,
        ))
        .add_pad(Pad::thru_hole(
            "2",
            PadShape::Circle,
            Point::new(0.0, 10.16),
            pad_size,
            1.1,
        ))
        .add_pad(Pad::thru_hole(
            "3",
            PadShape::Rect,
            Point::new(25.4, 0.0),
            pad_size,
            1.1,
        ))
        .add_pad(Pad::thru_hole(
            "4",
            PadShape::Circle,
            Point::new(25.4, 10.16),
            pad_size,
            1.1,
        ));

    let body = Rect::from_min_max(-3.0, -3.0, 28.4, 13.2);
    add_production_outlines(&mut footprint, body, 0.5);
    add_reference_texts(&mut footprint, "MP1584_BUCK_WIRE_ADAPTER", body);
    for (label, x, y) in [
        ("IN+", 0.0, -2.0),
        ("IN-", 0.0, 12.2),
        ("OUT+ 5V", 25.4, -2.0),
        ("OUT-", 25.4, 12.2),
    ] {
        footprint.add_text(GraphicText::user(label, Point::new(x, y), "F.SilkS"));
    }
    GeneratedFootprint::new(
        footprint,
        FootprintMetadata::generated("mp1584_4wire_adapter"),
    )
}

pub fn silentstepstick_tmc2209_v20_socket() -> GeneratedFootprint {
    let footprint = tht_header_2x("SilentStepStick_TMC2209_v20_CarrierSocket_2x8_Row12p70", 8)
        .row_spacing(12.70)
        .pad_diameter(1.8)
        .drill(1.0)
        .body_margin(1.27, 1.27)
        .right_row_order(RightRowOrder::BottomUp)
        .value("SILENTSTEPSTICK_TMC2209_V20_SOCKET")
        .row_labels(
            [
                "DIR",
                "STEP",
                "NC_J1_3",
                "NC_J1_4",
                "UART/SPREAD VERIFY",
                "MS2",
                "MS1",
                "EN",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>(),
            ["GND", "VIO", "OB2", "OB1", "OA1", "OA2", "GND", "VMOT"]
                .into_iter()
                .map(str::to_owned)
                .collect::<Vec<_>>(),
        )
        .build()
        .into_ir();

    GeneratedFootprint::new(
        footprint,
        FootprintMetadata::generated("silentstepstick_tmc2209_v20_socket"),
    )
}

pub fn esp32_s3_n16r8_devboard_socket() -> GeneratedFootprint {
    let left_labels = [
        "3V3",
        "3V3",
        "RST",
        "IO4 X_EN",
        "IO5 X_UART_TX",
        "IO6 X_UART_RX",
        "IO7 X_STEP",
        "IO15 X_DIR",
        "IO16 Y_EN",
        "IO17 Y_UART_TX",
        "IO18 Y_UART_RX",
        "IO8",
        "IO3",
        "IO46",
        "IO9 Y_STEP",
        "IO10 Y_DIR",
        "IO11 spare",
        "IO12",
        "IO13",
        "IO14",
        "5VIN",
        "GND",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let right_labels = [
        "GND",
        "TX",
        "RX",
        "IO1",
        "IO2",
        "IO42",
        "IO41",
        "IO40",
        "IO39 ESTOP",
        "IO38 spare",
        "IO37",
        "IO36",
        "IO35",
        "IO0 BOOT",
        "IO45",
        "IO48",
        "IO47",
        "IO21",
        "IO20",
        "IO19",
        "GND",
        "GND",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();

    let mut footprint = tht_header_2x("ESP32-S3-N16R8_DevBoard_2x22_P2.54_Row25.40", 22)
        .row_spacing(25.40)
        .pad_diameter(1.8)
        .drill(1.0)
        .body_margin(1.27, 5.025)
        .value("ESP32-S3-N16R8_DEVBOARD_SOCKET")
        .row_labels(left_labels, right_labels)
        .build()
        .into_ir();

    footprint
        .add_text(
            GraphicText::user("ESP32-S3 N16R8", Point::new(12.7, -6.725), "F.SilkS")
                .size(0.9, 0.9)
                .thickness(0.12),
        )
        .add_text(
            GraphicText::user("USB-C end", Point::new(12.7, 56.765), "F.SilkS").size(0.75, 0.75),
        )
        .add_rect(
            Point::new(3.6, 53.7),
            Point::new(21.8, 58.365),
            "F.SilkS",
            0.12,
        );

    GeneratedFootprint::new(
        footprint,
        FootprintMetadata::generated("esp32_s3_n16r8_devboard_socket"),
    )
}
