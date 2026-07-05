use via_core::{Component, ModuleId, PinRef, part, pin};

#[derive(Debug, Clone)]
pub struct Dc005BarrelJack {
    id: ModuleId,
}

impl Dc005BarrelJack {
    pub fn id(&self) -> &ModuleId {
        &self.id
    }

    pub fn sleeve_ground_verify(&self) -> PinRef {
        self.id.pin("2")
    }

    pub fn switched_nc_verify(&self) -> PinRef {
        self.id.pin("3")
    }

    pub fn tip_12v(&self) -> PinRef {
        self.id.pin("4")
    }
}

#[derive(Debug, Clone)]
pub struct Mp1584BuckAdapter {
    id: ModuleId,
}

impl Mp1584BuckAdapter {
    pub fn id(&self) -> &ModuleId {
        &self.id
    }

    pub fn input_positive(&self) -> PinRef {
        self.id.pin("IN+")
    }

    pub fn input_negative(&self) -> PinRef {
        self.id.pin("IN-")
    }

    pub fn output_positive(&self) -> PinRef {
        self.id.pin("OUT+")
    }

    pub fn output_negative(&self) -> PinRef {
        self.id.pin("OUT-")
    }
}

pub fn dc005_barrel_jack(refdes: &str) -> impl Component<Output = Dc005BarrelJack> {
    part(refdes, "DC-005 5.5x2.1 right-angle barrel jack")
        .footprint("DC005_5p5x2p1_RightAngle_THT_Drawing_2_3_4_VERIFY")
        .pin(pin("2").ground())
        .pin(pin("3"))
        .pin(pin("4").power("12V"))
        .production_note("Verify DC jack switched contact and sleeve pin with a multimeter")
        .verify()
        .handle(|id| Dc005BarrelJack { id })
}

pub fn mp1584_buck_adapter(refdes: &str) -> impl Component<Output = Mp1584BuckAdapter> {
    part(refdes, "MP1584 buck adapter")
        .footprint("BuckModule_4Wire_MP1584_Adapter")
        .pin(pin("IN+").power("12V").pad("1"))
        .pin(pin("IN-").ground().pad("2"))
        .pin(pin("OUT+").power("5V").pad("3"))
        .pin(pin("OUT-").ground().pad("4"))
        .production_note(
            "Verify MP1584 module pin order, hole pitch, and adjusted 5V output before production",
        )
        .verify()
        .handle(|id| Mp1584BuckAdapter { id })
}
