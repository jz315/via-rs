use via::prelude::*;

pub fn modern_api_minimal_board() -> Result<Board> {
    let mut design = Design::new("modern_api_minimal")
        .rules(Rules::new())
        .units(Unit::Mm);

    let signal = design.logic("SIGNAL", "3V3");
    let v3v3 = design.power("3V3", Voltage::dc(3.3));
    let ground = design.ground("GND");

    let input = design.add(
        part("J1", "External 3.3V signal input")
            .footprint("Header_1x03")
            .pin(pin("SIG").logic("3V3"))
            .pin(pin("3V3").power("3V3"))
            .pin(pin("GND").ground()),
    )?;
    let load = design.add(
        part("U1", "Demo load")
            .footprint("Demo_Load_3Pin")
            .pin(pin("IN").logic("3V3"))
            .pin(pin("VCC").power("3V3"))
            .pin(pin("GND").ground()),
    )?;

    signal.connect_all(&mut design, [input.pin("SIG"), load.pin("IN")]);
    v3v3.connect_all(&mut design, [input.pin("3V3"), load.pin("VCC")]);
    ground.connect_all(&mut design, [input.pin("GND"), load.pin("GND")]);

    design.check(CheckProfile::Prototype)?;
    design.build()
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
