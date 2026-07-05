use via_core::{BoardSpec, Component, Error, PinRef, Result};
use via_parts::{Capacitor2, capacitor_0805, polarized_capacitor_radial_verify};
use via_parts_harmonic::{
    Dc005BarrelJack, Header2, Mp1584BuckAdapter, dc005_barrel_jack, mp1584_buck_adapter,
    pin_header_1x02,
};

#[derive(Debug, Clone)]
pub struct DcBuckInputStageSpec {
    dc_jack_refdes: String,
    buck_refdes: String,
    input_bulk_refdes: String,
    input_hf_refdes: String,
    buck_input_bulk_refdes: String,
    output_bulk_refdes: String,
    output_hf_refdes: String,
    input_rail: String,
    input_domain: String,
    output_rail: String,
    output_domain: String,
    ground_net: String,
    input_loads: Vec<PinRef>,
    output_loads: Vec<PinRef>,
}

impl DcBuckInputStageSpec {
    pub fn new() -> Self {
        Self {
            dc_jack_refdes: "J1".to_owned(),
            buck_refdes: "U4".to_owned(),
            input_bulk_refdes: "C1".to_owned(),
            input_hf_refdes: "C2".to_owned(),
            buck_input_bulk_refdes: "C7".to_owned(),
            output_bulk_refdes: "C8".to_owned(),
            output_hf_refdes: "C9".to_owned(),
            input_rail: "12V_IN".to_owned(),
            input_domain: "12V".to_owned(),
            output_rail: "5V_BUCK".to_owned(),
            output_domain: "5V".to_owned(),
            ground_net: "GND".to_owned(),
            input_loads: Vec::new(),
            output_loads: Vec::new(),
        }
    }

    pub fn dc_jack(mut self, refdes: impl Into<String>) -> Self {
        self.dc_jack_refdes = refdes.into();
        self
    }

    pub fn buck(mut self, refdes: impl Into<String>) -> Self {
        self.buck_refdes = refdes.into();
        self
    }

    pub fn input_bulk(mut self, refdes: impl Into<String>) -> Self {
        self.input_bulk_refdes = refdes.into();
        self
    }

    pub fn input_hf(mut self, refdes: impl Into<String>) -> Self {
        self.input_hf_refdes = refdes.into();
        self
    }

    pub fn buck_input_bulk(mut self, refdes: impl Into<String>) -> Self {
        self.buck_input_bulk_refdes = refdes.into();
        self
    }

    pub fn output_bulk(mut self, refdes: impl Into<String>) -> Self {
        self.output_bulk_refdes = refdes.into();
        self
    }

    pub fn output_hf(mut self, refdes: impl Into<String>) -> Self {
        self.output_hf_refdes = refdes.into();
        self
    }

    pub fn input_rail(mut self, name: impl Into<String>, domain: impl Into<String>) -> Self {
        self.input_rail = name.into();
        self.input_domain = domain.into();
        self
    }

    pub fn output_rail(mut self, name: impl Into<String>, domain: impl Into<String>) -> Self {
        self.output_rail = name.into();
        self.output_domain = domain.into();
        self
    }

    pub fn ground_net(mut self, name: impl Into<String>) -> Self {
        self.ground_net = name.into();
        self
    }

    pub fn input_loads<I>(mut self, pins: I) -> Self
    where
        I: IntoIterator<Item = PinRef>,
    {
        self.input_loads.extend(pins);
        self
    }

    pub fn output_loads<I>(mut self, pins: I) -> Self
    where
        I: IntoIterator<Item = PinRef>,
    {
        self.output_loads.extend(pins);
        self
    }
}

impl Default for DcBuckInputStageSpec {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for DcBuckInputStageSpec {
    type Output = DcBuckInputStage;

    fn add_to(self, board: &mut BoardSpec) -> Result<Self::Output> {
        let dc_jack = board.add(dc005_barrel_jack(&self.dc_jack_refdes))?;
        let buck = board.add(mp1584_buck_adapter(&self.buck_refdes))?;
        let input_bulk = board.add(polarized_capacitor_radial_verify(
            &self.input_bulk_refdes,
            "100uF 25V 12V input bulk VERIFY",
        ))?;
        let input_hf = board.add(capacitor_0805(
            &self.input_hf_refdes,
            "100nF 50V 12V input HF",
        ))?;
        let buck_input_bulk = board.add(polarized_capacitor_radial_verify(
            &self.buck_input_bulk_refdes,
            "47uF 25V MP1584 input bulk VERIFY",
        ))?;
        let output_bulk = board.add(polarized_capacitor_radial_verify(
            &self.output_bulk_refdes,
            "47uF 10V MP1584 output bulk VERIFY",
        ))?;
        let output_hf = board.add(capacitor_0805(
            &self.output_hf_refdes,
            "100nF 50V MP1584 output HF",
        ))?;

        board.ground(&self.ground_net).connect_all([
            dc_jack.sleeve_ground_verify(),
            buck.input_negative(),
            buck.output_negative(),
        ]);
        board
            .rail(&self.input_rail, &self.input_domain)
            .ground_net(&self.ground_net)
            .connect_all(
                [dc_jack.tip_12v(), buck.input_positive()]
                    .into_iter()
                    .chain(self.input_loads),
            )
            .decouple(&input_bulk)
            .decouple(&input_hf)
            .decouple(&buck_input_bulk);
        board
            .rail(&self.output_rail, &self.output_domain)
            .ground_net(&self.ground_net)
            .connect_all(
                [buck.output_positive()]
                    .into_iter()
                    .chain(self.output_loads),
            )
            .decouple(&output_bulk)
            .decouple(&output_hf);

        Ok(DcBuckInputStage {
            input_rail: self.input_rail,
            input_domain: self.input_domain,
            output_rail: self.output_rail,
            output_domain: self.output_domain,
            ground_net: self.ground_net,
            dc_jack,
            buck,
            input_bulk,
            input_hf,
            buck_input_bulk,
            output_bulk,
            output_hf,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DcBuckInputStage {
    input_rail: String,
    input_domain: String,
    output_rail: String,
    output_domain: String,
    ground_net: String,
    dc_jack: Dc005BarrelJack,
    buck: Mp1584BuckAdapter,
    input_bulk: Capacitor2,
    input_hf: Capacitor2,
    buck_input_bulk: Capacitor2,
    output_bulk: Capacitor2,
    output_hf: Capacitor2,
}

impl DcBuckInputStage {
    pub fn input_rail(&self) -> &str {
        &self.input_rail
    }

    pub fn input_domain(&self) -> &str {
        &self.input_domain
    }

    pub fn output_rail(&self) -> &str {
        &self.output_rail
    }

    pub fn output_domain(&self) -> &str {
        &self.output_domain
    }

    pub fn ground_net(&self) -> &str {
        &self.ground_net
    }

    pub fn dc_jack(&self) -> &Dc005BarrelJack {
        &self.dc_jack
    }

    pub fn buck(&self) -> &Mp1584BuckAdapter {
        &self.buck
    }

    pub fn input_bulk(&self) -> &Capacitor2 {
        &self.input_bulk
    }

    pub fn input_hf(&self) -> &Capacitor2 {
        &self.input_hf
    }

    pub fn buck_input_bulk(&self) -> &Capacitor2 {
        &self.buck_input_bulk
    }

    pub fn output_bulk(&self) -> &Capacitor2 {
        &self.output_bulk
    }

    pub fn output_hf(&self) -> &Capacitor2 {
        &self.output_hf
    }
}

#[derive(Debug, Clone)]
pub struct ActiveLowSwitchInputSpec {
    header_refdes: Option<String>,
    value: String,
    net: Option<String>,
    logic_domain: String,
    ground_net: String,
    signal_pin: Option<PinRef>,
}

impl ActiveLowSwitchInputSpec {
    pub fn new(net: impl Into<String>) -> Self {
        Self {
            header_refdes: None,
            value: "active-low switch".to_owned(),
            net: Some(net.into()),
            logic_domain: "3V3".to_owned(),
            ground_net: "GND".to_owned(),
            signal_pin: None,
        }
    }

    pub fn header(mut self, refdes: impl Into<String>) -> Self {
        self.header_refdes = Some(refdes.into());
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    pub fn logic_domain(mut self, domain: impl Into<String>) -> Self {
        self.logic_domain = domain.into();
        self
    }

    pub fn ground_net(mut self, name: impl Into<String>) -> Self {
        self.ground_net = name.into();
        self
    }

    pub fn signal(mut self, pin: PinRef) -> Self {
        self.signal_pin = Some(pin);
        self
    }
}

impl Component for ActiveLowSwitchInputSpec {
    type Output = ActiveLowSwitchInput;

    fn add_to(self, board: &mut BoardSpec) -> Result<Self::Output> {
        let net = self
            .net
            .ok_or_else(|| Error::Io("active-low switch input is missing net".to_owned()))?;
        let header_refdes = self
            .header_refdes
            .ok_or_else(|| Error::Io(format!("{net} switch input is missing header refdes")))?;
        let signal_pin = self
            .signal_pin
            .ok_or_else(|| Error::Io(format!("{net} switch input is missing signal pin")))?;

        let header = board.add(pin_header_1x02(&header_refdes, &self.value))?;

        board
            .logic(&net, &self.logic_domain)
            .connect_all([signal_pin, header.pin1()]);
        board.ground(&self.ground_net).connect(header.pin2());

        Ok(ActiveLowSwitchInput {
            net,
            logic_domain: self.logic_domain,
            ground_net: self.ground_net,
            header,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ActiveLowSwitchInput {
    net: String,
    logic_domain: String,
    ground_net: String,
    header: Header2,
}

impl ActiveLowSwitchInput {
    pub fn net(&self) -> &str {
        &self.net
    }

    pub fn logic_domain(&self) -> &str {
        &self.logic_domain
    }

    pub fn ground_net(&self) -> &str {
        &self.ground_net
    }

    pub fn header(&self) -> &Header2 {
        &self.header
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use via_core::Board;
    use via_parts_harmonic::esp32_s3_n16r8;

    #[test]
    fn dc_buck_input_stage_creates_rails_and_decoupling() {
        let mut spec = BoardSpec::new("power_stage");
        let esp32 = spec.add(esp32_s3_n16r8("U1")).unwrap();

        spec.add(
            DcBuckInputStageSpec::new()
                .input_loads(std::iter::empty())
                .output_loads([esp32.power_5v()]),
        )
        .unwrap();
        spec.ground("GND").connect(esp32.ground());

        let board = spec.build().unwrap();

        assert_net(
            &board,
            "12V_IN",
            &[
                ("J1", "4"),
                ("U4", "IN+"),
                ("C1", "1"),
                ("C2", "1"),
                ("C7", "1"),
            ],
        );
        assert_net(
            &board,
            "5V_BUCK",
            &[("U4", "OUT+"), ("U1", "5VIN"), ("C8", "1"), ("C9", "1")],
        );
    }

    #[test]
    fn active_low_switch_input_wires_signal_and_ground() {
        let mut spec = BoardSpec::new("switch_input");
        let esp32 = spec.add(esp32_s3_n16r8("U1")).unwrap();

        spec.add(
            ActiveLowSwitchInputSpec::new("ESTOP_SW")
                .header("J6")
                .value("E-stop switch active-low")
                .signal(esp32.gpio39()),
        )
        .unwrap();
        spec.ground("GND").connect(esp32.ground());

        let board = spec.build().unwrap();

        assert_net(&board, "ESTOP_SW", &[("U1", "GPIO39"), ("J6", "1")]);
        assert_net(&board, "GND", &[("J6", "2"), ("U1", "GND")]);
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
