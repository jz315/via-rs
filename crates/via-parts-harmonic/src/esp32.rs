use via_core::{Component, ModuleId, PartSpecBuilder, PinRef, part, pin};

#[derive(Debug, Clone)]
pub struct Esp32S3N16R8 {
    id: ModuleId,
}

impl Esp32S3N16R8 {
    pub fn id(&self) -> &ModuleId {
        &self.id
    }

    pub fn gpio4(&self) -> PinRef {
        self.id.pin("GPIO4")
    }

    pub fn gpio5(&self) -> PinRef {
        self.id.pin("GPIO5")
    }

    pub fn gpio6(&self) -> PinRef {
        self.id.pin("GPIO6")
    }

    pub fn gpio7(&self) -> PinRef {
        self.id.pin("GPIO7")
    }

    pub fn gpio9(&self) -> PinRef {
        self.id.pin("GPIO9")
    }

    pub fn gpio10(&self) -> PinRef {
        self.id.pin("GPIO10")
    }

    pub fn gpio11(&self) -> PinRef {
        self.id.pin("GPIO11")
    }

    pub fn gpio15(&self) -> PinRef {
        self.id.pin("GPIO15")
    }

    pub fn gpio16(&self) -> PinRef {
        self.id.pin("GPIO16")
    }

    pub fn gpio17(&self) -> PinRef {
        self.id.pin("GPIO17")
    }

    pub fn gpio18(&self) -> PinRef {
        self.id.pin("GPIO18")
    }

    pub fn gpio38(&self) -> PinRef {
        self.id.pin("GPIO38")
    }

    pub fn gpio39(&self) -> PinRef {
        self.id.pin("GPIO39")
    }

    pub fn power_3v3(&self) -> PinRef {
        self.id.pin("3V3")
    }

    pub fn power_5v(&self) -> PinRef {
        self.id.pin("5VIN")
    }

    pub fn ground(&self) -> PinRef {
        self.id.pin("GND")
    }
}

pub fn esp32_s3_n16r8(refdes: &str) -> impl Component<Output = Esp32S3N16R8> {
    esp32_s3_n16r8_part(refdes).handle(|id| Esp32S3N16R8 { id })
}

fn esp32_s3_n16r8_part(refdes: &str) -> PartSpecBuilder {
    part(refdes, "ESP32-S3 N16R8 dev board")
        .footprint("ESP32-S3-N16R8_DevBoard_2x22_P2.54_Row25.40")
        .pin(pin("3V3").power("3V3").pads(["1", "2"]))
        .pin(pin("RST").pad("3"))
        .pin(pin("GPIO4").logic("3V3").pad("4"))
        .pin(pin("GPIO5").logic("3V3").pad("5"))
        .pin(pin("GPIO6").logic("3V3").pad("6"))
        .pin(pin("GPIO7").logic("3V3").pad("7"))
        .pin(pin("GPIO15").logic("3V3").pad("8"))
        .pin(pin("GPIO16").logic("3V3").pad("9"))
        .pin(pin("GPIO17").logic("3V3").pad("10"))
        .pin(pin("GPIO18").logic("3V3").pad("11"))
        .pin(pin("GPIO8").pad("12"))
        .pin(pin("GPIO3").pad("13"))
        .pin(pin("GPIO46").pad("14"))
        .pin(pin("GPIO9").logic("3V3").pad("15"))
        .pin(pin("GPIO10").logic("3V3").pad("16"))
        .pin(pin("GPIO11").logic("3V3").pad("17"))
        .pin(pin("GPIO12").pad("18"))
        .pin(pin("GPIO13").pad("19"))
        .pin(pin("GPIO14").pad("20"))
        .pin(pin("5VIN").power("5V").pad("21"))
        .pin(pin("GND").ground().pads(["22", "23", "43", "44"]))
        .pin(pin("TX").pad("24"))
        .pin(pin("RX").pad("25"))
        .pin(pin("GPIO1").pad("26"))
        .pin(pin("GPIO2").pad("27"))
        .pin(pin("GPIO42").pad("28"))
        .pin(pin("GPIO41").pad("29"))
        .pin(pin("GPIO40").pad("30"))
        .pin(pin("GPIO39").logic("3V3").pad("31"))
        .pin(pin("GPIO38").logic("3V3").pad("32"))
        .pin(pin("GPIO37").pad("33"))
        .pin(pin("GPIO36").pad("34"))
        .pin(pin("GPIO35").pad("35"))
        .pin(pin("GPIO0").pad("36"))
        .pin(pin("GPIO45").pad("37"))
        .pin(pin("GPIO48").pad("38"))
        .pin(pin("GPIO47").pad("39"))
        .pin(pin("GPIO21").pad("40"))
        .pin(pin("GPIO20").pad("41"))
        .pin(pin("GPIO19").pad("42"))
        .production_note(
            "Verify exact ESP32-S3 N16R8 dev board outline and header pinout before production",
        )
        .verify()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn esp32_devboard_model_covers_all_44_socket_pads() {
        let module = esp32_s3_n16r8_part("U1").untyped();
        let pads = module
            .pins_iter()
            .flat_map(|pin| module.pads_for_pin(pin))
            .collect::<BTreeSet<_>>();

        let expected = (1..=44).map(|pad| pad.to_string()).collect::<BTreeSet<_>>();
        assert_eq!(pads, expected);
    }
}
