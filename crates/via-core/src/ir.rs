use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use via_footprint_ir::FootprintIr;

use crate::electrical::ElectricalClass;
use crate::footprint::FootprintAsset;
use crate::symbol::{SymbolKind, SymbolSide};

pub const BOARD_IR_SCHEMA: &str = "via.board";
pub const BOARD_IR_VERSION: u32 = 2;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoardIr {
    pub schema: String,
    pub version: u32,
    pub board: BoardDataIr,
}

impl BoardIr {
    pub fn new(board: BoardDataIr) -> Self {
        Self {
            schema: BOARD_IR_SCHEMA.to_owned(),
            version: BOARD_IR_VERSION,
            board,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoardDataIr {
    pub name: String,
    pub rules: RulesIr,
    pub modules: Vec<ModuleIr>,
    pub nets: Vec<NetIr>,
    pub footprints: Vec<FootprintPadsIr>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RulesIr {
    pub grid_mm: f64,
    pub default_track_width_mm: f64,
    pub net_class_track_width_mm: BTreeMap<String, f64>,
    pub clearance_mm: f64,
    pub via_drill_mm: f64,
    pub via_diameter_mm: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleIr {
    pub refdes: String,
    pub value: String,
    pub footprint: Option<String>,
    pub symbol: Option<SymbolIr>,
    pub pins: Vec<PinIr>,
    pub requires_verification: bool,
    pub manufacturer_part_number: Option<String>,
    pub supplier_parts: BTreeMap<String, String>,
    pub production_notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolIr {
    pub kind: SymbolKind,
    pub label: Option<String>,
    pub pins: BTreeMap<SymbolSide, Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PinIr {
    pub name: String,
    pub pads: Vec<String>,
    pub class: Option<ElectricalClass>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetIr {
    pub name: String,
    pub class: Option<ElectricalClass>,
    pub connections: Vec<PinRefIr>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PinRefIr {
    pub module: String,
    pub pin: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FootprintPadsIr {
    pub name: String,
    pub pads: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset: Option<FootprintAsset>,
    pub source: Option<PathBuf>,
    pub ir: Option<FootprintIr>,
}
