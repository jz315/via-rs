use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SymbolKind {
    Generic,
    Ic,
    Module,
    Connector,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SymbolSide {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolSpec {
    kind: SymbolKind,
    label: Option<String>,
    pins: BTreeMap<SymbolSide, Vec<String>>,
}

pub mod sym {
    use super::{SymbolKind, SymbolSpec};

    pub fn generic() -> SymbolSpec {
        SymbolSpec::new(SymbolKind::Generic)
    }

    pub fn ic() -> SymbolSpec {
        SymbolSpec::new(SymbolKind::Ic)
    }

    pub fn module() -> SymbolSpec {
        SymbolSpec::new(SymbolKind::Module)
    }

    pub fn connector() -> SymbolSpec {
        SymbolSpec::new(SymbolKind::Connector)
    }
}

impl SymbolSpec {
    pub fn new(kind: SymbolKind) -> Self {
        Self {
            kind,
            label: None,
            pins: BTreeMap::new(),
        }
    }

    pub fn kind(&self) -> SymbolKind {
        self.kind
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn label_text(&self) -> Option<&str> {
        self.label.as_deref()
    }

    pub fn left<const N: usize>(self, pins: [&str; N]) -> Self {
        self.side(SymbolSide::Left, pins)
    }

    pub fn right<const N: usize>(self, pins: [&str; N]) -> Self {
        self.side(SymbolSide::Right, pins)
    }

    pub fn top<const N: usize>(self, pins: [&str; N]) -> Self {
        self.side(SymbolSide::Top, pins)
    }

    pub fn bottom<const N: usize>(self, pins: [&str; N]) -> Self {
        self.side(SymbolSide::Bottom, pins)
    }

    pub fn side<const N: usize>(mut self, side: SymbolSide, pins: [&str; N]) -> Self {
        self = self.side_pins(side, pins);
        self
    }

    pub fn side_pins<I, S>(mut self, side: SymbolSide, pins: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.pins
            .entry(side)
            .or_default()
            .extend(pins.into_iter().map(Into::into));
        self
    }

    pub fn pins_on(&self, side: SymbolSide) -> &[String] {
        self.pins.get(&side).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn pin_sides(&self) -> impl Iterator<Item = (&String, SymbolSide)> {
        self.pins
            .iter()
            .flat_map(|(side, pins)| pins.iter().map(move |pin| (pin, *side)))
    }
}
