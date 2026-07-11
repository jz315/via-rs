use via::prelude::*;

pub fn modern_api_minimal_board() -> Result<Board> {
    let mut d = Design::new("modern_api_minimal")
        .rules(Rules::new())
        .units(Unit::Mm);

    let signal = d.logic("SIGNAL", "3V3");
    let v3v3 = d.power("3V3", 3.3);
    let ground = d.ground("GND");

    let input = d.add(
        part("J1", "External 3.3V signal input")
            .footprint(fp::pin_1x03())
            .symbol(sym::connector().left(["SIG", "3V3", "GND"]))
            .pin(pin("SIG").logic("3V3").pad("1"))
            .pin(pin("3V3").power("3V3").pad("2"))
            .pin(pin("GND").ground().pad("3")),
    )?;
    let load = d.add(
        part("U1", "Demo load")
            .footprint(fp::pin_1x03())
            .symbol(sym::module().left(["IN"]).right(["VCC", "GND"]))
            .pin(pin("IN").logic("3V3").pad("1"))
            .pin(pin("VCC").power("3V3").pad("2"))
            .pin(pin("GND").ground().pad("3")),
    )?;

    d.connect(&signal, [input.pin("SIG"), load.pin("IN")]);
    d.connect(&v3v3, [input.pin("3V3"), load.pin("VCC")]);
    d.connect(&ground, [input.pin("GND"), load.pin("GND")]);

    d.finish(ValidationProfile::Prototype)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_modern_api_example_is_valid() {
        let board = modern_api_minimal_board().unwrap();
        assert_eq!(board.name(), "modern_api_minimal");
        assert_eq!(board.modules().count(), 2);
        assert_eq!(board.nets().count(), 3);
    }
}
