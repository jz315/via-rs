use std::collections::{BTreeMap, BTreeSet};

use crate::error::{Diagnostic, Error, ObjectRef, Result};
use crate::footprint::FootprintPads;
use crate::rules::BoardRules;

use super::{ModuleId, Net, Part};

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
