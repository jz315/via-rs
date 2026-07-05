use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use via_footprint_ir::FootprintIr;

#[derive(Debug, Clone, PartialEq)]
pub struct FootprintPads {
    name: String,
    pads: BTreeSet<String>,
    source: Option<PathBuf>,
    ir: Option<FootprintIr>,
}

impl FootprintPads {
    pub fn new(name: impl Into<String>, pads: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            name: name.into(),
            pads: pads.into_iter().map(Into::into).collect(),
            source: None,
            ir: None,
        }
    }

    pub fn from_ir(ir: FootprintIr) -> Self {
        Self {
            name: ir.name().to_owned(),
            pads: ir.pads().iter().map(|pad| pad.number.clone()).collect(),
            source: None,
            ir: Some(ir),
        }
    }

    pub fn with_source(mut self, source: impl Into<PathBuf>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn with_ir(mut self, ir: FootprintIr) -> Self {
        self.pads = ir.pads().iter().map(|pad| pad.number.clone()).collect();
        self.ir = Some(ir);
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn pads(&self) -> &BTreeSet<String> {
        &self.pads
    }

    pub fn source(&self) -> Option<&Path> {
        self.source.as_deref()
    }

    pub fn ir(&self) -> Option<&FootprintIr> {
        self.ir.as_ref()
    }

    pub fn contains_pad(&self, pad: &str) -> bool {
        self.pads.contains(pad)
    }
}
