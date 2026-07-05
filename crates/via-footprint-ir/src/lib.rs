use std::fs;
use std::io;
use std::path::Path;
use std::{error, fmt};

pub mod kicad;

#[derive(Debug, Clone, PartialEq)]
pub struct FootprintIr {
    name: String,
    description: Option<String>,
    tags: Vec<String>,
    properties: Vec<FootprintProperty>,
    pads: Vec<Pad>,
    lines: Vec<GraphicLine>,
    texts: Vec<GraphicText>,
}

impl FootprintIr {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            tags: Vec::new(),
            properties: Vec::new(),
            pads: Vec::new(),
            lines: Vec::new(),
            texts: Vec::new(),
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub fn property(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.set_property(name, value);
        self
    }

    pub fn set_property(&mut self, name: impl Into<String>, value: impl Into<String>) -> &mut Self {
        let name = name.into();
        let value = value.into();
        if let Some(property) = self
            .properties
            .iter_mut()
            .find(|property| property.name == name)
        {
            property.value = value;
        } else {
            self.properties.push(FootprintProperty { name, value });
        }
        self
    }

    pub fn add_pad(&mut self, pad: Pad) -> &mut Self {
        self.pads.push(pad);
        self
    }

    pub fn add_line(&mut self, line: GraphicLine) -> &mut Self {
        self.lines.push(line);
        self
    }

    pub fn add_text(&mut self, text: GraphicText) -> &mut Self {
        self.texts.push(text);
        self
    }

    pub fn add_rect(
        &mut self,
        start: Point,
        end: Point,
        layer: impl Into<String>,
        width: f64,
    ) -> &mut Self {
        let layer = layer.into();
        self.add_line(GraphicLine::new(
            Point::new(start.x, start.y),
            Point::new(end.x, start.y),
            layer.clone(),
            width,
        ))
        .add_line(GraphicLine::new(
            Point::new(end.x, start.y),
            Point::new(end.x, end.y),
            layer.clone(),
            width,
        ))
        .add_line(GraphicLine::new(
            Point::new(end.x, end.y),
            Point::new(start.x, end.y),
            layer.clone(),
            width,
        ))
        .add_line(GraphicLine::new(
            Point::new(start.x, end.y),
            Point::new(start.x, start.y),
            layer,
            width,
        ))
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn pads(&self) -> &[Pad] {
        &self.pads
    }

    pub fn lines(&self) -> &[GraphicLine] {
        &self.lines
    }

    pub fn texts(&self) -> &[GraphicText] {
        &self.texts
    }

    pub fn description_text(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    pub fn properties(&self) -> &[FootprintProperty] {
        &self.properties
    }

    pub fn write_kicad_mod(&self, path: impl AsRef<Path>) -> Result<(), FootprintWriteError> {
        self.validate()?;
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, kicad::try_render_kicad_mod(self)?)?;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), FootprintValidationError> {
        if self.name.trim().is_empty() {
            return Err(FootprintValidationError::new("footprint name is empty"));
        }

        if self.pads.is_empty() {
            return Err(FootprintValidationError::new(format!(
                "{} has no pads",
                self.name
            )));
        }

        let mut seen = std::collections::BTreeSet::new();
        for pad in &self.pads {
            if pad.number.trim().is_empty() {
                return Err(FootprintValidationError::new(format!(
                    "{} contains a pad with an empty number",
                    self.name
                )));
            }
            if !seen.insert(pad.number.clone()) {
                return Err(FootprintValidationError::new(format!(
                    "{} contains duplicate pad {}",
                    self.name, pad.number
                )));
            }
            if pad.size.x <= 0.0 || pad.size.y <= 0.0 {
                return Err(FootprintValidationError::new(format!(
                    "{} pad {} has non-positive size",
                    self.name, pad.number
                )));
            }
            if pad.kind.requires_drill()
                && !pad.drill.map(|drill| drill.is_positive()).unwrap_or(false)
            {
                return Err(FootprintValidationError::new(format!(
                    "{} pad {} has no positive drill",
                    self.name, pad.number
                )));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FootprintProperty {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pad {
    pub number: String,
    pub kind: PadKind,
    pub shape: PadShape,
    pub at: Point,
    pub size: Size,
    pub drill: Option<PadDrill>,
    pub layers: Vec<String>,
}

impl Pad {
    pub fn thru_hole(
        number: impl Into<String>,
        shape: PadShape,
        at: Point,
        size: Size,
        drill: f64,
    ) -> Self {
        Self {
            number: number.into(),
            kind: PadKind::ThruHole,
            shape,
            at,
            size,
            drill: Some(PadDrill::round(drill)),
            layers: vec!["*.Cu".to_owned(), "*.Mask".to_owned()],
        }
    }

    pub fn thru_hole_slot(
        number: impl Into<String>,
        shape: PadShape,
        at: Point,
        size: Size,
        drill_x: f64,
        drill_y: f64,
    ) -> Self {
        Self {
            number: number.into(),
            kind: PadKind::ThruHole,
            shape,
            at,
            size,
            drill: Some(PadDrill::oval(drill_x, drill_y)),
            layers: vec!["*.Cu".to_owned(), "*.Mask".to_owned()],
        }
    }

    pub fn np_thru_hole(
        number: impl Into<String>,
        shape: PadShape,
        at: Point,
        size: Size,
        drill: f64,
    ) -> Self {
        Self {
            number: number.into(),
            kind: PadKind::NpThruHole,
            shape,
            at,
            size,
            drill: Some(PadDrill::round(drill)),
            layers: vec!["*.Cu".to_owned(), "*.Mask".to_owned()],
        }
    }

    pub fn smd(number: impl Into<String>, shape: PadShape, at: Point, size: Size) -> Self {
        Self {
            number: number.into(),
            kind: PadKind::Smd,
            shape,
            at,
            size,
            drill: None,
            layers: vec!["F.Cu".to_owned(), "F.Paste".to_owned(), "F.Mask".to_owned()],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PadKind {
    ThruHole,
    NpThruHole,
    Smd,
}

impl PadKind {
    pub fn requires_drill(self) -> bool {
        matches!(self, Self::ThruHole | Self::NpThruHole)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PadDrill {
    pub x: f64,
    pub y: f64,
}

impl PadDrill {
    pub fn round(diameter: f64) -> Self {
        Self {
            x: diameter,
            y: diameter,
        }
    }

    pub fn oval(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn is_round(self) -> bool {
        (self.x - self.y).abs() < 0.001
    }

    pub fn is_positive(self) -> bool {
        self.x > 0.0 && self.y > 0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PadShape {
    Circle,
    Oval,
    Rect,
    RoundRect,
    Trapezoid,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphicText {
    pub kind: TextKind,
    pub text: String,
    pub at: Point,
    pub rotation: f64,
    pub layer: String,
    pub size: Size,
    pub thickness: f64,
    pub justify: Option<TextJustify>,
}

impl GraphicText {
    pub fn reference(text: impl Into<String>, at: Point, layer: impl Into<String>) -> Self {
        Self::new(TextKind::Reference, text, at, layer)
    }

    pub fn value(text: impl Into<String>, at: Point, layer: impl Into<String>) -> Self {
        Self::new(TextKind::Value, text, at, layer)
    }

    pub fn user(text: impl Into<String>, at: Point, layer: impl Into<String>) -> Self {
        Self::new(TextKind::User, text, at, layer)
    }

    fn new(kind: TextKind, text: impl Into<String>, at: Point, layer: impl Into<String>) -> Self {
        Self {
            kind,
            text: text.into(),
            at,
            rotation: 0.0,
            layer: layer.into(),
            size: Size::new(0.75, 0.75),
            thickness: 0.1,
            justify: None,
        }
    }

    pub fn rotation(mut self, rotation: f64) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn size(mut self, x: f64, y: f64) -> Self {
        self.size = Size::new(x, y);
        self
    }

    pub fn thickness(mut self, thickness: f64) -> Self {
        self.thickness = thickness;
        self
    }

    pub fn justify(mut self, justify: TextJustify) -> Self {
        self.justify = Some(justify);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextKind {
    Reference,
    Value,
    User,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextJustify {
    Left,
    Right,
    Center,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphicLine {
    pub start: Point,
    pub end: Point,
    pub layer: String,
    pub width: f64,
}

impl GraphicLine {
    pub fn new(start: Point, end: Point, layer: impl Into<String>, width: f64) -> Self {
        Self {
            start,
            end,
            layer: layer.into(),
            width,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub x: f64,
    pub y: f64,
}

impl Size {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FootprintValidationError {
    message: String,
}

impl FootprintValidationError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for FootprintValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl error::Error for FootprintValidationError {}

#[derive(Debug)]
pub enum FootprintWriteError {
    Validation(FootprintValidationError),
    Io(io::Error),
}

impl fmt::Display for FootprintWriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FootprintWriteError::Validation(err) => write!(f, "{err}"),
            FootprintWriteError::Io(err) => write!(f, "{err}"),
        }
    }
}

impl error::Error for FootprintWriteError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            FootprintWriteError::Validation(err) => Some(err),
            FootprintWriteError::Io(err) => Some(err),
        }
    }
}

impl From<FootprintValidationError> for FootprintWriteError {
    fn from(value: FootprintValidationError) -> Self {
        FootprintWriteError::Validation(value)
    }
}

impl From<io::Error> for FootprintWriteError {
    fn from(value: io::Error) -> Self {
        FootprintWriteError::Io(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_duplicate_pads() {
        let mut footprint = FootprintIr::new("Bad");
        footprint.add_pad(Pad::thru_hole(
            "1",
            PadShape::Rect,
            Point::new(0.0, 0.0),
            Size::new(1.8, 1.8),
            1.0,
        ));
        footprint.add_pad(Pad::thru_hole(
            "1",
            PadShape::Circle,
            Point::new(2.54, 0.0),
            Size::new(1.8, 1.8),
            1.0,
        ));

        let err = footprint.validate().unwrap_err();
        assert!(err.to_string().contains("duplicate pad 1"));
    }

    #[test]
    fn renders_reference_value_and_user_text() {
        let mut footprint = FootprintIr::new("TextDemo");
        footprint.add_pad(Pad::thru_hole(
            "1",
            PadShape::Rect,
            Point::new(0.0, 0.0),
            Size::new(1.8, 1.8),
            1.0,
        ));
        footprint
            .add_text(GraphicText::reference(
                "REF**",
                Point::new(0.0, -2.0),
                "F.SilkS",
            ))
            .add_text(GraphicText::value(
                "TextDemo",
                Point::new(0.0, 2.0),
                "F.Fab",
            ))
            .add_text(GraphicText::user("pin 1", Point::new(0.0, 0.0), "F.Fab"));

        let text = kicad::try_render_kicad_mod(&footprint).unwrap();
        assert!(text.contains("(fp_text reference \"REF**\""));
        assert!(text.contains("(fp_text value \"TextDemo\""));
        assert!(text.contains("(fp_text user \"pin 1\""));
    }

    #[test]
    fn renders_roundrect_pad_ratio() {
        let mut footprint = FootprintIr::new("RoundRectDemo");
        footprint.add_pad(Pad::smd(
            "1",
            PadShape::RoundRect,
            Point::new(0.0, 0.0),
            Size::new(1.2, 1.6),
        ));

        let text = kicad::try_render_kicad_mod(&footprint).unwrap();
        assert!(text.contains("roundrect"));
        assert!(text.contains("(roundrect_rratio 0.25)"));
    }

    #[test]
    fn renders_trapezoid_pad_shape() {
        let mut footprint = FootprintIr::new("TrapezoidDemo");
        footprint.add_pad(Pad::smd(
            "1",
            PadShape::Trapezoid,
            Point::new(0.0, 0.0),
            Size::new(1.2, 1.6),
        ));

        let text = kicad::try_render_kicad_mod(&footprint).unwrap();
        assert!(text.contains(" trapezoid "));
    }

    #[test]
    fn renders_slot_and_np_thru_hole_drills() {
        let mut footprint = FootprintIr::new("SlotDemo");
        footprint.add_pad(Pad::thru_hole_slot(
            "1",
            PadShape::Oval,
            Point::new(0.0, 0.0),
            Size::new(2.0, 4.0),
            1.0,
            3.2,
        ));
        footprint.add_pad(Pad::np_thru_hole(
            "MH1",
            PadShape::Circle,
            Point::new(5.0, 0.0),
            Size::new(3.2, 3.2),
            3.2,
        ));

        let text = kicad::try_render_kicad_mod(&footprint).unwrap();
        assert!(text.contains("(drill oval 1 3.2)"));
        assert!(text.contains("(pad \"MH1\" np_thru_hole circle"));
    }

    #[test]
    fn try_render_returns_validation_errors() {
        let mut footprint = FootprintIr::new("Bad");
        footprint.add_pad(Pad::thru_hole(
            "1",
            PadShape::Rect,
            Point::new(0.0, 0.0),
            Size::new(1.8, 1.8),
            1.0,
        ));
        footprint.add_pad(Pad::thru_hole(
            "1",
            PadShape::Circle,
            Point::new(2.54, 0.0),
            Size::new(1.8, 1.8),
            1.0,
        ));

        let err = kicad::try_render_kicad_mod(&footprint).unwrap_err();
        assert!(err.to_string().contains("duplicate pad 1"));
    }
}
