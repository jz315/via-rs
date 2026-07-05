use via_footprint_ir::{FootprintIr, GraphicText, Point, TextJustify};

pub(super) const SILK_WIDTH: f64 = 0.12;
pub(super) const FAB_WIDTH: f64 = 0.1;
pub(super) const COURTYARD_WIDTH: f64 = 0.05;

#[derive(Debug, Clone, Copy)]
pub(super) struct Rect {
    pub(super) min_x: f64,
    pub(super) min_y: f64,
    pub(super) max_x: f64,
    pub(super) max_y: f64,
}

impl Rect {
    pub(super) fn from_min_max(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    fn expanded(self, amount: f64) -> Self {
        Self {
            min_x: self.min_x - amount,
            min_y: self.min_y - amount,
            max_x: self.max_x + amount,
            max_y: self.max_y + amount,
        }
    }

    fn center_x(self) -> f64 {
        (self.min_x + self.max_x) / 2.0
    }
}

pub(super) fn add_production_outlines(
    footprint: &mut FootprintIr,
    body: Rect,
    courtyard_margin: f64,
) {
    footprint
        .add_rect(
            Point::new(body.min_x, body.min_y),
            Point::new(body.max_x, body.max_y),
            "F.SilkS",
            SILK_WIDTH,
        )
        .add_rect(
            Point::new(body.min_x, body.min_y),
            Point::new(body.max_x, body.max_y),
            "F.Fab",
            FAB_WIDTH,
        );

    let courtyard = body.expanded(courtyard_margin);
    footprint.add_rect(
        Point::new(courtyard.min_x, courtyard.min_y),
        Point::new(courtyard.max_x, courtyard.max_y),
        "F.CrtYd",
        COURTYARD_WIDTH,
    );
}

pub(super) fn add_reference_texts(footprint: &mut FootprintIr, value: &str, body: Rect) {
    footprint
        .add_text(
            GraphicText::reference(
                "REF**",
                Point::new(body.center_x(), body.min_y - 1.7),
                "F.SilkS",
            )
            .size(1.0, 1.0)
            .thickness(0.15),
        )
        .add_text(
            GraphicText::value(
                value,
                Point::new(body.center_x(), body.max_y + 1.7),
                "F.Fab",
            )
            .size(1.0, 1.0)
            .thickness(0.15),
        );
}

pub(super) fn add_row_labels(
    footprint: &mut FootprintIr,
    left_labels: &[String],
    right_labels: &[String],
    pitch: f64,
    row_spacing: f64,
) {
    for (idx, label) in left_labels.iter().enumerate() {
        footprint.add_text(
            GraphicText::user(label, Point::new(-3.0, idx as f64 * pitch), "F.Fab")
                .size(0.62, 0.62)
                .thickness(0.08)
                .justify(TextJustify::Right),
        );
    }

    for (idx, label) in right_labels.iter().enumerate() {
        footprint.add_text(
            GraphicText::user(
                label,
                Point::new(row_spacing + 3.0, idx as f64 * pitch),
                "F.Fab",
            )
            .size(0.62, 0.62)
            .thickness(0.08)
            .justify(TextJustify::Left),
        );
    }
}
