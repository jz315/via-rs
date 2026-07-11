use via::prelude::*;

#[derive(Debug, Clone)]
pub struct DemoI2cSensor {
    id: ModuleId,
}

impl DemoI2cSensor {
    pub fn vcc(&self) -> PinRef {
        self.id.pin("VCC")
    }

    pub fn ground(&self) -> PinRef {
        self.id.pin("GND")
    }

    pub fn scl(&self) -> PinRef {
        self.id.pin("SCL")
    }

    pub fn sda(&self) -> PinRef {
        self.id.pin("SDA")
    }
}

pub fn demo_i2c_sensor(refdes: &str) -> impl Component<Output = DemoI2cSensor> {
    part(refdes, "Demo I2C sensor module")
        .footprint("Demo_I2C_Sensor_1x04")
        .pin(pin("VCC").power("3V3").pad("1"))
        .pin(pin("GND").ground().pad("2"))
        .pin(pin("SCL").logic("3V3").pad("3"))
        .pin(pin("SDA").logic("3V3").pad("4"))
        .production_note("Replace demo sensor with an exact sourced module before production")
        .needs_verification()
        .handle(|id| DemoI2cSensor { id })
}

pub fn custom_part_board() -> Result<Board> {
    let mut design = Design::new("modern_custom_part");
    let v3v3 = design.power("3V3", 3.3);
    let ground = design.ground("GND");
    let scl = design.logic("I2C_SCL", "3V3");
    let sda = design.logic("I2C_SDA", "3V3");

    let sensor = design.add(demo_i2c_sensor("U1"))?;
    let header = design.add(
        part("J1", "I2C host header")
            .footprint("Header_1x04")
            .pin(pin("3V3").power("3V3"))
            .pin(pin("GND").ground())
            .pin(pin("SCL").logic("3V3"))
            .pin(pin("SDA").logic("3V3")),
    )?;

    design.connect(&v3v3, [header.pin("3V3"), sensor.vcc()]);
    design.connect(&ground, [header.pin("GND"), sensor.ground()]);
    design.connect(&scl, [header.pin("SCL"), sensor.scl()]);
    design.connect(&sda, [header.pin("SDA"), sensor.sda()]);

    design.finish(ValidationProfile::Draft)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_part_example_builds_with_typed_handle() {
        let board = custom_part_board().unwrap();
        assert_eq!(board.name(), "modern_custom_part");
        assert_eq!(
            board.module("U1").unwrap().footprint_name(),
            Some("Demo_I2C_Sensor_1x04")
        );
    }
}
