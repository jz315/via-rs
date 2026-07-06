use crate::{FootprintMetadata, GeneratedFootprint};
use via_footprint_ir::{FootprintIr, GraphicLine, GraphicText, Pad, PadShape, Point, Size};

use super::common::{Rect, add_production_outlines, add_reference_texts};

pub fn resistor_0402(name: impl Into<String>) -> GeneratedFootprint {
    smd_resistor_2terminal(
        name,
        "Generated 0402 / 1005 metric two-terminal resistor footprint",
        "0402",
        0.5,
        Size::new(0.55, 0.60),
        Rect::from_min_max(-0.5, -0.25, 0.5, 0.25),
        "resistor_0402",
    )
}

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

pub fn resistor_1206(name: impl Into<String>) -> GeneratedFootprint {
    smd_resistor_2terminal(
        name,
        "Generated 1206 / 3216 metric two-terminal resistor footprint",
        "1206",
        1.5,
        Size::new(1.25, 1.75),
        Rect::from_min_max(-1.6, -0.8, 1.6, 0.8),
        "resistor_1206",
    )
}

pub fn capacitor_0402(name: impl Into<String>) -> GeneratedFootprint {
    smd_capacitor_2terminal(
        name,
        "Generated 0402 / 1005 metric two-terminal capacitor footprint",
        "0402",
        0.5,
        Size::new(0.55, 0.60),
        Rect::from_min_max(-0.5, -0.25, 0.5, 0.25),
        "capacitor_0402",
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

pub fn capacitor_1206(name: impl Into<String>) -> GeneratedFootprint {
    smd_capacitor_2terminal(
        name,
        "Generated 1206 / 3216 metric two-terminal capacitor footprint",
        "1206",
        1.5,
        Size::new(1.25, 1.75),
        Rect::from_min_max(-1.6, -0.8, 1.6, 0.8),
        "capacitor_1206",
    )
}

pub fn polarized_capacitor_radial_d5p0_p2p00_verify(name: impl Into<String>) -> GeneratedFootprint {
    polarized_capacitor_radial_verify(name, 5.0, 2.0, 0.6, "capacitor_radial_d5p0_p2p00_verify")
}

pub fn polarized_capacitor_radial_d6p3_p2p50_verify(name: impl Into<String>) -> GeneratedFootprint {
    polarized_capacitor_radial_verify(name, 6.3, 2.5, 0.8, "capacitor_radial_d6p3_p2p50_verify")
}

pub fn polarized_capacitor_radial_d8p0_p3p50_verify(name: impl Into<String>) -> GeneratedFootprint {
    polarized_capacitor_radial_verify(name, 8.0, 3.5, 0.8, "capacitor_radial_d8p0_p3p50_verify")
}

pub fn polarized_capacitor_radial_d10p0_p5p00_verify(
    name: impl Into<String>,
) -> GeneratedFootprint {
    polarized_capacitor_radial_verify(name, 10.0, 5.0, 0.8, "capacitor_radial_d10p0_p5p00_verify")
}

fn polarized_capacitor_radial_verify(
    name: impl Into<String>,
    diameter: f64,
    pitch: f64,
    drill: f64,
    generator: &str,
) -> GeneratedFootprint {
    let name = name.into();
    let mut footprint = FootprintIr::new(name.clone())
        .description(format!(
            "Generated polarized radial capacitor D{diameter:.1}mm P{pitch:.2}mm footprint; verify purchased capacitor"
        ))
        .tag("via-generated")
        .tag("capacitor")
        .tag("polarized")
        .tag("radial")
        .tag("verify");

    footprint
        .add_pad(Pad::thru_hole(
            "1",
            PadShape::Rect,
            Point::new(-pitch / 2.0, 0.0),
            Size::new(1.8, 1.8),
            drill,
        ))
        .add_pad(Pad::thru_hole(
            "2",
            PadShape::Circle,
            Point::new(pitch / 2.0, 0.0),
            Size::new(1.8, 1.8),
            drill,
        ));

    let radius = diameter / 2.0;
    let body = Rect::from_min_max(-radius, -radius, radius, radius);
    add_production_outlines(&mut footprint, body, 0.35);
    add_reference_texts(&mut footprint, &name, body);
    footprint
        .add_text(
            GraphicText::user("+", Point::new(-radius + 0.8, 0.0), "F.SilkS")
                .size(1.0, 1.0)
                .thickness(0.15),
        )
        .add_text(
            GraphicText::user(
                format!("VERIFY D{diameter:.1} P{pitch:.2}"),
                Point::new(0.0, radius + 0.85),
                "F.Fab",
            )
            .size(0.6, 0.6)
            .thickness(0.08),
        );

    GeneratedFootprint::new(footprint, FootprintMetadata::generated(generator))
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
