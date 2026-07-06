use crate::{FootprintBuildError, FootprintMetadata, GeneratedFootprint};
use via_footprint_ir::{FootprintIr, Pad, PadShape, Point, Size};

use super::common::{Rect, add_production_outlines, add_reference_texts, add_row_labels};

#[derive(Debug, Clone)]
pub struct ThtHeader1x {
    name: String,
    pins: usize,
    pitch: f64,
    drill: f64,
    pad_diameter: f64,
    body_margin_x: f64,
    body_margin_y: f64,
    courtyard_margin: f64,
    value: Option<String>,
}

pub fn tht_header_1x(name: impl Into<String>, pins: usize) -> ThtHeader1x {
    ThtHeader1x {
        name: name.into(),
        pins,
        pitch: 2.54,
        drill: 1.0,
        pad_diameter: 1.7,
        body_margin_x: 1.27,
        body_margin_y: 1.27,
        courtyard_margin: 0.5,
        value: None,
    }
}

pub fn pin_header_1x02_p2p54() -> GeneratedFootprint {
    tht_header_1x("Pin_1x02_P2.54", 2).build()
}

pub fn pin_header_1x03_p2p54() -> GeneratedFootprint {
    tht_header_1x("Pin_1x03_P2.54", 3).build()
}

pub fn pin_header_1x04_p2p54() -> GeneratedFootprint {
    tht_header_1x("Pin_1x04_P2.54", 4).build()
}

pub fn pin_header_1x05_p2p54() -> GeneratedFootprint {
    tht_header_1x("Pin_1x05_P2.54", 5).build()
}

pub fn pin_header_1x06_p2p54() -> GeneratedFootprint {
    tht_header_1x("Pin_1x06_P2.54", 6).build()
}

pub fn pin_header_1x08_p2p54() -> GeneratedFootprint {
    tht_header_1x("Pin_1x08_P2.54", 8).build()
}

pub fn pin_header_1x10_p2p54() -> GeneratedFootprint {
    tht_header_1x("Pin_1x10_P2.54", 10).build()
}

pub fn pin_header_1x20_p2p54() -> GeneratedFootprint {
    tht_header_1x("Pin_1x20_P2.54", 20).build()
}

impl ThtHeader1x {
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

    pub fn outline_margin(mut self, outline_margin: f64) -> Self {
        self.body_margin_x = outline_margin;
        self.body_margin_y = outline_margin;
        self
    }

    pub fn body_margin(mut self, x: f64, y: f64) -> Self {
        self.body_margin_x = x;
        self.body_margin_y = y;
        self
    }

    pub fn courtyard_margin(mut self, courtyard_margin: f64) -> Self {
        self.courtyard_margin = courtyard_margin;
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn build(self) -> GeneratedFootprint {
        self.try_build()
            .expect("THT header builder parameters should be valid")
    }

    pub fn try_build(self) -> Result<GeneratedFootprint, FootprintBuildError> {
        if self.pins == 0 {
            return Err(FootprintBuildError::invalid_parameter(
                "THT header must have at least one pin",
            ));
        }

        let mut footprint = FootprintIr::new(self.name.clone())
            .description(format!("Generated 1x{} THT pin header", self.pins))
            .tag("via-generated")
            .tag("tht-header");
        let pad_size = Size::new(self.pad_diameter, self.pad_diameter);

        for idx in 0..self.pins {
            let number = idx + 1;
            let shape = if number == 1 {
                PadShape::Rect
            } else {
                PadShape::Circle
            };
            footprint.add_pad(Pad::thru_hole(
                number.to_string(),
                shape,
                Point::new(0.0, idx as f64 * self.pitch),
                pad_size,
                self.drill,
            ));
        }

        let body = Rect::from_min_max(
            -self.body_margin_x,
            -self.body_margin_y,
            self.body_margin_x,
            (self.pins - 1) as f64 * self.pitch + self.body_margin_y,
        );
        add_production_outlines(&mut footprint, body, self.courtyard_margin);
        add_reference_texts(&mut footprint, &self.value.unwrap_or(self.name), body);

        Ok(GeneratedFootprint::new(
            footprint,
            FootprintMetadata::generated("tht_header_1x"),
        ))
    }
}

#[derive(Debug, Clone)]
pub struct ThtHeader2x {
    name: String,
    pins_per_row: usize,
    pitch: f64,
    row_spacing: f64,
    drill: f64,
    pad_diameter: f64,
    body_margin_x: f64,
    body_margin_y: f64,
    courtyard_margin: f64,
    right_row_order: RightRowOrder,
    value: Option<String>,
    left_labels: Vec<String>,
    right_labels: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RightRowOrder {
    TopDown,
    BottomUp,
}

pub fn tht_header_2x(name: impl Into<String>, pins_per_row: usize) -> ThtHeader2x {
    ThtHeader2x {
        name: name.into(),
        pins_per_row,
        pitch: 2.54,
        row_spacing: 2.54,
        drill: 1.0,
        pad_diameter: 1.7,
        body_margin_x: 1.27,
        body_margin_y: 1.27,
        courtyard_margin: 0.5,
        right_row_order: RightRowOrder::TopDown,
        value: None,
        left_labels: Vec::new(),
        right_labels: Vec::new(),
    }
}

pub fn pin_socket_2x08_p2p54_row12p70() -> GeneratedFootprint {
    tht_header_2x("Socket_2x08_R12.7", 8)
        .row_spacing(12.70)
        .value("Socket_2x08_R12.7")
        .build()
}

pub fn pin_header_2x03_p2p54() -> GeneratedFootprint {
    tht_header_2x("Pin_2x03_P2.54", 3).build()
}

pub fn pin_header_2x05_p2p54() -> GeneratedFootprint {
    tht_header_2x("Pin_2x05_P2.54", 5).build()
}

pub fn pin_header_2x10_p2p54() -> GeneratedFootprint {
    tht_header_2x("Pin_2x10_P2.54", 10).build()
}

pub fn pin_header_2x20_p2p54() -> GeneratedFootprint {
    tht_header_2x("Pin_2x20_P2.54", 20).build()
}

impl ThtHeader2x {
    pub fn pitch(mut self, pitch: f64) -> Self {
        self.pitch = pitch;
        self
    }

    pub fn row_spacing(mut self, row_spacing: f64) -> Self {
        self.row_spacing = row_spacing;
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

    pub fn body_margin(mut self, x: f64, y: f64) -> Self {
        self.body_margin_x = x;
        self.body_margin_y = y;
        self
    }

    pub fn courtyard_margin(mut self, courtyard_margin: f64) -> Self {
        self.courtyard_margin = courtyard_margin;
        self
    }

    pub fn right_row_order(mut self, order: RightRowOrder) -> Self {
        self.right_row_order = order;
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn row_labels(
        mut self,
        left: impl Into<Vec<String>>,
        right: impl Into<Vec<String>>,
    ) -> Self {
        self.left_labels = left.into();
        self.right_labels = right.into();
        self
    }

    pub fn build(self) -> GeneratedFootprint {
        self.try_build()
            .expect("2xN THT header builder parameters should be valid")
    }

    pub fn try_build(self) -> Result<GeneratedFootprint, FootprintBuildError> {
        if self.pins_per_row == 0 {
            return Err(FootprintBuildError::invalid_parameter(
                "2xN THT header must have at least one pin per row",
            ));
        }

        let mut footprint = FootprintIr::new(self.name.clone())
            .description(format!(
                "Generated 2x{} THT socket, row spacing {} mm",
                self.pins_per_row, self.row_spacing
            ))
            .tag("via-generated")
            .tag("tht-socket")
            .tag("2xn");
        let pad_size = Size::new(self.pad_diameter, self.pad_diameter);
        let last_y = (self.pins_per_row - 1) as f64 * self.pitch;

        for idx in 0..self.pins_per_row {
            let number = idx + 1;
            let shape = if number == 1 {
                PadShape::Rect
            } else {
                PadShape::Circle
            };
            footprint.add_pad(Pad::thru_hole(
                number.to_string(),
                shape,
                Point::new(0.0, idx as f64 * self.pitch),
                pad_size,
                self.drill,
            ));
        }

        for idx in 0..self.pins_per_row {
            let number = self.pins_per_row + idx + 1;
            let y = match self.right_row_order {
                RightRowOrder::TopDown => idx as f64 * self.pitch,
                RightRowOrder::BottomUp => last_y - idx as f64 * self.pitch,
            };
            let shape = if number == self.pins_per_row + 1 {
                PadShape::Rect
            } else {
                PadShape::Circle
            };
            footprint.add_pad(Pad::thru_hole(
                number.to_string(),
                shape,
                Point::new(self.row_spacing, y),
                pad_size,
                self.drill,
            ));
        }

        let body = Rect::from_min_max(
            -self.body_margin_x,
            -self.body_margin_y,
            self.row_spacing + self.body_margin_x,
            last_y + self.body_margin_y,
        );
        add_production_outlines(&mut footprint, body, self.courtyard_margin);
        add_reference_texts(&mut footprint, &self.value.unwrap_or(self.name), body);
        add_row_labels(
            &mut footprint,
            &self.left_labels,
            &self.right_labels,
            self.pitch,
            self.row_spacing,
        );

        Ok(GeneratedFootprint::new(
            footprint,
            FootprintMetadata::generated("tht_header_2x"),
        ))
    }
}
