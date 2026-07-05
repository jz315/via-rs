use via_core::{Component, ModuleId, PinRef, part, pin};

#[derive(Debug, Clone)]
pub struct Xh2p54Motor4 {
    id: ModuleId,
}

impl Xh2p54Motor4 {
    pub fn id(&self) -> &ModuleId {
        &self.id
    }

    pub fn a2(&self) -> PinRef {
        self.id.pin("A2")
    }

    pub fn a1(&self) -> PinRef {
        self.id.pin("A1")
    }

    pub fn b1(&self) -> PinRef {
        self.id.pin("B1")
    }

    pub fn b2(&self) -> PinRef {
        self.id.pin("B2")
    }
}

#[derive(Debug, Clone)]
pub struct TerminalBlock5 {
    id: ModuleId,
}

impl TerminalBlock5 {
    pub fn id(&self) -> &ModuleId {
        &self.id
    }

    pub fn pin1(&self) -> PinRef {
        self.id.pin("1")
    }

    pub fn pin2(&self) -> PinRef {
        self.id.pin("2")
    }

    pub fn pin3(&self) -> PinRef {
        self.id.pin("3")
    }

    pub fn pin4(&self) -> PinRef {
        self.id.pin("4")
    }

    pub fn pin5(&self) -> PinRef {
        self.id.pin("5")
    }
}

#[derive(Debug, Clone)]
pub struct Header2 {
    id: ModuleId,
}

impl Header2 {
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

pub fn xh2p54_motor4(refdes: &str, value: &str) -> impl Component<Output = Xh2p54Motor4> {
    part(refdes, value)
        .footprint("XH2p54_1x04_Vertical_THT_VERIFY")
        .pin(pin("A2").motor_phase().pad("1"))
        .pin(pin("A1").motor_phase().pad("2"))
        .pin(pin("B1").motor_phase().pad("3"))
        .pin(pin("B2").motor_phase().pad("4"))
        .production_note("Verify purchased XH-style connector pin pitch and housing orientation")
        .verify()
        .handle(|id| Xh2p54Motor4 { id })
}

pub fn pin_header_1x02(refdes: &str, value: &str) -> impl Component<Output = Header2> {
    part(refdes, value)
        .footprint("PinHeader_1x02_P2.54")
        .pins(["1", "2"])
        .production_note("Bind exact header or terminal part before production")
        .verify()
        .handle(|id| Header2 { id })
}

pub fn terminal_block_1x05(refdes: &str, value: &str) -> impl Component<Output = TerminalBlock5> {
    part(refdes, value)
        .footprint("TerminalBlock_1x05_P5.08")
        .pins(["1", "2", "3", "4", "5"])
        .production_note("Bind exact terminal block part before production")
        .verify()
        .handle(|id| TerminalBlock5 { id })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use via_core::BoardSpec;

    #[test]
    fn xh_motor_connector_uses_tmc_friendly_physical_order() {
        let mut board = BoardSpec::new("motor_connector_order");
        board.add(xh2p54_motor4("J1", "motor")).unwrap();
        let connector = board.board().module("J1").unwrap();

        assert_eq!(
            connector.pads_for_pin("A2"),
            BTreeSet::from(["1".to_owned()])
        );
        assert_eq!(
            connector.pads_for_pin("A1"),
            BTreeSet::from(["2".to_owned()])
        );
        assert_eq!(
            connector.pads_for_pin("B1"),
            BTreeSet::from(["3".to_owned()])
        );
        assert_eq!(
            connector.pads_for_pin("B2"),
            BTreeSet::from(["4".to_owned()])
        );
    }
}
