use via::prelude::*;

#[derive(Debug, Clone)]
pub struct GenericI2cSensor {
    id: ModuleId,
}

impl GenericI2cSensor {
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

pub fn generic_i2c_sensor(refdes: &str) -> impl Component<Output = GenericI2cSensor> {
    part(refdes, "Generic I2C sensor")
        .footprint(fp::pin_1x04())
        .symbol(sym::module().left(["VCC", "GND"]).right(["SCL", "SDA"]))
        .pin(pin("VCC").power("3V3").pad("1"))
        .pin(pin("GND").ground().pad("2"))
        .pin(pin("SCL").logic("3V3").pad("3"))
        .pin(pin("SDA").logic("3V3").pad("4"))
        .production_note("Replace with a sourced sensor module or IC footprint before production")
        .verify()
        .handle(|id| GenericI2cSensor { id })
}

pub fn i2c_sensor_breakout_board() -> Result<Board> {
    let mut d = Design::new("i2c_sensor_breakout").units(Unit::Mm);

    let v3v3 = d.rail("3V3").dc(3.3);
    let ground = d.ground("GND");
    let scl = d.signal("I2C_SCL", "3V3");
    let sda = d.signal("I2C_SDA", "3V3");

    let host = d.add(
        part("J1", "I2C host connector")
            .footprint(fp::pin_1x04())
            .symbol(sym::connector().left(["3V3", "GND"]).right(["SCL", "SDA"]))
            .pin(pin("3V3").power("3V3").pad("1"))
            .pin(pin("GND").ground().pad("2"))
            .pin(pin("SCL").logic("3V3").pad("3"))
            .pin(pin("SDA").logic("3V3").pad("4")),
    )?;

    let sensor = d.add(generic_i2c_sensor("U1"))?;
    let scl_pullup = d.add(parts::resistor("R1").value(4.7.kohm()).fp(fp::r0603()))?;
    let sda_pullup = d.add(parts::resistor("R2").value(4.7.kohm()).fp(fp::r0603()))?;
    let bypass = d.add(
        parts::capacitor("C1")
            .value(100.nf())
            .voltage(16.v())
            .fp(fp::c0805()),
    )?;

    d.connect(
        &v3v3,
        [
            host.pin("3V3"),
            sensor.vcc(),
            scl_pullup.pin1(),
            sda_pullup.pin1(),
            bypass.positive(),
        ],
    );
    d.connect(
        &ground,
        [host.pin("GND"), sensor.ground(), bypass.negative()],
    );
    d.connect(&scl, [host.pin("SCL"), sensor.scl(), scl_pullup.pin2()]);
    d.connect(&sda, [host.pin("SDA"), sensor.sda(), sda_pullup.pin2()]);

    d.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn i2c_sensor_breakout_example_is_valid() {
        let board = i2c_sensor_breakout_board().unwrap();
        assert_eq!(board.name(), "i2c_sensor_breakout");
        assert_eq!(board.modules().count(), 5);
        assert_eq!(board.nets().count(), 4);
        assert!(board.footprints().any(|fp| fp.name() == "Pin_1x04_P2.54"));
        assert!(board.footprints().any(|fp| fp.name() == "R_0603"));
        assert!(board.footprints().any(|fp| fp.name() == "C_0805"));
        assert!(board.diagnostics().is_empty());
    }
}
