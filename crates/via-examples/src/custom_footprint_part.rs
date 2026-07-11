use via::footprint_ir::{FootprintIr, GraphicText, Pad, PadShape, Point, Size};
use via::prelude::*;

#[derive(Debug, Clone)]
pub struct CustomAnalogSensor {
    id: ModuleId,
}

impl CustomAnalogSensor {
    pub fn out(&self) -> PinRef {
        self.id.pin("OUT")
    }

    pub fn vcc(&self) -> PinRef {
        self.id.pin("VCC")
    }

    pub fn ground(&self) -> PinRef {
        self.id.pin("GND")
    }
}

fn custom_sensor_footprint() -> FootprintDefinition {
    let mut ir = FootprintIr::new("Demo_Sensor_SMD_1x03")
        .description("Example embedded SMD footprint for a three-pin sensor")
        .tag("via-example")
        .tag("custom-footprint");

    for (number, y) in [("1", -1.5), ("2", 0.0), ("3", 1.5)] {
        ir.add_pad(Pad::smd(
            number,
            PadShape::RoundRect,
            Point::new(-2.2, y),
            Size::new(1.2, 0.8),
        ));
    }
    ir.add_rect(Point::new(-3.2, -2.7), Point::new(2.8, 2.7), "F.Fab", 0.1)
        .add_rect(
            Point::new(-3.5, -3.0),
            Point::new(3.1, 3.0),
            "F.CrtYd",
            0.05,
        )
        .add_text(GraphicText::reference("REF**", Point::new(0.0, -4.0), "F.SilkS").size(0.9, 0.9))
        .add_text(
            GraphicText::value("Demo_Sensor_SMD_1x03", Point::new(0.0, 4.0), "F.Fab")
                .size(0.8, 0.8),
        );

    FootprintDefinition::generated(ir)
}

pub fn custom_analog_sensor(refdes: &str) -> impl Component<Output = CustomAnalogSensor> {
    part(refdes, "Custom analog sensor")
        .footprint(custom_sensor_footprint())
        .symbol(sym::module().left(["VCC", "GND"]).right(["OUT"]))
        .pin(pin("OUT").logic("3V3").pad("1"))
        .pin(pin("VCC").power("3V3").pad("2"))
        .pin(pin("GND").ground().pad("3"))
        .production_note("Example footprint geometry; replace with a datasheet-backed footprint")
        .needs_verification()
        .handle(|id| CustomAnalogSensor { id })
}

pub fn custom_footprint_part_board() -> Result<Board> {
    let mut d = Design::new("custom_footprint_part").units(Unit::Mm);

    let v3v3 = d.power("3V3", 3.3);
    let ground = d.ground("GND");
    let analog = d.logic("SENSOR_OUT", "3V3");

    let header = d.add(
        part("J1", "Host connector")
            .footprint(fp::pin_1x03())
            .symbol(sym::connector().left(["OUT"]).right(["3V3", "GND"]))
            .pin(pin("OUT").logic("3V3").pad("1"))
            .pin(pin("3V3").power("3V3").pad("2"))
            .pin(pin("GND").ground().pad("3")),
    )?;
    let sensor = d.add(custom_analog_sensor("U1"))?;
    let bypass = d.add(
        parts::capacitor("C1")
            .value(100.nf())
            .voltage(16.v())
            .fp(fp::c0603()),
    )?;

    d.connect(&analog, [header.pin("OUT"), sensor.out()]);
    d.connect(&v3v3, [header.pin("3V3"), sensor.vcc(), bypass.positive()]);
    d.connect(
        &ground,
        [header.pin("GND"), sensor.ground(), bypass.negative()],
    );

    d.finish(ValidationProfile::Prototype)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_footprint_part_example_is_valid() {
        let board = custom_footprint_part_board().unwrap();
        assert_eq!(board.name(), "custom_footprint_part");
        assert_eq!(board.modules().count(), 3);
        assert_eq!(board.nets().count(), 3);

        let footprint = board
            .footprints()
            .find(|footprint| footprint.name() == "Demo_Sensor_SMD_1x03")
            .unwrap();
        assert!(footprint.ir().is_some());
        assert!(footprint.contains_pad("1"));
        assert!(footprint.contains_pad("2"));
        assert!(footprint.contains_pad("3"));
        assert!(board.diagnostics().is_empty());
    }
}
