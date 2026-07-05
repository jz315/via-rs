use crate::{FootprintBuildError, FootprintMetadata, GeneratedFootprint};
use via_footprint_ir::{FootprintIr, GraphicText, Pad, PadShape, Point, Size, TextJustify};

use super::common::{FAB_WIDTH, Rect, add_production_outlines, add_reference_texts};
use super::headers::tht_header_1x;

#[derive(Debug, Clone)]
pub struct TerminalBlock1x {
    name: String,
    pins: usize,
    pitch: f64,
    drill: f64,
    pad_diameter: f64,
    body_depth: f64,
    courtyard_margin: f64,
}

pub fn terminal_block_1x(name: impl Into<String>, pins: usize) -> TerminalBlock1x {
    TerminalBlock1x {
        name: name.into(),
        pins,
        pitch: 5.08,
        drill: 1.25,
        pad_diameter: 2.25,
        body_depth: 5.0,
        courtyard_margin: 0.5,
    }
}

pub fn dc005_5p5x2p1_right_angle_drawing_verify(name: impl Into<String>) -> GeneratedFootprint {
    let name = name.into();
    let mut footprint = FootprintIr::new(name.clone())
        .description("DC-005 style right-angle DC barrel jack, 5.5x2.1mm, user drawing pins 2/3/4")
        .tag("via-generated")
        .tag("dc005")
        .tag("barrel-jack")
        .tag("verify");

    footprint
        .add_pad(Pad::thru_hole_slot(
            "3",
            PadShape::Oval,
            Point::new(7.5, 0.0),
            Size::new(2.2, 4.4),
            1.0,
            3.2,
        ))
        .add_pad(Pad::thru_hole_slot(
            "4",
            PadShape::Oval,
            Point::new(13.7, 0.0),
            Size::new(2.2, 4.4),
            1.0,
            3.2,
        ))
        .add_pad(Pad::thru_hole_slot(
            "2",
            PadShape::Oval,
            Point::new(11.0, 4.7),
            Size::new(4.4, 2.2),
            3.2,
            1.0,
        ))
        .add_rect(
            Point::new(0.0, -4.5),
            Point::new(14.0, 4.5),
            "F.SilkS",
            0.15,
        )
        .add_rect(Point::new(0.0, -4.5), Point::new(14.0, 4.5), "F.Fab", 0.1)
        .add_rect(
            Point::new(-0.7, -5.5),
            Point::new(15.0, 6.4),
            "F.CrtYd",
            0.05,
        )
        .add_text(GraphicText::reference("REF**", Point::new(0.0, -3.0), "F.SilkS").size(1.0, 1.0))
        .add_text(GraphicText::value(name.clone(), Point::new(0.0, 3.0), "F.Fab").size(1.0, 1.0))
        .add_text(GraphicText::user("4=+12V", Point::new(13.7, -5.8), "F.SilkS").size(0.75, 0.75))
        .add_text(
            GraphicText::user("2/3 VERIFY", Point::new(9.4, 7.1), "F.SilkS").size(0.75, 0.75),
        );

    GeneratedFootprint::new(
        footprint,
        FootprintMetadata::generated("dc005_5p5x2p1_right_angle_drawing_verify")
            .notes("Drawing-based DC-005 geometry; pad 4 = 12V_IN, pad 2 tentative sleeve/GND, pad 3 tentative switched contact. Pads use oval slot drills in VIA IR; verify against the purchased jack before fabrication."),
    )
}

impl TerminalBlock1x {
    pub fn pitch(mut self, pitch: f64) -> Self {
        self.pitch = pitch;
        self
    }

    pub fn drill(mut self, drill: f64) -> Self {
        self.drill = drill;
        self
    }

    pub fn pad_diameter(mut self, pad_diameter: f64) -> Self {
        self.pad_diameter = pad_diameter;
        self
    }

    pub fn body_depth(mut self, body_depth: f64) -> Self {
        self.body_depth = body_depth;
        self
    }

    pub fn build(self) -> GeneratedFootprint {
        self.try_build()
            .expect("terminal block builder parameters should be valid")
    }

    pub fn try_build(self) -> Result<GeneratedFootprint, FootprintBuildError> {
        if self.pins == 0 {
            return Err(FootprintBuildError::invalid_parameter(
                "terminal block must have at least one pin",
            ));
        }

        let mut footprint = FootprintIr::new(self.name.clone())
            .description(format!("Generated 1x{} terminal block", self.pins))
            .tag("via-generated")
            .tag("terminal-block");
        let pad_size = Size::new(self.pad_diameter, self.pad_diameter);

        for idx in 0..self.pins {
            let number = idx + 1;
            footprint.add_pad(Pad::thru_hole(
                number.to_string(),
                if number == 1 {
                    PadShape::Rect
                } else {
                    PadShape::Circle
                },
                Point::new(idx as f64 * self.pitch, 0.0),
                pad_size,
                self.drill,
            ));
        }

        let body = Rect::from_min_max(
            -self.pitch / 2.0,
            -self.body_depth / 2.0,
            (self.pins - 1) as f64 * self.pitch + self.pitch / 2.0,
            self.body_depth / 2.0,
        );
        add_production_outlines(&mut footprint, body, self.courtyard_margin);
        add_reference_texts(&mut footprint, &self.name, body);

        Ok(GeneratedFootprint::new(
            footprint,
            FootprintMetadata::generated("terminal_block_1x"),
        ))
    }
}

#[derive(Debug, Clone)]
pub struct XhVertical1x {
    name: String,
    pins: usize,
    pitch: f64,
    drill: f64,
    pad_diameter: f64,
    body_width: f64,
    body_depth: f64,
    labels: Vec<String>,
}

pub fn xh_vertical_1x(name: impl Into<String>, pins: usize) -> XhVertical1x {
    XhVertical1x {
        name: name.into(),
        pins,
        pitch: 2.54,
        drill: 1.0,
        pad_diameter: 1.7,
        body_width: pins.saturating_sub(1) as f64 * 2.54 + 5.0,
        body_depth: 7.0,
        labels: Vec::new(),
    }
}

impl XhVertical1x {
    pub fn pitch(mut self, pitch: f64) -> Self {
        self.pitch = pitch;
        self
    }

    pub fn drill(mut self, drill: f64) -> Self {
        self.drill = drill;
        self
    }

    pub fn pad_diameter(mut self, pad_diameter: f64) -> Self {
        self.pad_diameter = pad_diameter;
        self
    }

    pub fn body(mut self, width: f64, depth: f64) -> Self {
        self.body_width = width;
        self.body_depth = depth;
        self
    }

    pub fn pin_labels(mut self, labels: impl Into<Vec<String>>) -> Self {
        self.labels = labels.into();
        self
    }

    pub fn build(self) -> GeneratedFootprint {
        self.try_build()
            .expect("XH vertical builder parameters should be valid")
    }

    pub fn try_build(self) -> Result<GeneratedFootprint, FootprintBuildError> {
        let mut footprint = tht_header_1x(self.name.clone(), self.pins)
            .pitch(self.pitch)
            .drill(self.drill)
            .pad_diameter(self.pad_diameter)
            .value(self.name.clone())
            .try_build()?
            .into_ir();

        footprint
            .add_text(
                GraphicText::user("XH vertical VERIFY", Point::new(0.0, -4.6), "F.SilkS")
                    .size(0.75, 0.75),
            )
            .add_rect(
                Point::new(-self.body_depth / 2.0, -2.5),
                Point::new(self.body_depth / 2.0, self.body_width - 2.5),
                "F.Fab",
                FAB_WIDTH,
            );

        for (idx, label) in self.labels.iter().enumerate() {
            footprint.add_text(
                GraphicText::user(label, Point::new(3.0, idx as f64 * self.pitch), "F.Fab")
                    .size(0.62, 0.62)
                    .thickness(0.08)
                    .justify(TextJustify::Left),
            );
        }

        Ok(GeneratedFootprint::new(
            footprint,
            FootprintMetadata::generated("xh_vertical_1x"),
        ))
    }
}
