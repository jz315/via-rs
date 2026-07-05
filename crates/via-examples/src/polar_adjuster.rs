use via::parts::{
    capacitor_0805, esp32_s3_n16r8, generated_footprint_pads, polarized_capacitor_radial_verify,
};
use via::patterns::{ActiveLowSwitchInputSpec, DcBuckInputStageSpec};
use via::patterns::{Tmc2209UartAxisPins, Tmc2209UartAxisSpec};
use via::prelude::*;

pub fn polar_adjuster_v0_board() -> Result<Board> {
    let mut design = Design::new("polar_adjuster_v0").units(Unit::Mm);

    for footprint in generated_footprint_pads() {
        design.add_footprint_pads(footprint);
    }

    let ground = design.ground("GND");
    let v12 = design.power("12V_IN", Voltage::dc(12.0));
    let v5 = design.power("5V_BUCK", Voltage::dc(5.0));
    let v3v3 = design.power("3V3", Voltage::dc(3.3));

    let esp32 = design.add(esp32_s3_n16r8("U1"))?;
    let x_axis = design.add(
        Tmc2209UartAxisSpec::new("X")
            .driver("U2")
            .motor_connector("J2")
            .uart_resistor("R1")
            .pins(Tmc2209UartAxisPins::new(
                esp32.gpio4(),
                esp32.gpio5(),
                esp32.gpio6(),
                esp32.gpio7(),
                esp32.gpio15(),
            )),
    )?;
    let y_axis = design.add(
        Tmc2209UartAxisSpec::new("Y")
            .driver("U3")
            .motor_connector("J3")
            .uart_resistor("R2")
            .pins(Tmc2209UartAxisPins::new(
                esp32.gpio16(),
                esp32.gpio17(),
                esp32.gpio18(),
                esp32.gpio9(),
                esp32.gpio10(),
            )),
    )?;
    design.add(
        DcBuckInputStageSpec::new()
            .dc_jack("J1")
            .buck("U4")
            .input_bulk("C1")
            .input_hf("C2")
            .buck_input_bulk("C7")
            .output_bulk("C8")
            .output_hf("C9")
            .input_loads([x_axis.vmot(), y_axis.vmot()])
            .output_loads([esp32.power_5v()]),
    )?;
    let x_vmot_bulk = design.add(polarized_capacitor_radial_verify(
        "C3",
        "100uF 25V X VMOT bulk VERIFY",
    ))?;
    let x_vio_decouple = design.add(capacitor_0805("C4", "100nF 50V X VIO decoupling"))?;
    let y_vmot_bulk = design.add(polarized_capacitor_radial_verify(
        "C5",
        "100uF 25V Y VMOT bulk VERIFY",
    ))?;
    let y_vio_decouple = design.add(capacitor_0805("C6", "100nF 50V Y VIO decoupling"))?;
    let esp32_5v_local = design.add(capacitor_0805("C10", "10uF 10V ESP32 5V local"))?;
    design.add(
        ActiveLowSwitchInputSpec::new("ESTOP_SW")
            .header("J6")
            .value("E-stop switch active-low")
            .signal(esp32.gpio39()),
    )?;

    ground.connect_all(
        &mut design,
        [
            esp32.ground(),
            x_axis.ground(),
            y_axis.ground(),
            x_axis.ms1(),
            x_axis.ms2(),
            y_axis.ms1(),
            y_axis.ms2(),
        ],
    );
    v12.decouple(&mut design, &x_vmot_bulk)
        .decouple(&mut design, &y_vmot_bulk);
    v5.decouple(&mut design, &esp32_5v_local);
    v3v3.connect_all(&mut design, [esp32.power_3v3(), x_axis.vio(), y_axis.vio()])
        .decouple(&mut design, &x_vio_decouple)
        .decouple(&mut design, &y_vio_decouple);

    design.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn polar_adjuster_example_is_valid() {
        let board = polar_adjuster_v0_board().unwrap();
        board.check().unwrap();
    }

    #[test]
    fn polar_adjuster_uses_nearby_independent_tmc_control_pins() {
        let board = polar_adjuster_v0_board().unwrap();

        assert_net(&board, "X_EN", &[("U1", "GPIO4"), ("U2", "EN")]);
        assert_net(&board, "X_UART_TX", &[("U1", "GPIO5"), ("R1", "1")]);
        assert_net(
            &board,
            "X_UART",
            &[("U1", "GPIO6"), ("R1", "2"), ("U2", "UART")],
        );
        assert_net(&board, "X_STEP", &[("U1", "GPIO7"), ("U2", "STEP")]);
        assert_net(&board, "X_DIR", &[("U1", "GPIO15"), ("U2", "DIR")]);
        assert_net(&board, "Y_EN", &[("U1", "GPIO16"), ("U3", "EN")]);
        assert_net(&board, "Y_UART_TX", &[("U1", "GPIO17"), ("R2", "1")]);
        assert_net(
            &board,
            "Y_UART",
            &[("U1", "GPIO18"), ("R2", "2"), ("U3", "UART")],
        );
        assert_net(&board, "Y_STEP", &[("U1", "GPIO9"), ("U3", "STEP")]);
        assert_net(&board, "Y_DIR", &[("U1", "GPIO10"), ("U3", "DIR")]);
    }

    #[test]
    fn polar_adjuster_has_local_power_decoupling() {
        let board = polar_adjuster_v0_board().unwrap();

        assert_net(
            &board,
            "12V_IN",
            &[
                ("J1", "4"),
                ("U4", "IN+"),
                ("U2", "VMOT"),
                ("U3", "VMOT"),
                ("C1", "1"),
                ("C2", "1"),
                ("C7", "1"),
                ("C3", "1"),
                ("C5", "1"),
            ],
        );
        assert_net(
            &board,
            "5V_BUCK",
            &[
                ("U4", "OUT+"),
                ("U1", "5VIN"),
                ("C8", "1"),
                ("C9", "1"),
                ("C10", "1"),
            ],
        );
        assert_net(
            &board,
            "3V3",
            &[
                ("U1", "3V3"),
                ("U2", "VIO"),
                ("U3", "VIO"),
                ("C4", "1"),
                ("C6", "1"),
            ],
        );
    }

    fn assert_net(board: &Board, name: &str, expected: &[(&str, &str)]) {
        let net = board
            .nets()
            .find(|net| net.name() == name)
            .unwrap_or_else(|| panic!("missing net {name}"));
        let actual = net
            .connections()
            .iter()
            .map(|pin| (pin.module.as_str(), pin.pin.as_str()))
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "{name}");
    }
}
