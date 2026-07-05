use via_core::{Component, DecouplerPins, ModuleId, PinRef, part, pin};

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

pub fn resistor_0603(refdes: &str, value: &str) -> impl Component<Output = Resistor2> {
    resistor(refdes, value, "R_0603_1608Metric")
}

pub fn resistor_0805(refdes: &str, value: &str) -> impl Component<Output = Resistor2> {
    resistor(refdes, value, "R_0805_2012Metric")
}

fn resistor(refdes: &str, value: &str, footprint: &str) -> impl Component<Output = Resistor2> {
    part(refdes, value)
        .footprint(footprint)
        .pin(pin("1").passive())
        .pin(pin("2").passive())
        .production_note("Bind exact resistance/tolerance/power LCSC part before production")
        .handle(|id| Resistor2 { id })
}

pub fn capacitor_0603(refdes: &str, value: &str) -> impl Component<Output = Capacitor2> {
    capacitor(refdes, value, "C_0603_1608Metric", false)
}

pub fn capacitor_0805(refdes: &str, value: &str) -> impl Component<Output = Capacitor2> {
    capacitor(refdes, value, "C_0805_2012Metric", false)
}

pub fn polarized_capacitor_radial_verify(
    refdes: &str,
    value: &str,
) -> impl Component<Output = Capacitor2> {
    capacitor(refdes, value, "CP_Radial_D6p3_P2p50_VERIFY", true)
}

fn capacitor(
    refdes: &str,
    value: &str,
    footprint: &str,
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
    use via_core::BoardSpec;

    #[test]
    fn passive_parts_build_with_generic_part_spec() {
        let mut board = BoardSpec::new("generic_passives");
        let r = board.add(resistor_0805("R1", "1k")).unwrap();
        let c = board.add(capacitor_0805("C1", "100nF")).unwrap();

        board.logic("N1", "3V3").connect_all([r.pin1(), c.pin1()]);
        board.logic("N2", "3V3").connect_all([r.pin2(), c.pin2()]);

        board.build().unwrap();
    }
}
