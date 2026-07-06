use std::fmt;

use via_core::{Component, DecouplerPins, Design, Footprint, ModuleId, PinRef, Result, part, pin};
use via_footprint::{GeneratedFootprint, fp};

#[derive(Debug, Clone)]
pub struct Resistor2 {
    id: ModuleId,
}

#[derive(Debug, Clone)]
pub struct Capacitor2 {
    id: ModuleId,
}

impl Resistor2 {
    pub fn id(&self) -> &ModuleId {
        &self.id
    }

    pub fn pin1(&self) -> PinRef {
        self.id.pin("1")
    }

    pub fn pin2(&self) -> PinRef {
        self.id.pin("2")
    }
}

impl Capacitor2 {
    pub fn id(&self) -> &ModuleId {
        &self.id
    }

    pub fn pin1(&self) -> PinRef {
        self.id.pin("1")
    }

    pub fn pin2(&self) -> PinRef {
        self.id.pin("2")
    }

    pub fn positive(&self) -> PinRef {
        self.pin1()
    }

    pub fn negative(&self) -> PinRef {
        self.pin2()
    }
}

impl DecouplerPins for &Capacitor2 {
    fn into_decoupler_pins(self) -> (PinRef, PinRef) {
        (self.positive(), self.negative())
    }
}

#[derive(Debug, Clone)]
pub struct ResistorBuilder {
    refdes: String,
    value: String,
    footprint: Footprint,
    verify: bool,
}

#[derive(Debug, Clone)]
pub struct CapacitorBuilder {
    refdes: String,
    capacitance: String,
    voltage: Option<String>,
    footprint: Footprint,
    polarized: bool,
    verify: bool,
}

pub fn resistor(refdes: impl Into<String>) -> ResistorBuilder {
    ResistorBuilder {
        refdes: refdes.into(),
        value: "resistor".to_owned(),
        footprint: fp::r0805().into(),
        verify: false,
    }
}

pub fn capacitor(refdes: impl Into<String>) -> CapacitorBuilder {
    CapacitorBuilder {
        refdes: refdes.into(),
        capacitance: "capacitor".to_owned(),
        voltage: None,
        footprint: fp::c0805().into(),
        polarized: false,
        verify: false,
    }
}

impl ResistorBuilder {
    pub fn value(mut self, value: impl fmt::Display) -> Self {
        self.value = value.to_string();
        self
    }

    pub fn fp(mut self, footprint: impl Into<Footprint>) -> Self {
        self.footprint = footprint.into();
        self
    }

    pub fn verify(mut self) -> Self {
        self.verify = true;
        self
    }
}

impl CapacitorBuilder {
    pub fn value(mut self, value: impl fmt::Display) -> Self {
        self.capacitance = value.to_string();
        self
    }

    pub fn voltage(mut self, voltage: impl fmt::Display) -> Self {
        self.voltage = Some(voltage.to_string());
        self
    }

    pub fn fp(mut self, footprint: impl Into<Footprint>) -> Self {
        self.footprint = footprint.into();
        self
    }

    pub fn polarized(mut self) -> Self {
        self.polarized = true;
        self.verify = true;
        self
    }

    pub fn verify(mut self) -> Self {
        self.verify = true;
        self
    }

    fn value_text(&self) -> String {
        match &self.voltage {
            Some(voltage) => format!("{} {}", self.capacitance, voltage),
            None => self.capacitance.clone(),
        }
    }
}

impl Component for ResistorBuilder {
    type Output = Resistor2;

    fn add_to(self, design: &mut Design) -> Result<Self::Output> {
        let mut builder = part(&self.refdes, &self.value)
            .footprint(self.footprint)
            .pin(pin("1").passive())
            .pin(pin("2").passive())
            .production_note("Bind exact resistance/tolerance/power LCSC part before production");
        if self.verify {
            builder = builder.verify();
        }
        design.add(builder.handle(|id| Resistor2 { id }))
    }
}

impl Component for CapacitorBuilder {
    type Output = Capacitor2;

    fn add_to(self, design: &mut Design) -> Result<Self::Output> {
        let mut builder = part(&self.refdes, self.value_text())
            .footprint(self.footprint)
            .pin(pin("1").passive())
            .pin(pin("2").passive())
            .production_note("Bind exact capacitance/voltage/package LCSC part before production");
        if self.verify || self.polarized {
            builder = builder.verify();
        }
        design.add(builder.handle(|id| Capacitor2 { id }))
    }
}

pub fn resistor_0402(refdes: &str, value: &str) -> impl Component<Output = Resistor2> {
    resistor_with_footprint(refdes, value, fp::r0402())
}

pub fn resistor_0603(refdes: &str, value: &str) -> impl Component<Output = Resistor2> {
    resistor_with_footprint(refdes, value, fp::r0603())
}

pub fn resistor_0805(refdes: &str, value: &str) -> impl Component<Output = Resistor2> {
    resistor_with_footprint(refdes, value, fp::r0805())
}

pub fn resistor_1206(refdes: &str, value: &str) -> impl Component<Output = Resistor2> {
    resistor_with_footprint(refdes, value, fp::r1206())
}

fn resistor_with_footprint(
    refdes: &str,
    value: &str,
    footprint: GeneratedFootprint,
) -> impl Component<Output = Resistor2> {
    part(refdes, value)
        .footprint(footprint)
        .pin(pin("1").passive())
        .pin(pin("2").passive())
        .production_note("Bind exact resistance/tolerance/power LCSC part before production")
        .handle(|id| Resistor2 { id })
}

pub fn capacitor_0402(refdes: &str, value: &str) -> impl Component<Output = Capacitor2> {
    capacitor_with_footprint(refdes, value, fp::c0402(), false)
}

pub fn capacitor_0603(refdes: &str, value: &str) -> impl Component<Output = Capacitor2> {
    capacitor_with_footprint(refdes, value, fp::c0603(), false)
}

pub fn capacitor_0805(refdes: &str, value: &str) -> impl Component<Output = Capacitor2> {
    capacitor_with_footprint(refdes, value, fp::c0805(), false)
}

pub fn capacitor_1206(refdes: &str, value: &str) -> impl Component<Output = Capacitor2> {
    capacitor_with_footprint(refdes, value, fp::c1206(), false)
}

pub fn capacitor_radial_d6p3_p2p50_verify(
    refdes: &str,
    value: &str,
) -> impl Component<Output = Capacitor2> {
    capacitor_with_footprint(refdes, value, fp::cp_d63_p25(), true)
}

fn capacitor_with_footprint(
    refdes: &str,
    value: &str,
    footprint: impl Into<Footprint>,
    verify: bool,
) -> impl Component<Output = Capacitor2> {
    let mut builder = part(refdes, value)
        .footprint(footprint)
        .pin(pin("1").passive())
        .pin(pin("2").passive())
        .production_note("Bind exact capacitance/voltage/package LCSC part before production");
    if verify {
        builder = builder.verify();
    }
    builder.handle(|id| Capacitor2 { id })
}

#[cfg(test)]
mod tests {
    use super::*;
    use via_core::Design;

    #[test]
    fn passive_parts_build_with_generic_part_spec() {
        let mut design = Design::new("generic_passives");
        let r = design.add(resistor_0805("R1", "1k")).unwrap();
        let c = design.add(capacitor_0805("C1", "100nF")).unwrap();
        let cp = design
            .add(capacitor_radial_d6p3_p2p50_verify("C2", "100uF 25V"))
            .unwrap();

        design
            .logic("N1", "3V3")
            .connect_all(&mut design, [r.pin1(), c.pin1(), cp.pin1()]);
        design
            .logic("N2", "3V3")
            .connect_all(&mut design, [r.pin2(), c.pin2(), cp.pin2()]);

        let board = design.build().unwrap();
        assert!(
            board
                .footprints()
                .any(|footprint| footprint.name() == "R_0805")
        );
        assert!(
            board
                .footprints()
                .any(|footprint| footprint.name() == "C_0805")
        );
        assert!(
            board
                .footprints()
                .any(|footprint| footprint.name() == "CP_D6.3_P2.5_VERIFY")
        );
    }

    #[test]
    fn chainable_passive_builders_use_default_modern_footprints() {
        let mut design = Design::new("chainable_passives");
        let r = design.add(resistor("R1").value("1k")).unwrap();
        let c = design
            .add(capacitor("C1").value("100nF").voltage("50V"))
            .unwrap();

        design
            .logic("N1", "3V3")
            .connect_all(&mut design, [r.pin1(), c.pin1()]);
        design
            .logic("N2", "3V3")
            .connect_all(&mut design, [r.pin2(), c.pin2()]);

        let board = design.build().unwrap();
        assert_eq!(board.module("R1").unwrap().value(), "1k");
        assert_eq!(board.module("C1").unwrap().value(), "100nF 50V");
        assert!(
            board
                .footprints()
                .any(|footprint| footprint.name() == "R_0805")
        );
        assert!(
            board
                .footprints()
                .any(|footprint| footprint.name() == "C_0805")
        );
    }
}
