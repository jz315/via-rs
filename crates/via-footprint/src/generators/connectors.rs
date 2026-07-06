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

pub fn terminal_block_1x02_p5p08() -> GeneratedFootprint {
    terminal_block_1x("TB_1x02_P5.08", 2).build()
}

pub fn terminal_block_1x03_p5p08() -> GeneratedFootprint {
    terminal_block_1x("TB_1x03_P5.08", 3).build()
}

pub fn terminal_block_1x04_p5p08() -> GeneratedFootprint {
    terminal_block_1x("TB_1x04_P5.08", 4).build()
}

pub fn terminal_block_1x05_p5p08() -> GeneratedFootprint {
    terminal_block_1x("TB_1x05_P5.08", 5).build()
}

pub fn terminal_block_1x06_p5p08() -> GeneratedFootprint {
    terminal_block_1x("TB_1x06_P5.08", 6).build()
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

pub fn jst_xh_1x02_p2p54_vertical_verify() -> GeneratedFootprint {
    xh_vertical_1x("XH_1x02_P2.54_VERIFY", 2).build()
}

pub fn jst_xh_1x03_p2p54_vertical_verify() -> GeneratedFootprint {
    xh_vertical_1x("XH_1x03_P2.54_VERIFY", 3).build()
}

pub fn jst_xh_1x04_p2p54_vertical_verify() -> GeneratedFootprint {
    xh_vertical_1x("XH_1x04_P2.54_VERIFY", 4).build()
}

pub fn jst_xh_1x05_p2p54_vertical_verify() -> GeneratedFootprint {
    xh_vertical_1x("XH_1x05_P2.54_VERIFY", 5).build()
}

pub fn jst_xh_1x06_p2p54_vertical_verify() -> GeneratedFootprint {
    xh_vertical_1x("XH_1x06_P2.54_VERIFY", 6).build()
}

pub fn jst_ph_1x02_p2p00_vertical_verify() -> GeneratedFootprint {
    ph_vertical_1x("PH_1x02_P2.00_VERIFY", 2)
}

pub fn jst_ph_1x03_p2p00_vertical_verify() -> GeneratedFootprint {
    ph_vertical_1x("PH_1x03_P2.00_VERIFY", 3)
}

pub fn jst_ph_1x04_p2p00_vertical_verify() -> GeneratedFootprint {
    ph_vertical_1x("PH_1x04_P2.00_VERIFY", 4)
}

fn ph_vertical_1x(name: impl Into<String>, pins: usize) -> GeneratedFootprint {
    let width = pins.saturating_sub(1) as f64 * 2.0 + 4.2;
    xh_vertical_1x(name, pins)
        .pitch(2.0)
        .drill(0.8)
        .pad_diameter(1.45)
        .body(width, 5.8)
        .build()
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
