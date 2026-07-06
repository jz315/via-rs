use via::prelude::*;

pub fn minimal_power_led_board() -> Result<Board> {
    let mut d = Design::new("minimal_power_led").units(Unit::Mm);

    let v5 = d.rail("5V").dc(5.0);
    let ground = d.ground("GND");
    let led_anode = d.signal("LED_A", "5V");

    let input = d.add(
        part("J1", "5V input")
            .footprint(fp::pin_1x02())
            .symbol(sym::connector().left(["5V"]).right(["GND"]))
            .pin(pin("5V").power("5V").pad("1"))
            .pin(pin("GND").ground().pad("2")),
    )?;

    let resistor = d.add(parts::resistor("R1").value(1.kohm()).fp(fp::r0805()))?;

    let led = d.add(
        part("D1", "0805 status LED")
            .footprint(fp::led_0805())
            .symbol(sym::generic().left(["A"]).right(["K"]))
            .pin(pin("A").passive().pad("2"))
            .pin(pin("K").passive().pad("1"))
            .production_note("Verify LED color, polarity, and forward current before production")
            .verify(),
    )?;

    d.connect(&v5, [input.pin("5V"), resistor.pin1()]);
    d.connect(&led_anode, [resistor.pin2(), led.pin("A")]);
    d.connect(&ground, [input.pin("GND"), led.pin("K")]);

    d.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_power_led_example_is_valid() {
        let board = minimal_power_led_board().unwrap();
        assert_eq!(board.name(), "minimal_power_led");
        assert_eq!(board.modules().count(), 3);
        assert_eq!(board.nets().count(), 3);
        assert!(board.footprints().any(|fp| fp.name() == "R_0805"));
        assert!(board.footprints().any(|fp| fp.name() == "LED_0805"));
        assert!(board.diagnostics().is_empty());
    }
}
