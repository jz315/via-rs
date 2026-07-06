use crate::{FootprintMetadata, GeneratedFootprint};
use via_footprint_ir::{FootprintIr, GraphicText, Pad, PadShape, Point, Size};

use super::common::{COURTYARD_WIDTH, Rect, add_reference_texts};

pub fn testpad_round(name: impl Into<String>, diameter: f64) -> GeneratedFootprint {
    let name = name.into();
    let radius = diameter / 2.0;
    let mut footprint = FootprintIr::new(name.clone())
        .description(format!("Generated round SMD test pad D{diameter:.1}mm"))
        .tag("via-generated")
        .tag("testpad");

    footprint.add_pad(Pad::smd(
        "1",
        PadShape::Circle,
        Point::new(0.0, 0.0),
        Size::new(diameter, diameter),
    ));
    footprint.add_rect(
        Point::new(-radius - 0.35, -radius - 0.35),
        Point::new(radius + 0.35, radius + 0.35),
        "F.CrtYd",
        COURTYARD_WIDTH,
    );
    add_reference_texts(
        &mut footprint,
        &name,
        Rect::from_min_max(-radius, -radius, radius, radius),
    );

    GeneratedFootprint::new(footprint, FootprintMetadata::generated("testpad_round"))
}

pub fn testpad_1p0() -> GeneratedFootprint {
    testpad_round("TestPad_D1.0", 1.0)
}

pub fn testpad_1p5() -> GeneratedFootprint {
    testpad_round("TestPad_D1.5", 1.5)
}

pub fn testpad_2p0() -> GeneratedFootprint {
    testpad_round("TestPad_D2.0", 2.0)
}

pub fn fiducial_round(name: impl Into<String>, copper_diameter: f64) -> GeneratedFootprint {
    let name = name.into();
    let radius = copper_diameter / 2.0;
    let mut footprint = FootprintIr::new(name.clone())
        .description(format!(
            "Generated fiducial copper mark D{copper_diameter:.1}mm; verify fab mask rules"
        ))
        .tag("via-generated")
        .tag("fiducial")
        .tag("verify");

    let mut pad = Pad::smd(
        "1",
        PadShape::Circle,
        Point::new(0.0, 0.0),
        Size::new(copper_diameter, copper_diameter),
    );
    pad.layers = vec!["F.Cu".to_owned(), "F.Mask".to_owned()];
    footprint.add_pad(pad);
    footprint
        .add_rect(
            Point::new(-radius - 1.0, -radius - 1.0),
            Point::new(radius + 1.0, radius + 1.0),
            "F.CrtYd",
            COURTYARD_WIDTH,
        )
        .add_text(
            GraphicText::reference("REF**", Point::new(0.0, -2.2), "F.Fab")
                .size(0.8, 0.8)
                .thickness(0.1),
        )
        .add_text(
            GraphicText::value(name.clone(), Point::new(0.0, 2.2), "F.Fab")
                .size(0.8, 0.8)
                .thickness(0.1),
        );

    GeneratedFootprint::new(
        footprint,
        FootprintMetadata::generated("fiducial_round")
            .notes("Generated fiducial; confirm copper diameter and solder-mask clearance with board fab before production"),
    )
}

pub fn fiducial_1p0() -> GeneratedFootprint {
    fiducial_round("Fiducial_D1.0", 1.0)
}

pub fn mounting_hole_np(name: impl Into<String>, drill: f64) -> GeneratedFootprint {
    let name = name.into();
    let radius = drill / 2.0;
    let mut footprint = FootprintIr::new(name.clone())
        .description(format!("Generated non-plated mounting hole D{drill:.1}mm"))
        .tag("via-generated")
        .tag("mounting-hole")
        .tag("npth");

    footprint.add_pad(Pad::np_thru_hole(
        "1",
        PadShape::Circle,
        Point::new(0.0, 0.0),
        Size::new(drill, drill),
        drill,
    ));
    footprint.add_rect(
        Point::new(-radius - 0.8, -radius - 0.8),
        Point::new(radius + 0.8, radius + 0.8),
        "F.CrtYd",
        COURTYARD_WIDTH,
    );
    add_reference_texts(
        &mut footprint,
        &name,
        Rect::from_min_max(-radius, -radius, radius, radius),
    );

    GeneratedFootprint::new(footprint, FootprintMetadata::generated("mounting_hole_np"))
}

pub fn mounting_hole_plated(
    name: impl Into<String>,
    drill: f64,
    pad_diameter: f64,
) -> GeneratedFootprint {
    let name = name.into();
    let radius = pad_diameter / 2.0;
    let mut footprint = FootprintIr::new(name.clone())
        .description(format!(
            "Generated plated mounting hole drill D{drill:.1}mm pad D{pad_diameter:.1}mm"
        ))
        .tag("via-generated")
        .tag("mounting-hole")
        .tag("pth");

    footprint.add_pad(Pad::thru_hole(
        "1",
        PadShape::Circle,
        Point::new(0.0, 0.0),
        Size::new(pad_diameter, pad_diameter),
        drill,
    ));
    footprint.add_rect(
        Point::new(-radius - 0.8, -radius - 0.8),
        Point::new(radius + 0.8, radius + 0.8),
        "F.CrtYd",
        COURTYARD_WIDTH,
    );
    add_reference_texts(
        &mut footprint,
        &name,
        Rect::from_min_max(-radius, -radius, radius, radius),
    );

    GeneratedFootprint::new(
        footprint,
        FootprintMetadata::generated("mounting_hole_plated"),
    )
}

pub fn mounting_hole_m2_np() -> GeneratedFootprint {
    mounting_hole_np("MH_M2_NPTH_D2.2", 2.2)
}

pub fn mounting_hole_m25_np() -> GeneratedFootprint {
    mounting_hole_np("MH_M2.5_NPTH_D2.7", 2.7)
}

pub fn mounting_hole_m3_np() -> GeneratedFootprint {
    mounting_hole_np("MH_M3_NPTH_D3.2", 3.2)
}

pub fn mounting_hole_m2_pth() -> GeneratedFootprint {
    mounting_hole_plated("MH_M2_PTH_D2.2_P4.2", 2.2, 4.2)
}

pub fn mounting_hole_m25_pth() -> GeneratedFootprint {
    mounting_hole_plated("MH_M2.5_PTH_D2.7_P4.8", 2.7, 4.8)
}

pub fn mounting_hole_m3_pth() -> GeneratedFootprint {
    mounting_hole_plated("MH_M3_PTH_D3.2_P5.5", 3.2, 5.5)
}
