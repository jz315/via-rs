use crate::{FootprintMetadata, GeneratedFootprint};
use via_footprint_ir::{FootprintIr, GraphicLine, GraphicText, Pad, PadShape, Point, Size};

use super::common::{Rect, add_production_outlines, add_reference_texts};

pub fn resistor_0603(name: impl Into<String>) -> GeneratedFootprint {
    smd_resistor_2terminal(
        name,
        "Generated 0603 / 1608 metric two-terminal resistor footprint",
        "0603",
        0.8,
        Size::new(0.9, 0.95),
        Rect::from_min_max(-0.8, -0.4, 0.8, 0.4),
        "resistor_0603",
    )
}

pub fn resistor_0805(name: impl Into<String>) -> GeneratedFootprint {
    smd_resistor_2terminal(
        name,
        "Generated 0805 / 2012 metric two-terminal resistor footprint",
        "0805",
        0.95,
        Size::new(1.15, 1.35),
        Rect::from_min_max(-1.0, -0.65, 1.0, 0.65),
        "resistor_0805",
    )
}

pub fn capacitor_0603(name: impl Into<String>) -> GeneratedFootprint {
    smd_capacitor_2terminal(
        name,
        "Generated 0603 / 1608 metric two-terminal capacitor footprint",
        "0603",
        0.8,
        Size::new(0.9, 0.95),
        Rect::from_min_max(-0.8, -0.4, 0.8, 0.4),
        "capacitor_0603",
    )
}

pub fn capacitor_0805(name: impl Into<String>) -> GeneratedFootprint {
    smd_capacitor_2terminal(
        name,
        "Generated 0805 / 2012 metric two-terminal capacitor footprint",
        "0805",
        0.95,
        Size::new(1.15, 1.35),
        Rect::from_min_max(-1.0, -0.65, 1.0, 0.65),
        "capacitor_0805",
    )
}

pub fn polarized_capacitor_radial_d6p3_p2p50_verify(name: impl Into<String>) -> GeneratedFootprint {
    let name = name.into();
    let mut footprint = FootprintIr::new(name.clone())
        .description("Generated polarized radial capacitor D6.3mm P2.50mm footprint; verify purchased capacitor")
        .tag("via-generated")
        .tag("capacitor")
        .tag("polarized")
        .tag("radial")
        .tag("verify");

    footprint
        .add_pad(Pad::thru_hole(
            "1",
            PadShape::Rect,
            Point::new(-1.25, 0.0),
            Size::new(1.8, 1.8),
            0.8,
        ))
        .add_pad(Pad::thru_hole(
            "2",
            PadShape::Circle,
            Point::new(1.25, 0.0),
            Size::new(1.8, 1.8),
            0.8,
        ));

    let body = Rect::from_min_max(-3.15, -3.15, 3.15, 3.15);
    add_production_outlines(&mut footprint, body, 0.35);
    add_reference_texts(&mut footprint, &name, body);
    footprint
        .add_text(
            GraphicText::user("+", Point::new(-2.35, 0.0), "F.SilkS")
                .size(1.0, 1.0)
                .thickness(0.15),
        )
        .add_text(
            GraphicText::user("VERIFY D6.3 P2.5", Point::new(0.0, 4.0), "F.Fab")
                .size(0.6, 0.6)
                .thickness(0.08),
        );

    GeneratedFootprint::new(
        footprint,
        FootprintMetadata::generated("capacitor_radial_d6p3_p2p50_verify"),
    )
}

fn smd_capacitor_2terminal(
    name: impl Into<String>,
    description: &str,
    package_label: &str,
    pad_x: f64,
    pad_size: Size,
    body: Rect,
    generator: &str,
) -> GeneratedFootprint {
    let name = name.into();
    let mut footprint = FootprintIr::new(name.clone())
        .description(description)
        .tag("via-generated")
        .tag("capacitor")
        .tag(package_label);

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

    add_production_outlines(&mut footprint, body, 0.25);
    add_reference_texts(&mut footprint, &name, body);
    footprint
        .add_line(GraphicLine::new(
            Point::new(-0.2, -0.45),
            Point::new(-0.2, 0.45),
            "F.Fab",
            0.08,
        ))
        .add_text(
            GraphicText::user(package_label, Point::new(0.0, 1.1), "F.Fab")
                .size(0.55, 0.55)
                .thickness(0.08),
        );

    GeneratedFootprint::new(footprint, FootprintMetadata::generated(generator))
}

fn smd_resistor_2terminal(
    name: impl Into<String>,
    description: &str,
    package_label: &str,
    pad_x: f64,
    pad_size: Size,
    body: Rect,
    generator: &str,
) -> GeneratedFootprint {
    let name = name.into();
    let mut footprint = FootprintIr::new(name.clone())
        .description(description)
        .tag("via-generated")
        .tag("resistor")
        .tag(package_label);

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

    add_production_outlines(&mut footprint, body, 0.25);
    add_reference_texts(&mut footprint, &name, body);
    footprint.add_text(
        GraphicText::user(package_label, Point::new(0.0, 1.1), "F.Fab")
            .size(0.55, 0.55)
            .thickness(0.08),
    );

    GeneratedFootprint::new(footprint, FootprintMetadata::generated(generator))
}
