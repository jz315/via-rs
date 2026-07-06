use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use via_footprint_ir::FootprintIr;

#[derive(Debug, Clone, PartialEq)]
pub struct FootprintPads {
    name: String,
    pads: BTreeSet<String>,
    source: Option<PathBuf>,
    ir: Option<FootprintIr>,
    asset: Option<FootprintAsset>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Footprint {
    Name(String),
    Pads(FootprintPads),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum FootprintAsset {
    KicadLibrary { library: String, name: String },
}

impl Footprint {
    pub fn name(name: impl Into<String>) -> Self {
        Self::Name(name.into())
    }

    pub fn pads(pads: FootprintPads) -> Self {
        Self::Pads(pads)
    }
}

impl From<&str> for Footprint {
    fn from(name: &str) -> Self {
        Self::name(name)
    }
}

impl From<String> for Footprint {
    fn from(name: String) -> Self {
        Self::name(name)
    }
}

impl From<FootprintPads> for Footprint {
    fn from(pads: FootprintPads) -> Self {
        Self::pads(pads)
    }
}

impl FootprintPads {
    pub fn new(name: impl Into<String>, pads: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            name: name.into(),
            pads: pads.into_iter().map(Into::into).collect(),
            source: None,
            ir: None,
            asset: None,
        }
    }

    pub fn from_ir(ir: FootprintIr) -> Self {
        Self {
            name: ir.name().to_owned(),
            pads: ir.pads().iter().map(|pad| pad.number.clone()).collect(),
            source: None,
            ir: Some(ir),
            asset: None,
        }
    }

    pub fn kicad_library(
        library: impl Into<String>,
        name: impl Into<String>,
        pads: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        let library = library.into();
        let name = name.into();
        Self::new(name.clone(), pads).with_kicad_library(library, name)
    }

    pub fn with_source(mut self, source: impl Into<PathBuf>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn with_asset(mut self, asset: FootprintAsset) -> Self {
        self.asset = Some(asset);
        self
    }

    pub fn with_kicad_library(self, library: impl Into<String>, name: impl Into<String>) -> Self {
        self.with_asset(FootprintAsset::KicadLibrary {
            library: library.into(),
            name: name.into(),
        })
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

    pub fn asset(&self) -> Option<&FootprintAsset> {
        self.asset.as_ref()
    }

    pub fn contains_pad(&self, pad: &str) -> bool {
        self.pads.contains(pad)
    }
}
