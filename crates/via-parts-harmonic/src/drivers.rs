use via_core::{Component, ModuleId, PartSpecBuilder, PinRef, part, pin};

#[derive(Debug, Clone)]
pub struct SilentStepStickTmc2209V20 {
    id: ModuleId,
}

impl SilentStepStickTmc2209V20 {
    pub fn id(&self) -> &ModuleId {
        &self.id
    }

    pub fn dir(&self) -> PinRef {
        self.id.pin("DIR")
    }

    pub fn step(&self) -> PinRef {
        self.id.pin("STEP")
    }

    pub fn enable(&self) -> PinRef {
        self.id.pin("EN")
    }

    pub fn uart(&self) -> PinRef {
        self.id.pin("UART")
    }

    pub fn ms1(&self) -> PinRef {
        self.id.pin("MS1")
    }

    pub fn ms2(&self) -> PinRef {
        self.id.pin("MS2")
    }

    pub fn vmot(&self) -> PinRef {
        self.id.pin("VMOT")
    }

    pub fn ground(&self) -> PinRef {
        self.id.pin("GND")
    }

    pub fn oa2(&self) -> PinRef {
        self.id.pin("OA2")
    }

    pub fn oa1(&self) -> PinRef {
        self.id.pin("OA1")
    }

    pub fn ob1(&self) -> PinRef {
        self.id.pin("OB1")
    }

    pub fn ob2(&self) -> PinRef {
        self.id.pin("OB2")
    }

    pub fn vio(&self) -> PinRef {
        self.id.pin("VIO")
    }
}

pub fn silentstepstick_tmc2209_v20(
    refdes: &str,
) -> impl Component<Output = SilentStepStickTmc2209V20> {
    silentstepstick_tmc2209_v20_part(refdes).handle(|id| SilentStepStickTmc2209V20 { id })
}

fn silentstepstick_tmc2209_v20_part(refdes: &str) -> PartSpecBuilder {
    part(refdes, "SilentStepStick TMC2209 v2.0")
        .footprint("SilentStepStick_TMC2209_v20_CarrierSocket_2x8_Row12p70")
        .pin(pin("DIR").logic("3V3").pad("1"))
        .pin(pin("STEP").logic("3V3").pad("2"))
        .pin(pin("NC_J1_3").pad("3"))
        .pin(pin("NC_J1_4").pad("4"))
        .pin(pin("UART").logic("3V3").pad("5"))
        .pin(pin("MS2").pad("6"))
        .pin(pin("MS1").pad("7"))
        .pin(pin("EN").logic("3V3").pad("8"))
        .pin(pin("VMOT").power("12V").pad("9"))
        .pin(pin("GND").ground().pads(["10", "16"]))
        .pin(pin("OA2").motor_phase().pad("11"))
        .pin(pin("OA1").motor_phase().pad("12"))
        .pin(pin("OB1").motor_phase().pad("13"))
        .pin(pin("OB2").motor_phase().pad("14"))
        .pin(pin("VIO").power("3V3").pad("15"))
        .production_note(
            "Verify exact SilentStepStick/TMC2209 module dimensions and pinout before production",
        )
        .verify()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn tmc2209_socket_model_covers_all_16_socket_pads() {
        let module = silentstepstick_tmc2209_v20_part("U1").untyped();
        let pads = module
            .pins_iter()
            .flat_map(|pin| module.pads_for_pin(pin))
            .collect::<BTreeSet<_>>();

        let expected = (1..=16).map(|pad| pad.to_string()).collect::<BTreeSet<_>>();
        assert_eq!(pads, expected);
    }
}
