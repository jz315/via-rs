use std::collections::{BTreeMap, BTreeSet};

use crate::electrical::ElectricalClass;
use crate::symbol::SymbolSpec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinSpec {
    name: String,
    pads: BTreeSet<String>,
    class: Option<ElectricalClass>,
}

pub fn pin(name: impl Into<String>) -> PinSpec {
    PinSpec::new(name)
}

impl PinSpec {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            pads: BTreeSet::new(),
            class: None,
        }
    }

    pub fn pad(mut self, pad: impl Into<String>) -> Self {
        self.pads.insert(pad.into());
        self
    }

    pub fn pads<const N: usize>(mut self, pads: [&str; N]) -> Self {
        self.pads
            .extend(pads.into_iter().map(std::string::ToString::to_string));
        self
    }

    pub fn class(mut self, class: ElectricalClass) -> Self {
        self.class = Some(class);
        self
    }

    pub fn power(self, domain: impl Into<String>) -> Self {
        self.class(ElectricalClass::power(domain))
    }

    pub fn logic(self, domain: impl Into<String>) -> Self {
        self.class(ElectricalClass::logic(domain))
    }

    pub fn passive(self) -> Self {
        self.class(ElectricalClass::Passive)
    }

    pub fn ground(self) -> Self {
        self.class(ElectricalClass::Ground)
    }

    pub fn motor_phase(self) -> Self {
        self.class(ElectricalClass::MotorPhase)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn pad_names(&self) -> impl Iterator<Item = &String> {
        self.pads.iter()
    }

    pub fn electrical_class(&self) -> Option<&ElectricalClass> {
        self.class.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Part {
    pub(crate) refdes: String,
    pub(crate) value: String,
    pub(crate) footprint: Option<String>,
    pub(crate) symbol: Option<SymbolSpec>,
    pub(crate) pins: BTreeSet<String>,
    pub(crate) pin_pads: BTreeMap<String, BTreeSet<String>>,
    pub(crate) pin_classes: BTreeMap<String, ElectricalClass>,
    pub(crate) verify: bool,
    pub(crate) manufacturer_part_number: Option<String>,
    pub(crate) supplier_parts: BTreeMap<String, String>,
    pub(crate) production_notes: Vec<String>,
}

impl Part {
    pub fn new(refdes: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            refdes: refdes.into(),
            value: value.into(),
            footprint: None,
            symbol: None,
            pins: BTreeSet::new(),
            pin_pads: BTreeMap::new(),
            pin_classes: BTreeMap::new(),
            verify: false,
            manufacturer_part_number: None,
            supplier_parts: BTreeMap::new(),
            production_notes: Vec::new(),
        }
    }

    pub fn footprint(mut self, footprint: impl Into<String>) -> Self {
        self.footprint = Some(footprint.into());
        self
    }

    pub fn symbol(mut self, symbol: impl Into<SymbolSpec>) -> Self {
        self.symbol = Some(symbol.into());
        self
    }

    pub fn pins<const N: usize>(mut self, pins: [&str; N]) -> Self {
        self.pins.extend(pins.into_iter().map(str::to_owned));
        self
    }

    pub fn map_pin(mut self, pin: impl Into<String>, pad: impl Into<String>) -> Self {
        self.pin_pads
            .entry(pin.into())
            .or_default()
            .insert(pad.into());
        self
    }

    pub fn map_pin_to_pads<const N: usize>(
        mut self,
        pin: impl Into<String>,
        pads: [&str; N],
    ) -> Self {
        self.pin_pads
            .entry(pin.into())
            .or_default()
            .extend(pads.into_iter().map(str::to_owned));
        self
    }

    pub fn pinmap<const N: usize>(mut self, mappings: [(&str, &str); N]) -> Self {
        for (pin, pad) in mappings {
            self.pin_pads
                .entry(pin.to_owned())
                .or_default()
                .insert(pad.to_owned());
        }
        self
    }

    pub fn pin_class(mut self, pin: impl Into<String>, class: ElectricalClass) -> Self {
        self.pin_classes.insert(pin.into(), class);
        self
    }

    pub fn pin_classes<const N: usize>(mut self, classes: [(&str, ElectricalClass); N]) -> Self {
        for (pin, class) in classes {
            self.pin_classes.insert(pin.to_owned(), class);
        }
        self
    }

    pub fn pin(mut self, pin: PinSpec) -> Self {
        let name = pin.name;
        self.pins.insert(name.clone());

        if !pin.pads.is_empty() {
            self.pin_pads.insert(name.clone(), pin.pads);
        }

        if let Some(class) = pin.class {
            self.pin_classes.insert(name, class);
        }

        self
    }

    pub fn pin_specs<I>(mut self, pins: I) -> Self
    where
        I: IntoIterator<Item = PinSpec>,
    {
        for pin in pins {
            self = self.pin(pin);
        }
        self
    }

    pub fn power_pin(self, pin: impl Into<String>, domain: impl Into<String>) -> Self {
        self.pin_class(pin, ElectricalClass::power(domain))
    }

    pub fn logic_pin(self, pin: impl Into<String>, domain: impl Into<String>) -> Self {
        self.pin_class(pin, ElectricalClass::logic(domain))
    }

    pub fn ground_pin(self, pin: impl Into<String>) -> Self {
        self.pin_class(pin, ElectricalClass::Ground)
    }

    pub fn motor_phase_pin(self, pin: impl Into<String>) -> Self {
        self.pin_class(pin, ElectricalClass::MotorPhase)
    }

    pub fn verify(mut self) -> Self {
        self.verify = true;
        self
    }

    pub fn mpn(mut self, part_number: impl Into<String>) -> Self {
        self.manufacturer_part_number = Some(part_number.into());
        self
    }

    pub fn supplier_part(
        mut self,
        supplier: impl Into<String>,
        part_number: impl Into<String>,
    ) -> Self {
        self.supplier_parts
            .insert(supplier.into(), part_number.into());
        self
    }

    pub fn lcsc(self, part_number: impl Into<String>) -> Self {
        self.supplier_part("LCSC", part_number)
    }

    pub fn production_note(mut self, note: impl Into<String>) -> Self {
        self.production_notes.push(note.into());
        self
    }

    pub fn refdes(&self) -> &str {
        &self.refdes
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn footprint_name(&self) -> Option<&str> {
        self.footprint.as_deref()
    }

    pub fn symbol_spec(&self) -> Option<&SymbolSpec> {
        self.symbol.as_ref()
    }

    pub fn pins_iter(&self) -> impl Iterator<Item = &String> {
        self.pins.iter()
    }

    pub fn pin_pad_mappings(&self) -> impl Iterator<Item = (&String, &BTreeSet<String>)> {
        self.pin_pads.iter()
    }

    pub fn class_for_pin(&self, pin: &str) -> Option<&ElectricalClass> {
        self.pin_classes.get(pin)
    }

    pub fn pads_for_pin(&self, pin: &str) -> BTreeSet<String> {
        self.pin_pads
            .get(pin)
            .cloned()
            .unwrap_or_else(|| BTreeSet::from([pin.to_owned()]))
    }

    pub fn modeled_pads(&self) -> BTreeSet<String> {
        self.pins
            .iter()
            .flat_map(|pin| self.pads_for_pin(pin))
            .collect()
    }

    pub fn requires_verification(&self) -> bool {
        self.verify
    }

    pub fn manufacturer_part_number(&self) -> Option<&str> {
        self.manufacturer_part_number.as_deref()
    }

    pub fn supplier_parts(&self) -> impl Iterator<Item = (&String, &String)> {
        self.supplier_parts.iter()
    }

    pub fn has_production_source(&self) -> bool {
        self.manufacturer_part_number.is_some() || !self.supplier_parts.is_empty()
    }

    pub fn production_notes(&self) -> &[String] {
        &self.production_notes
    }

    pub(crate) fn contains_pin(&self, pin: &str) -> bool {
        self.pins.contains(pin)
    }

    pub(crate) fn mapped_pins(&self) -> impl Iterator<Item = &String> {
        self.pin_pads.keys()
    }

    pub(crate) fn classified_pins(&self) -> impl Iterator<Item = &String> {
        self.pin_classes.keys()
    }
}
