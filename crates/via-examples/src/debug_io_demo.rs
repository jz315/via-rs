use via::prelude::*;

pub fn debug_io_demo_board() -> Result<Board> {
    let mut design = Design::new("debug_io_demo").units(Unit::Mm);

    let vin = design.rail("5V_IN").dc(5.0);
    let v3v3 = design.rail("3V3").dc(3.3);
    let ground = design.ground("GND");
    let i2c_scl = design.signal("I2C_SCL", "3V3");
    let i2c_sda = design.signal("I2C_SDA", "3V3");
    let gpio_a = design.signal("GPIO_A", "3V3");
    let gpio_b = design.signal("GPIO_B", "3V3");
    let led_node = design.signal("LED_STATUS", "3V3");
    let bus_b1 = design.signal("BUS_B1", "3V3");
    let bus_b2 = design.signal("BUS_B2", "3V3");
    let bus_b3 = design.signal("BUS_B3", "3V3");
    let bus_b4 = design.signal("BUS_B4", "3V3");

    let power_in = design.add(
        part("J1", "JST-PH 5V input")
            .footprint(fp::ph_2p())
            .symbol(sym::connector().left(["1"]).right(["2"]))
            .pin(pin("1").power("5V").pad("1"))
            .pin(pin("2").ground().pad("2"))
            .production_note(
                "Verify JST-PH footprint against the selected connector before fabrication",
            )
            .verify(),
    )?;

    let regulator = design.add(
        part("U1", "SOT-223 3.3V regulator placeholder")
            .footprint(fp::sot223())
            .symbol(sym::module().left(["VIN", "GND"]).right(["VOUT", "TAB"]))
            .pin(pin("VIN").power("5V").pad("1"))
            .pin(pin("GND").ground().pad("2"))
            .pin(pin("VOUT").power("3V3").pad("3"))
            .pin(pin("TAB").power("3V3").pad("4"))
            .production_note("Bind a real regulator MPN and verify SOT-223 land pattern")
            .verify(),
    )?;

    let c_in = design.add(
        parts::capacitor("C1")
            .value(10.uf())
            .voltage(10.v())
            .fp(fp::c0805()),
    )?;
    let c_out = design.add(
        parts::capacitor("C2")
            .value(10.uf())
            .voltage(10.v())
            .fp(fp::c0805()),
    )?;
    let led_resistor = design.add(parts::resistor("R1").value(1.kohm()).fp(fp::r0805()))?;

    let led = design.add(
        part("D1", "0805 status LED")
            .footprint(fp::led_0805())
            .symbol(sym::generic().left(["A"]).right(["K"]))
            .pin(pin("K").passive().pad("1"))
            .pin(pin("A").passive().pad("2"))
            .production_note("Verify LED polarity and exact 0805 package")
            .verify(),
    )?;

    let io_header = design.add(
        part("J2", "XH 4-pin debug IO")
            .footprint(fp::xh_4p())
            .symbol(sym::connector().left(["1", "2"]).right(["3", "4"]))
            .pin(pin("1").power("3V3").pad("1"))
            .pin(pin("2").ground().pad("2"))
            .pin(pin("3").logic("3V3").pad("3"))
            .pin(pin("4").logic("3V3").pad("4"))
            .production_note("Verify purchased XH connector orientation and pin numbering")
            .verify(),
    )?;

    let aux_header = design.add(
        part("J3", "2x05 expansion header")
            .footprint(fp::pin_2x05())
            .symbol(
                sym::connector()
                    .left(["1", "3", "5", "7", "9"])
                    .right(["2", "4", "6", "8", "10"]),
            )
            .pins(["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"])
            .production_note("Generic 2x05 header placeholder; verify mating connector"),
    )?;

    let io_buffer = design.add(
        part("U2", "SOIC-8 debug buffer placeholder")
            .footprint(fp::soic8())
            .symbol(
                sym::ic()
                    .left(["A1", "A2", "OE", "GND"])
                    .right(["Y1", "Y2", "VCC", "NC"]),
            )
            .pin(pin("A1").logic("3V3").pad("1"))
            .pin(pin("Y1").logic("3V3").pad("2"))
            .pin(pin("A2").logic("3V3").pad("3"))
            .pin(pin("Y2").logic("3V3").pad("4"))
            .pin(pin("GND").ground().pad("5"))
            .pin(pin("OE").logic("3V3").pad("6"))
            .pin(pin("NC").pad("7"))
            .pin(pin("VCC").power("3V3").pad("8"))
            .production_note("SOIC-8 placeholder for buffer/level-shifter experiments")
            .verify(),
    )?;

    let small_switch = design.add(
        part("Q1", "SOT-23 GPIO switch placeholder")
            .footprint(fp::sot23_3())
            .symbol(sym::generic().left(["B"]).right(["C", "E"]))
            .pin(pin("B").logic("3V3").pad("1"))
            .pin(pin("C").passive().pad("2"))
            .pin(pin("E").ground().pad("3"))
            .production_note(
                "SOT-23 placeholder; bind real transistor/MOSFET pinout before production",
            )
            .verify(),
    )?;

    let bus_buffer = design.add(
        part("U3", "74HC245 TSSOP-20 bus buffer demo")
            .footprint(fp::tssop20())
            .symbol(
                sym::ic()
                    .left(["DIR", "A1", "A2", "A3", "A4", "A5", "A6", "A7", "A8", "GND"])
                    .right([
                        "VCC", "OE_N", "B1", "B2", "B3", "B4", "B5", "B6", "B7", "B8",
                    ]),
            )
            .pin(pin("DIR").passive().pad("1"))
            .pin(pin("A1").logic("3V3").pad("2"))
            .pin(pin("A2").logic("3V3").pad("3"))
            .pin(pin("A3").logic("3V3").pad("4"))
            .pin(pin("A4").logic("3V3").pad("5"))
            .pin(pin("A5").logic("3V3").pad("6"))
            .pin(pin("A6").logic("3V3").pad("7"))
            .pin(pin("A7").logic("3V3").pad("8"))
            .pin(pin("A8").logic("3V3").pad("9"))
            .pin(pin("GND").ground().pad("10"))
            .pin(pin("B8").logic("3V3").pad("11"))
            .pin(pin("B7").logic("3V3").pad("12"))
            .pin(pin("B6").logic("3V3").pad("13"))
            .pin(pin("B5").logic("3V3").pad("14"))
            .pin(pin("B4").logic("3V3").pad("15"))
            .pin(pin("B3").logic("3V3").pad("16"))
            .pin(pin("B2").logic("3V3").pad("17"))
            .pin(pin("B1").logic("3V3").pad("18"))
            .pin(pin("OE_N").passive().pad("19"))
            .pin(pin("VCC").power("3V3").pad("20"))
            .production_note("74HC245-style TSSOP-20 demo; verify exact vendor pinout before use")
            .verify(),
    )?;

    let tp_5v = design.add(testpad("TP1", "5V test pad", fp::testpad_1p5(), "5V"))?;
    let tp_3v3 = design.add(testpad("TP2", "3V3 test pad", fp::testpad_1p5(), "3V3"))?;
    let tp_gnd = design.add(testpad("TP3", "GND test pad", fp::testpad_2p0(), "GND"))?;
    let tp_scl = design.add(testpad("TP4", "SCL test pad", fp::testpad_1p0(), "SCL"))?;
    let tp_sda = design.add(testpad("TP5", "SDA test pad", fp::testpad_1p0(), "SDA"))?;

    design.add(mechanical_marker(
        "FID1",
        "Top fiducial",
        fp::fiducial_1p0(),
    ))?;
    design.add(mechanical_marker("H1", "M3 mounting hole", fp::mh_m3_np()))?;
    design.add(mechanical_marker("H2", "M3 mounting hole", fp::mh_m3_np()))?;

    design.connect(
        &vin,
        [
            power_in.pin("1"),
            regulator.pin("VIN"),
            c_in.positive(),
            tp_5v.pin("1"),
        ],
    );
    design.connect(
        &v3v3,
        [
            regulator.pin("VOUT"),
            regulator.pin("TAB"),
            c_out.positive(),
            led_resistor.pin1(),
            io_header.pin("1"),
            aux_header.pin("1"),
            io_buffer.pin("VCC"),
            bus_buffer.pin("VCC"),
            bus_buffer.pin("DIR"),
            tp_3v3.pin("1"),
        ],
    );
    design.connect(
        &ground,
        [
            power_in.pin("2"),
            regulator.pin("GND"),
            c_in.negative(),
            c_out.negative(),
            led.pin("K"),
            io_header.pin("2"),
            aux_header.pin("2"),
            io_buffer.pin("GND"),
            small_switch.pin("E"),
            bus_buffer.pin("GND"),
            bus_buffer.pin("OE_N"),
            tp_gnd.pin("1"),
        ],
    );

    design.connect(&led_node, [led_resistor.pin2(), led.pin("A")]);
    design.connect(
        &i2c_scl,
        [
            io_header.pin("3"),
            aux_header.pin("3"),
            io_buffer.pin("A1"),
            bus_buffer.pin("A1"),
            tp_scl.pin("1"),
        ],
    );
    design.connect(
        &i2c_sda,
        [
            io_header.pin("4"),
            aux_header.pin("4"),
            io_buffer.pin("A2"),
            bus_buffer.pin("A2"),
            tp_sda.pin("1"),
        ],
    );
    design.connect(
        &gpio_a,
        [
            aux_header.pin("5"),
            io_buffer.pin("Y1"),
            small_switch.pin("B"),
            bus_buffer.pin("A3"),
        ],
    );
    design.connect(
        &gpio_b,
        [
            aux_header.pin("6"),
            io_buffer.pin("Y2"),
            bus_buffer.pin("A4"),
        ],
    );
    design.connect(&bus_b1, [aux_header.pin("7"), bus_buffer.pin("B1")]);
    design.connect(&bus_b2, [aux_header.pin("8"), bus_buffer.pin("B2")]);
    design.connect(&bus_b3, [aux_header.pin("9"), bus_buffer.pin("B3")]);
    design.connect(&bus_b4, [aux_header.pin("10"), bus_buffer.pin("B4")]);

    design.finish()
}

fn testpad(
    refdes: &str,
    value: &str,
    footprint: impl Into<Footprint>,
    label: &str,
) -> impl Component<Output = ModuleId> {
    part(refdes, value)
        .footprint(footprint)
        .symbol(sym::connector().left(["1"]))
        .pin(pin("1").passive().pad("1"))
        .production_note(format!("Test pad for {label}"))
}

fn mechanical_marker(
    refdes: &str,
    value: &str,
    footprint: impl Into<Footprint>,
) -> impl Component<Output = ModuleId> {
    part(refdes, value)
        .footprint(footprint)
        .symbol(sym::connector().left(["1"]))
        .pin(pin("1").passive().pad("1"))
        .production_note("Mechanical/reference footprint; normally left unconnected")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_io_demo_example_is_valid() {
        let board = debug_io_demo_board().unwrap();
        assert_eq!(board.name(), "debug_io_demo");
        assert!(board.module("J1").is_some());
        assert!(board.module("U3").is_some());
        assert_eq!(board.diagnostics(), Vec::new());
        assert!(board.footprints().any(|fp| fp.name() == "TSSOP-20"));
        assert!(board.footprints().any(|fp| fp.name() == "MH_M3_NPTH_D3.2"));
    }
}
