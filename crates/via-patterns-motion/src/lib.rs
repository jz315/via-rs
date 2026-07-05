use via_core::{BoardSpec, Component, Error, PinRef, Result};
use via_parts::{Resistor2, resistor_0805};
use via_parts_harmonic::{
    SilentStepStickTmc2209V20, Xh2p54Motor4, silentstepstick_tmc2209_v20, xh2p54_motor4,
};

#[derive(Debug, Clone)]
pub struct Tmc2209UartAxisPins {
    pub enable: PinRef,
    pub uart_tx: PinRef,
    pub uart_rx: PinRef,
    pub step: PinRef,
    pub dir: PinRef,
}

impl Tmc2209UartAxisPins {
    pub fn new(
        enable: PinRef,
        uart_tx: PinRef,
        uart_rx: PinRef,
        step: PinRef,
        dir: PinRef,
    ) -> Self {
        Self {
            enable,
            uart_tx,
            uart_rx,
            step,
            dir,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tmc2209UartAxisSpec {
    axis: String,
    driver_refdes: Option<String>,
    motor_connector_refdes: Option<String>,
    uart_resistor_refdes: Option<String>,
    logic_domain: String,
    uart_resistor_value: String,
    pins: Option<Tmc2209UartAxisPins>,
}

impl Tmc2209UartAxisSpec {
    pub fn new(axis: impl Into<String>) -> Self {
        Self {
            axis: axis.into(),
            driver_refdes: None,
            motor_connector_refdes: None,
            uart_resistor_refdes: None,
            logic_domain: "3V3".to_owned(),
            uart_resistor_value: "1k".to_owned(),
            pins: None,
        }
    }

    pub fn driver(mut self, refdes: impl Into<String>) -> Self {
        self.driver_refdes = Some(refdes.into());
        self
    }

    pub fn motor_connector(mut self, refdes: impl Into<String>) -> Self {
        self.motor_connector_refdes = Some(refdes.into());
        self
    }

    pub fn uart_resistor(mut self, refdes: impl Into<String>) -> Self {
        self.uart_resistor_refdes = Some(refdes.into());
        self
    }

    pub fn logic_domain(mut self, domain: impl Into<String>) -> Self {
        self.logic_domain = domain.into();
        self
    }

    pub fn uart_resistor_value(mut self, value: impl Into<String>) -> Self {
        self.uart_resistor_value = value.into();
        self
    }

    pub fn pins(mut self, pins: Tmc2209UartAxisPins) -> Self {
        self.pins = Some(pins);
        self
    }
}

impl Component for Tmc2209UartAxisSpec {
    type Output = Tmc2209UartAxis;

    fn add_to(self, board: &mut BoardSpec) -> Result<Self::Output> {
        let axis = self.axis;
        let driver_refdes = required(&axis, "driver refdes", self.driver_refdes)?;
        let motor_refdes = required(&axis, "motor connector refdes", self.motor_connector_refdes)?;
        let resistor_refdes = required(&axis, "UART resistor refdes", self.uart_resistor_refdes)?;
        let pins = required(&axis, "control pins", self.pins)?;

        let driver = board.add(silentstepstick_tmc2209_v20(&driver_refdes))?;
        let motor_connector = board.add(xh2p54_motor4(
            &motor_refdes,
            &format!("{axis} motor connector"),
        ))?;
        let uart_tx_resistor = board.add(resistor_0805(
            &resistor_refdes,
            &format!("{} {axis} TMC UART TX series", self.uart_resistor_value),
        ))?;

        board
            .logic(format!("{axis}_EN"), &self.logic_domain)
            .connect_all([pins.enable, driver.enable()]);
        board
            .logic(format!("{axis}_UART_TX"), &self.logic_domain)
            .connect_all([pins.uart_tx, uart_tx_resistor.pin1()]);
        board
            .logic(format!("{axis}_UART"), &self.logic_domain)
            .connect_all([pins.uart_rx, uart_tx_resistor.pin2(), driver.uart()]);
        board
            .logic(format!("{axis}_STEP"), &self.logic_domain)
            .connect_all([pins.step, driver.step()]);
        board
            .logic(format!("{axis}_DIR"), &self.logic_domain)
            .connect_all([pins.dir, driver.dir()]);

        connect_motor_phases(board, &axis, &driver, &motor_connector);

        Ok(Tmc2209UartAxis {
            axis,
            driver,
            motor_connector,
            uart_tx_resistor,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Tmc2209UartAxis {
    axis: String,
    driver: SilentStepStickTmc2209V20,
    motor_connector: Xh2p54Motor4,
    uart_tx_resistor: Resistor2,
}

impl Tmc2209UartAxis {
    pub fn axis(&self) -> &str {
        &self.axis
    }

    pub fn driver(&self) -> &SilentStepStickTmc2209V20 {
        &self.driver
    }

    pub fn motor_connector(&self) -> &Xh2p54Motor4 {
        &self.motor_connector
    }

    pub fn uart_tx_resistor(&self) -> &Resistor2 {
        &self.uart_tx_resistor
    }

    pub fn vmot(&self) -> PinRef {
        self.driver.vmot()
    }

    pub fn vio(&self) -> PinRef {
        self.driver.vio()
    }

    pub fn ground(&self) -> PinRef {
        self.driver.ground()
    }

    pub fn ms1(&self) -> PinRef {
        self.driver.ms1()
    }

    pub fn ms2(&self) -> PinRef {
        self.driver.ms2()
    }
}

fn connect_motor_phases(
    board: &mut BoardSpec,
    axis: &str,
    driver: &SilentStepStickTmc2209V20,
    connector: &Xh2p54Motor4,
) {
    board
        .motor_phase(format!("{axis}_OA1"))
        .connect_all([driver.oa1(), connector.a1()]);
    board
        .motor_phase(format!("{axis}_OA2"))
        .connect_all([driver.oa2(), connector.a2()]);
    board
        .motor_phase(format!("{axis}_OB1"))
        .connect_all([driver.ob1(), connector.b1()]);
    board
        .motor_phase(format!("{axis}_OB2"))
        .connect_all([driver.ob2(), connector.b2()]);
}

fn required<T>(axis: &str, field: &str, value: Option<T>) -> Result<T> {
    value.ok_or_else(|| Error::Io(format!("TMC2209 axis {axis} is missing {field}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use via_core::Board;
    use via_parts_harmonic::esp32_s3_n16r8;

    #[test]
    fn tmc2209_uart_axis_creates_control_and_motor_nets() {
        let mut spec = BoardSpec::new("axis_pattern");
        let esp32 = spec.add(esp32_s3_n16r8("U1")).unwrap();

        spec.add(
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
        )
        .unwrap();

        let board = spec.build().unwrap();
        assert_eq!(
            board.module("R1").unwrap().footprint_name(),
            Some("R_0805_2012Metric")
        );
        assert_net(&board, "X_EN", &[("U1", "GPIO4"), ("U2", "EN")]);
        assert_net(&board, "X_UART_TX", &[("U1", "GPIO5"), ("R1", "1")]);
        assert_net(
            &board,
            "X_UART",
            &[("U1", "GPIO6"), ("R1", "2"), ("U2", "UART")],
        );
        assert_net(&board, "X_STEP", &[("U1", "GPIO7"), ("U2", "STEP")]);
        assert_net(&board, "X_DIR", &[("U1", "GPIO15"), ("U2", "DIR")]);
        assert_net(&board, "X_OA1", &[("U2", "OA1"), ("J2", "A1")]);
        assert_net(&board, "X_OA2", &[("U2", "OA2"), ("J2", "A2")]);
        assert_net(&board, "X_OB1", &[("U2", "OB1"), ("J2", "B1")]);
        assert_net(&board, "X_OB2", &[("U2", "OB2"), ("J2", "B2")]);
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
