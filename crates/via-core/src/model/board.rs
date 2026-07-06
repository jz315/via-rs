use std::collections::{BTreeMap, BTreeSet};

use crate::error::{Diagnostic, Error, ObjectRef, Result};
use crate::footprint::FootprintPads;
use crate::ir::{
    BOARD_IR_SCHEMA, BOARD_IR_VERSION, BoardDataIr, BoardIr, FootprintPadsIr, ModuleIr, NetIr,
    PinIr, PinRefIr, RulesIr, SymbolIr,
};
use crate::rules::BoardRules;
use crate::symbol::{SymbolSide, SymbolSpec};

use super::{ModuleId, Net, Part, PinRef};

#[derive(Debug, Clone, PartialEq)]
pub struct Board {
    name: String,
    modules: BTreeMap<String, Part>,
    nets: BTreeMap<String, Net>,
    footprints: BTreeMap<String, FootprintPads>,
    rules: BoardRules,
}

impl Board {
    pub(crate) fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            modules: BTreeMap::new(),
            nets: BTreeMap::new(),
            footprints: BTreeMap::new(),
            rules: BoardRules::default(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn modules(&self) -> impl Iterator<Item = &Part> {
        self.modules.values()
    }

    pub fn module(&self, refdes: &str) -> Option<&Part> {
        self.modules.get(refdes)
    }

    pub fn nets(&self) -> impl Iterator<Item = &Net> {
        self.nets.values()
    }

    pub fn footprints(&self) -> impl Iterator<Item = &FootprintPads> {
        self.footprints.values()
    }

    pub fn rules(&self) -> &BoardRules {
        &self.rules
    }

    pub fn rules_mut(&mut self) -> &mut BoardRules {
        &mut self.rules
    }

    pub fn to_ir(&self) -> BoardIr {
        BoardIr::new(BoardDataIr {
            name: self.name.clone(),
            rules: RulesIr {
                grid_mm: self.rules.grid_mm(),
                default_track_width_mm: self.rules.default_track_width_mm(),
                net_class_track_width_mm: self
                    .rules
                    .net_class_track_widths_mm()
                    .map(|(class, width)| (class.clone(), *width))
                    .collect(),
                clearance_mm: self.rules.clearance_mm(),
                via_drill_mm: self.rules.via_drill_mm(),
                via_diameter_mm: self.rules.via_diameter_mm(),
            },
            modules: self
                .modules
                .values()
                .map(|module| ModuleIr {
                    refdes: module.refdes().to_owned(),
                    value: module.value().to_owned(),
                    footprint: module.footprint_name().map(str::to_owned),
                    symbol: module.symbol_spec().map(symbol_to_ir),
                    pins: module
                        .pins_iter()
                        .map(|pin| PinIr {
                            name: pin.clone(),
                            pads: module.pads_for_pin(pin).into_iter().collect(),
                            class: module.class_for_pin(pin).cloned(),
                        })
                        .collect(),
                    requires_verification: module.requires_verification(),
                    manufacturer_part_number: module.manufacturer_part_number().map(str::to_owned),
                    supplier_parts: module
                        .supplier_parts()
                        .map(|(supplier, part)| (supplier.clone(), part.clone()))
                        .collect(),
                    production_notes: module.production_notes().to_vec(),
                })
                .collect(),
            nets: self
                .nets
                .values()
                .map(|net| NetIr {
                    name: net.name().to_owned(),
                    class: net.electrical_class().cloned(),
                    connections: net
                        .connections()
                        .iter()
                        .map(|pin| PinRefIr {
                            module: pin.module.clone(),
                            pin: pin.pin.clone(),
                        })
                        .collect(),
                })
                .collect(),
            footprints: self
                .footprints
                .values()
                .map(|footprint| FootprintPadsIr {
                    name: footprint.name().to_owned(),
                    pads: footprint.pads().iter().cloned().collect(),
                    asset: footprint.asset().cloned(),
                    source: footprint.source().map(std::path::Path::to_path_buf),
                    ir: footprint.ir().cloned(),
                })
                .collect(),
        })
    }

    pub fn from_ir(ir: BoardIr) -> Result<Self> {
        if ir.schema != BOARD_IR_SCHEMA {
            return Err(invalid_ir(format!(
                "unsupported schema {}; expected {}",
                ir.schema, BOARD_IR_SCHEMA
            )));
        }

        if !matches!(ir.version, 1 | BOARD_IR_VERSION) {
            return Err(invalid_ir(format!(
                "unsupported version {}; expected 1 or {}",
                ir.version, BOARD_IR_VERSION
            )));
        }

        let mut board = Self::new(ir.board.name);
        board.rules = rules_from_ir(ir.board.rules);

        for module in ir.board.modules {
            if board.modules.contains_key(&module.refdes) {
                return Err(Error::DuplicateModule(module.refdes));
            }
            let part = part_from_ir(module);
            board.modules.insert(part.refdes.clone(), part);
        }

        for net in ir.board.nets {
            if board.nets.contains_key(&net.name) {
                return Err(invalid_ir(format!("duplicate net {}", net.name)));
            }
            board.nets.insert(net.name.clone(), net_from_ir(net));
        }

        for footprint in ir.board.footprints {
            if board.footprints.contains_key(&footprint.name) {
                return Err(invalid_ir(format!(
                    "duplicate footprint {}",
                    footprint.name
                )));
            }
            board
                .footprints
                .insert(footprint.name.clone(), footprint_from_ir(footprint));
        }

        Ok(board)
    }

    pub(crate) fn add_module(&mut self, part: Part) -> Result<ModuleId> {
        let refdes = part.refdes().to_owned();
        if self.modules.contains_key(&refdes) {
            return Err(Error::DuplicateModule(refdes));
        }

        self.modules.insert(refdes.clone(), part);
        Ok(ModuleId::new(refdes))
    }

    pub(crate) fn net(&mut self, name: impl Into<String>) -> &mut Net {
        let name = name.into();
        self.nets
            .entry(name.clone())
            .or_insert_with(|| Net::new(name))
    }

    pub(crate) fn add_footprint_pads(&mut self, footprint: FootprintPads) {
        self.footprints
            .insert(footprint.name().to_owned(), footprint);
    }

    pub fn check(&self) -> Result<()> {
        let diagnostics = self.diagnostics();
        if diagnostics.is_empty() {
            Ok(())
        } else {
            Err(Error::Validation(diagnostics))
        }
    }

    pub fn check_production(&self) -> Result<()> {
        let diagnostics = self.production_diagnostics();
        if diagnostics.is_empty() {
            Ok(())
        } else {
            Err(Error::Validation(diagnostics))
        }
    }

    pub fn diagnostics(&self) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        self.check_footprints(&mut diagnostics);
        self.check_part_metadata(&mut diagnostics);
        self.check_net_connections(&mut diagnostics);
        self.check_physical_pad_shorts(&mut diagnostics);

        diagnostics
    }

    pub fn production_diagnostics(&self) -> Vec<Diagnostic> {
        let mut diagnostics = self.diagnostics();
        self.check_production_readiness(&mut diagnostics);
        diagnostics
    }

    fn check_footprints(&self, diagnostics: &mut Vec<Diagnostic>) {
        for module in self.modules.values() {
            let Some(footprint_name) = module.footprint_name() else {
                diagnostics.push(
                    Diagnostic::coded(
                        "part.no_footprint",
                        format!("{} has no footprint", module.refdes()),
                    )
                    .at(ObjectRef::module(module.refdes())),
                );
                continue;
            };

            if let Some(footprint) = self.footprints.get(footprint_name) {
                if let Some(crate::footprint::FootprintAsset::KicadLibrary { name, .. }) =
                    footprint.asset()
                    && name != footprint.name()
                {
                    diagnostics.push(
                        Diagnostic::coded(
                            "footprint.asset_alias",
                            format!(
                                "footprint {} points to KiCad library footprint {}; aliasing is not supported",
                                footprint.name(),
                                name
                            ),
                        )
                        .at(ObjectRef::footprint(footprint.name())),
                    );
                }

                for pin in module.pins_iter() {
                    for pad in module.pads_for_pin(pin) {
                        if !footprint.contains_pad(&pad) {
                            diagnostics.push(
                                Diagnostic::coded(
                                    "pin_pad_map.missing_pad",
                                    format!(
                                        "{} pin {} maps to missing pad {} on footprint {}",
                                        module.refdes(),
                                        pin,
                                        pad,
                                        footprint_name
                                    ),
                                )
                                .at(ObjectRef::pin(module.refdes(), pin))
                                .relates_to(ObjectRef::footprint(footprint_name)),
                            );
                        }
                    }
                }

                let modeled_pads = module.modeled_pads();
                let missing_pads = footprint
                    .pads()
                    .difference(&modeled_pads)
                    .cloned()
                    .collect::<Vec<_>>();
                if !missing_pads.is_empty() {
                    diagnostics.push(
                        Diagnostic::coded(
                            "pin_pad_map.uncovered_footprint_pad",
                            format!(
                                "{} model does not cover pads on footprint {}: {}",
                                module.refdes(),
                                footprint_name,
                                missing_pads.join(", ")
                            ),
                        )
                        .at(ObjectRef::footprint(footprint_name))
                        .relates_to(ObjectRef::module(module.refdes())),
                    );
                }
            } else if !self.footprints.is_empty() {
                diagnostics.push(
                    Diagnostic::coded(
                        "part.unknown_footprint",
                        format!(
                            "{} references footprint {} but no footprint pads or asset were loaded",
                            module.refdes(),
                            footprint_name
                        ),
                    )
                    .at(ObjectRef::module(module.refdes()))
                    .relates_to(ObjectRef::footprint(footprint_name)),
                );
            }
        }
    }

    fn check_part_metadata(&self, diagnostics: &mut Vec<Diagnostic>) {
        for module in self.modules.values() {
            for mapped_pin in module.mapped_pins() {
                if !module.contains_pin(mapped_pin) {
                    diagnostics.push(
                        Diagnostic::coded(
                            "part.unknown_mapped_pin",
                            format!(
                                "{} maps unknown logical pin {}",
                                module.refdes(),
                                mapped_pin
                            ),
                        )
                        .at(ObjectRef::pin(module.refdes(), mapped_pin)),
                    );
                }
            }

            for classified_pin in module.classified_pins() {
                if !module.contains_pin(classified_pin) {
                    diagnostics.push(
                        Diagnostic::coded(
                            "part.unknown_classified_pin",
                            format!(
                                "{} classifies unknown logical pin {}",
                                module.refdes(),
                                classified_pin
                            ),
                        )
                        .at(ObjectRef::pin(module.refdes(), classified_pin)),
                    );
                }
            }

            if let Some(symbol) = module.symbol_spec() {
                let mut seen_symbol_pins = BTreeSet::new();
                for (pin, _side) in symbol.pin_sides() {
                    if !module.contains_pin(pin) {
                        diagnostics.push(
                            Diagnostic::coded(
                                "symbol.unknown_pin",
                                format!(
                                    "{} symbol references unknown logical pin {}",
                                    module.refdes(),
                                    pin
                                ),
                            )
                            .at(ObjectRef::pin(module.refdes(), pin))
                            .relates_to(ObjectRef::module(module.refdes())),
                        );
                    }

                    if !seen_symbol_pins.insert(pin.clone()) {
                        diagnostics.push(
                            Diagnostic::coded(
                                "symbol.duplicate_pin",
                                format!(
                                    "{} symbol places logical pin {} more than once",
                                    module.refdes(),
                                    pin
                                ),
                            )
                            .at(ObjectRef::pin(module.refdes(), pin))
                            .relates_to(ObjectRef::module(module.refdes())),
                        );
                    }
                }
            }
        }
    }

    fn check_net_connections(&self, diagnostics: &mut Vec<Diagnostic>) {
        for net in self.nets.values() {
            if net.connections().len() < 2 {
                diagnostics.push(
                    Diagnostic::coded(
                        "net.too_few_connections",
                        format!("net {} has fewer than 2 connections", net.name()),
                    )
                    .at(ObjectRef::net(net.name())),
                );
            }

            for pin_ref in net.connections() {
                match self.modules.get(&pin_ref.module) {
                    Some(module) if !module.contains_pin(&pin_ref.pin) => {
                        diagnostics.push(
                            Diagnostic::coded(
                                "net.unknown_pin",
                                format!(
                                    "net {} references unknown pin {} on {}",
                                    net.name(),
                                    pin_ref.pin,
                                    pin_ref.module
                                ),
                            )
                            .at(ObjectRef::pin(&pin_ref.module, &pin_ref.pin))
                            .relates_to(ObjectRef::net(net.name())),
                        );
                    }
                    Some(_) => {}
                    None => diagnostics.push(
                        Diagnostic::coded(
                            "net.unknown_module",
                            format!(
                                "net {} references unknown module {}",
                                net.name(),
                                pin_ref.module
                            ),
                        )
                        .at(ObjectRef::module(&pin_ref.module))
                        .relates_to(ObjectRef::net(net.name())),
                    ),
                }
            }

            if let Some(net_class) = net.electrical_class() {
                for pin_ref in net.connections() {
                    let Some(module) = self.modules.get(&pin_ref.module) else {
                        continue;
                    };
                    let Some(pin_class) = module.class_for_pin(&pin_ref.pin) else {
                        continue;
                    };

                    if !net_class.is_compatible_with(pin_class) {
                        diagnostics.push(
                            Diagnostic::coded(
                                "net.electrical_class_mismatch",
                                format!(
                                    "net {} is {} but connects to {}.{} classified as {}",
                                    net.name(),
                                    net_class,
                                    pin_ref.module,
                                    pin_ref.pin,
                                    pin_class
                                ),
                            )
                            .at(ObjectRef::pin(&pin_ref.module, &pin_ref.pin))
                            .relates_to(ObjectRef::net(net.name())),
                        );
                    }
                }
            }
        }
    }

    fn check_physical_pad_shorts(&self, diagnostics: &mut Vec<Diagnostic>) {
        let mut physical_pad_nets: BTreeMap<(String, String), BTreeSet<String>> = BTreeMap::new();
        for net in self.nets.values() {
            for pin_ref in net.connections() {
                let Some(module) = self.modules.get(&pin_ref.module) else {
                    continue;
                };
                if !module.contains_pin(&pin_ref.pin) {
                    continue;
                }

                for pad in module.pads_for_pin(&pin_ref.pin) {
                    physical_pad_nets
                        .entry((pin_ref.module.clone(), pad))
                        .or_default()
                        .insert(net.name().to_owned());
                }
            }
        }

        for ((module, pad), nets) in physical_pad_nets {
            if nets.len() > 1 {
                diagnostics.push(
                    Diagnostic::coded(
                        "net.physical_pad_short",
                        format!(
                            "physical pad {module}.{pad} is connected to multiple nets: {}",
                            nets.into_iter().collect::<Vec<_>>().join(", ")
                        ),
                    )
                    .at(ObjectRef::pad(module, pad)),
                );
            }
        }
    }

    fn check_production_readiness(&self, diagnostics: &mut Vec<Diagnostic>) {
        for module in self.modules.values() {
            if module.requires_verification() {
                let footprint = module.footprint_name().unwrap_or("unassigned footprint");
                diagnostics.push(Diagnostic::coded(
                    "production.unverified_footprint",
                    format!(
                        "{} footprint {} still requires physical verification before production",
                        module.refdes(),
                        footprint
                    ),
                )
                .at(ObjectRef::module(module.refdes()))
                .relates_to(ObjectRef::footprint(footprint)));
            }

            if !module.has_production_source() {
                diagnostics.push(
                    Diagnostic::coded(
                        "production.missing_source",
                        format!(
                            "{} has no production source; add an MPN or supplier part number",
                            module.refdes()
                        ),
                    )
                    .at(ObjectRef::module(module.refdes())),
                );
            }
        }
    }
}

fn symbol_to_ir(symbol: &SymbolSpec) -> SymbolIr {
    SymbolIr {
        kind: symbol.kind(),
        label: symbol.label_text().map(str::to_owned),
        pins: [
            SymbolSide::Left,
            SymbolSide::Right,
            SymbolSide::Top,
            SymbolSide::Bottom,
        ]
        .into_iter()
        .filter_map(|side| {
            let pins = symbol.pins_on(side);
            (!pins.is_empty()).then(|| (side, pins.to_vec()))
        })
        .collect(),
    }
}

fn rules_from_ir(ir: RulesIr) -> BoardRules {
    let mut rules = BoardRules::new();
    rules
        .set_grid_mm(ir.grid_mm)
        .set_default_track_width_mm(ir.default_track_width_mm)
        .set_net_class_track_widths_mm(ir.net_class_track_width_mm)
        .set_clearance_mm(ir.clearance_mm)
        .set_via(ir.via_diameter_mm, ir.via_drill_mm);
    rules
}

fn part_from_ir(ir: ModuleIr) -> Part {
    let mut part = Part::new(ir.refdes, ir.value);
    part.footprint = ir.footprint;
    part.symbol = ir.symbol.map(symbol_from_ir);
    part.verify = ir.requires_verification;
    part.manufacturer_part_number = ir.manufacturer_part_number;
    part.supplier_parts = ir.supplier_parts;
    part.production_notes = ir.production_notes;

    for pin in ir.pins {
        part.pins.insert(pin.name.clone());
        if !pin.pads.is_empty() {
            part.pin_pads
                .insert(pin.name.clone(), pin.pads.into_iter().collect());
        }
        if let Some(class) = pin.class {
            part.pin_classes.insert(pin.name, class);
        }
    }

    part
}

fn symbol_from_ir(ir: SymbolIr) -> SymbolSpec {
    let mut symbol = SymbolSpec::new(ir.kind);
    if let Some(label) = ir.label {
        symbol = symbol.label(label);
    }
    for (side, pins) in ir.pins {
        symbol = symbol.side_pins(side, pins);
    }
    symbol
}

fn net_from_ir(ir: NetIr) -> Net {
    let mut net = Net::new(ir.name);
    if let Some(class) = ir.class {
        net.class(class);
    }
    net.connect_all(ir.connections.into_iter().map(|pin| PinRef {
        module: pin.module,
        pin: pin.pin,
    }));
    net
}

fn footprint_from_ir(ir: FootprintPadsIr) -> FootprintPads {
    let mut footprint = FootprintPads::new(ir.name, ir.pads);
    if let Some(asset) = ir.asset {
        footprint = footprint.with_asset(asset);
    }
    if let Some(source) = ir.source {
        footprint = footprint.with_source(source);
    }
    if let Some(geometry) = ir.ir {
        footprint = footprint.with_ir(geometry);
    }
    footprint
}

fn invalid_ir(message: impl Into<String>) -> Error {
    Error::Io(format!("invalid BoardIr: {}", message.into()))
}
