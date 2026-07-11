use crate::{FootprintMetadata, GeneratedFootprint};
use via_footprint_ir::{FootprintIr, GraphicLine, GraphicText, Pad, PadShape, Point, Size};

use super::common::{Rect, add_production_outlines, add_reference_texts};

pub fn led_0603() -> GeneratedFootprint {
    smd_led_2terminal(
        "LED_0603",
        "Generated 0603 / 1608 metric LED footprint",
        0.8,
        Size::new(0.9, 0.95),
        Rect::from_min_max(-0.8, -0.4, 0.8, 0.4),
        "led_0603",
    )
}

pub fn led_0805() -> GeneratedFootprint {
    smd_led_2terminal(
        "LED_0805",
        "Generated 0805 / 2012 metric LED footprint",
        0.95,
        Size::new(1.15, 1.35),
        Rect::from_min_max(-1.0, -0.65, 1.0, 0.65),
        "led_0805",
    )
}

pub fn sod323() -> GeneratedFootprint {
    smd_diode_2terminal(
        "SOD-323",
        "Generated SOD-323 diode footprint",
        1.35,
        Size::new(0.65, 0.9),
        Rect::from_min_max(-1.0, -0.65, 1.0, 0.65),
        "sod323",
    )
}

pub fn sod123() -> GeneratedFootprint {
    smd_diode_2terminal(
        "SOD-123",
        "Generated SOD-123 diode footprint",
        2.0,
        Size::new(1.0, 1.25),
        Rect::from_min_max(-1.9, -0.9, 1.9, 0.9),
        "sod123",
    )
}

pub fn sot23_3() -> GeneratedFootprint {
    sot23("SOT-23-3", 3)
}

pub fn sot23_5() -> GeneratedFootprint {
    sot23("SOT-23-5", 5)
}

pub fn sot23_6() -> GeneratedFootprint {
    sot23("SOT-23-6", 6)
}

pub fn sot223() -> GeneratedFootprint {
    let name = "SOT-223";
    let mut footprint = FootprintIr::new(name)
        .description("Generated SOT-223 footprint")
        .tag("via-generated")
        .tag("sot223")
        .tag("verify");

    for (idx, x) in [-2.3, 0.0, 2.3].into_iter().enumerate() {
        footprint.add_pad(Pad::smd(
            (idx + 1).to_string(),
            PadShape::Rect,
            Point::new(x, 3.15),
            Size::new(1.2, 2.0),
        ));
    }
    footprint.add_pad(Pad::smd(
        "4",
        PadShape::Rect,
        Point::new(0.0, -3.2),
        Size::new(3.8, 2.2),
    ));

    let body = Rect::from_min_max(-3.3, -3.4, 3.3, 3.4);
    add_production_outlines(&mut footprint, body, 0.45);
    add_reference_texts(&mut footprint, name, body);

    GeneratedFootprint::new(
        footprint,
        FootprintMetadata::generated("sot223")
            .notes("Generated generic SOT-223; verify pad sizes against selected package variant"),
    )
}

pub fn soic8() -> GeneratedFootprint {
    gullwing_ic(GullwingIcSpec {
        name: "SOIC-8",
        pins: 8,
        pitch: 1.27,
        row_x: 5.4,
        pad_size: Size::new(1.8, 0.6),
        body_width: 3.9,
        body_length: 4.9,
        generator: "soic8",
    })
}

pub fn soic14() -> GeneratedFootprint {
    gullwing_ic(GullwingIcSpec {
        name: "SOIC-14",
        pins: 14,
        pitch: 1.27,
        row_x: 5.4,
        pad_size: Size::new(1.8, 0.6),
        body_width: 3.9,
        body_length: 8.7,
        generator: "soic14",
    })
}

pub fn soic16() -> GeneratedFootprint {
    gullwing_ic(GullwingIcSpec {
        name: "SOIC-16",
        pins: 16,
        pitch: 1.27,
        row_x: 5.4,
        pad_size: Size::new(1.8, 0.6),
        body_width: 3.9,
        body_length: 9.9,
        generator: "soic16",
    })
}

pub fn tssop16() -> GeneratedFootprint {
    gullwing_ic(GullwingIcSpec {
        name: "TSSOP-16",
        pins: 16,
        pitch: 0.65,
        row_x: 4.5,
        pad_size: Size::new(1.45, 0.35),
        body_width: 4.4,
        body_length: 5.0,
        generator: "tssop16",
    })
}

pub fn tssop20() -> GeneratedFootprint {
    gullwing_ic(GullwingIcSpec {
        name: "TSSOP-20",
        pins: 20,
        pitch: 0.65,
        row_x: 4.5,
        pad_size: Size::new(1.45, 0.35),
        body_width: 4.4,
        body_length: 6.5,
        generator: "tssop20",
    })
}

fn smd_led_2terminal(
    name: &str,
    description: &str,
    pad_x: f64,
    pad_size: Size,
    body: Rect,
    generator: &str,
) -> GeneratedFootprint {
    let mut footprint = FootprintIr::new(name)
        .description(description)
        .tag("via-generated")
        .tag("led")
        .tag("verify");

    add_two_terminal_pads(&mut footprint, pad_x, pad_size);
    add_production_outlines(&mut footprint, body, 0.25);
    add_reference_texts(&mut footprint, name, body);
    add_cathode_marker(&mut footprint, body);

    GeneratedFootprint::new(
        footprint,
        FootprintMetadata::generated(generator)
            .notes("Generated LED footprint; verify polarity marking and package dimensions before production"),
    )
}

fn smd_diode_2terminal(
    name: &str,
    description: &str,
    pad_x: f64,
    pad_size: Size,
    body: Rect,
    generator: &str,
) -> GeneratedFootprint {
    let mut footprint = FootprintIr::new(name)
        .description(description)
        .tag("via-generated")
        .tag("diode")
        .tag("verify");

    add_two_terminal_pads(&mut footprint, pad_x, pad_size);
    add_production_outlines(&mut footprint, body, 0.3);
    add_reference_texts(&mut footprint, name, body);
    add_cathode_marker(&mut footprint, body);

    GeneratedFootprint::new(
        footprint,
        FootprintMetadata::generated(generator)
            .notes("Generated diode footprint; verify cathode orientation and package dimensions before production"),
    )
}

fn add_two_terminal_pads(footprint: &mut FootprintIr, pad_x: f64, pad_size: Size) {
    footprint
        .add_pad(Pad::smd(
            "1",
            PadShape::Rect,
            Point::new(-pad_x, 0.0),
            pad_size,
        ))
        .add_pad(Pad::smd(
            "2",
            PadShape::Rect,
            Point::new(pad_x, 0.0),
            pad_size,
        ));
}

fn add_cathode_marker(footprint: &mut FootprintIr, body: Rect) {
    footprint
        .add_line(GraphicLine::new(
            Point::new(body.min_x + 0.2, body.min_y),
            Point::new(body.min_x + 0.2, body.max_y),
            "F.SilkS",
            0.12,
        ))
        .add_text(
            GraphicText::user("K", Point::new(body.min_x - 0.55, 0.0), "F.Fab")
                .size(0.55, 0.55)
                .thickness(0.08),
        );
}

fn sot23(name: &str, pins: usize) -> GeneratedFootprint {
    let mut footprint = FootprintIr::new(name)
        .description(format!("Generated {name} footprint"))
        .tag("via-generated")
        .tag("sot23")
        .tag("verify");
    let left_count = pins.div_ceil(2);
    let right_count = pins - left_count;
    let pitch = 0.95;
    let row_x = 1.25;
    let pad = Size::new(1.05, 0.65);

    for idx in 0..left_count {
        let y = centered_y(idx, left_count, pitch);
        footprint.add_pad(Pad::smd(
            (idx + 1).to_string(),
            PadShape::RoundRect,
            Point::new(-row_x, y),
            pad,
        ));
    }

    for idx in 0..right_count {
        let number = left_count + idx + 1;
        let y = centered_y(idx, right_count, pitch);
        footprint.add_pad(Pad::smd(
            number.to_string(),
            PadShape::RoundRect,
            Point::new(row_x, y),
            pad,
        ));
    }

    let body = Rect::from_min_max(-1.45, -1.55, 1.45, 1.55);
    add_production_outlines(&mut footprint, body, 0.35);
    add_reference_texts(&mut footprint, name, body);
    footprint.add_text(
        GraphicText::user(
            "1",
            Point::new(-1.9, centered_y(0, left_count, pitch)),
            "F.SilkS",
        )
        .size(0.55, 0.55)
        .thickness(0.08),
    );

    GeneratedFootprint::new(
        footprint,
        FootprintMetadata::generated("sot23")
            .notes("Generated generic SOT-23 footprint; verify selected variant before production"),
    )
}

struct GullwingIcSpec {
    name: &'static str,
    pins: usize,
    pitch: f64,
    row_x: f64,
    pad_size: Size,
    body_width: f64,
    body_length: f64,
    generator: &'static str,
}

fn gullwing_ic(spec: GullwingIcSpec) -> GeneratedFootprint {
    let GullwingIcSpec {
        name,
        pins,
        pitch,
        row_x,
        pad_size,
        body_width,
        body_length,
        generator,
    } = spec;
    let mut footprint = FootprintIr::new(name)
        .description(format!("Generated {name} gullwing IC footprint"))
        .tag("via-generated")
        .tag("ic")
        .tag("gullwing")
        .tag("verify");
    let pins_per_side = pins / 2;

    for idx in 0..pins_per_side {
        let y = centered_y(idx, pins_per_side, pitch);
        footprint.add_pad(Pad::smd(
            (idx + 1).to_string(),
            PadShape::RoundRect,
            Point::new(-row_x / 2.0, y),
            pad_size,
        ));
    }

    for idx in 0..pins_per_side {
        let number = pins - idx;
        let y = centered_y(idx, pins_per_side, pitch);
        footprint.add_pad(Pad::smd(
            number.to_string(),
            PadShape::RoundRect,
            Point::new(row_x / 2.0, y),
            pad_size,
        ));
    }

    let body = Rect::from_min_max(
        -body_width / 2.0,
        -body_length / 2.0,
        body_width / 2.0,
        body_length / 2.0,
    );
    add_production_outlines(&mut footprint, body, 0.35);
    add_reference_texts(&mut footprint, name, body);
    footprint.add_text(
        GraphicText::user(
            "1",
            Point::new(-body_width / 2.0 - 0.8, -body_length / 2.0 + 0.5),
            "F.SilkS",
        )
        .size(0.55, 0.55)
        .thickness(0.08),
    );

    GeneratedFootprint::new(
        footprint,
        FootprintMetadata::generated(generator).notes(
            "Generated generic gullwing IC footprint; verify exact land pattern before production",
        ),
    )
}

fn centered_y(index: usize, count: usize, pitch: f64) -> f64 {
    let center = (count.saturating_sub(1)) as f64 / 2.0;
    (index as f64 - center) * pitch
}
