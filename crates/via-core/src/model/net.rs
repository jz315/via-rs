use crate::electrical::ElectricalClass;

use super::PinRef;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Net {
    pub(crate) name: String,
    pub(crate) connections: Vec<PinRef>,
    pub(crate) class: Option<ElectricalClass>,
}

impl Net {
    pub(crate) fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            connections: Vec::new(),
            class: None,
        }
    }

    pub fn connect(&mut self, pin: PinRef) -> &mut Self {
        self.connections.push(pin);
        self
    }

    pub fn connect_all<I>(&mut self, pins: I) -> &mut Self
    where
        I: IntoIterator<Item = PinRef>,
    {
        self.connections.extend(pins);
        self
    }

    pub fn class(&mut self, class: ElectricalClass) -> &mut Self {
        self.class = Some(class);
        self
    }

    pub fn power(&mut self, domain: impl Into<String>) -> &mut Self {
        self.class(ElectricalClass::power(domain))
    }

    pub fn logic(&mut self, domain: impl Into<String>) -> &mut Self {
        self.class(ElectricalClass::logic(domain))
    }

    pub fn ground(&mut self) -> &mut Self {
        self.class(ElectricalClass::Ground)
    }

    pub fn motor_phase(&mut self) -> &mut Self {
        self.class(ElectricalClass::MotorPhase)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn electrical_class(&self) -> Option<&ElectricalClass> {
        self.class.as_ref()
    }

    pub fn connections(&self) -> &[PinRef] {
        &self.connections
    }
}
