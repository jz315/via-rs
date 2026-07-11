use crate::error::Result;
use crate::export::Exporter;
use crate::footprint::FootprintPads;
use crate::model::{Board, ModuleId, Net, Part, PinRef};
use crate::rules::BoardRules;
use crate::spec::{BoardSpec, Component, DecouplerPins};
use crate::{ValidationProfile, ValidationReport};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Mm,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Voltage {
    volts: f64,
}

impl Voltage {
    pub fn dc(volts: f64) -> Self {
        Self { volts }
    }

    pub fn volts(&self) -> f64 {
        self.volts
    }

    pub fn domain(&self) -> String {
        let rounded = self.volts.round();
        if (self.volts - rounded).abs() < 0.000_001 {
            return format!("{rounded:.0}V");
        }

        let mut text = format!("{:.3}", self.volts);
        while text.contains('.') && text.ends_with('0') {
            text.pop();
        }
        text.replace('.', "V")
    }
}

impl From<f64> for Voltage {
    fn from(volts: f64) -> Self {
        Self::dc(volts)
    }
}

/// Backwards-compatible name for [`ValidationProfile`].
///
/// New code should use `ValidationProfile`, which communicates that a profile
/// controls both diagnostics and whether an operation may proceed.
pub type CheckProfile = ValidationProfile;

#[derive(Debug, Clone, PartialEq)]
pub struct Design {
    spec: BoardSpec,
    unit: Unit,
}

impl Design {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            spec: BoardSpec::new(name),
            unit: Unit::Mm,
        }
    }

    pub fn units(mut self, unit: Unit) -> Self {
        self.unit = unit;
        self
    }

    pub fn unit(&self) -> Unit {
        self.unit
    }

    pub fn rules(mut self, rules: BoardRules) -> Self {
        *self.spec.rules_mut() = rules;
        self
    }

    pub fn rules_ref(&self) -> &BoardRules {
        self.spec.rules()
    }

    pub fn rules_mut(&mut self) -> &mut BoardRules {
        self.spec.rules_mut()
    }

    pub fn add<C>(&mut self, component: C) -> Result<C::Output>
    where
        C: Component,
    {
        component.add_to(self)
    }

    pub fn add_part(&mut self, part: Part) -> Result<ModuleId> {
        self.spec.add_part(part)
    }

    pub fn add_footprint_pads(&mut self, footprint: FootprintPads) -> &mut Self {
        self.spec.add_footprint_pads(footprint);
        self
    }

    pub fn net(&mut self, name: impl Into<String>) -> NetHandle {
        let name = name.into();
        self.spec.net(name.clone());
        NetHandle::new(name, NetKind::Plain)
    }

    pub fn ground(&mut self, name: impl Into<String>) -> NetHandle {
        let name = name.into();
        self.spec.ground(name.clone());
        NetHandle::new(name, NetKind::Ground)
    }

    pub fn power(&mut self, name: impl Into<String>, voltage: impl Into<Voltage>) -> NetHandle {
        self.power_domain(name, voltage.into().domain())
    }

    pub fn power_domain(
        &mut self,
        name: impl Into<String>,
        domain: impl Into<String>,
    ) -> NetHandle {
        let name = name.into();
        let domain = domain.into();
        self.spec.power(name.clone(), domain.clone());
        NetHandle::new(name, NetKind::Power { domain })
    }

    pub fn logic(&mut self, name: impl Into<String>, domain: impl Into<String>) -> NetHandle {
        let name = name.into();
        let domain = domain.into();
        self.spec.logic(name.clone(), domain.clone());
        NetHandle::new(name, NetKind::Logic { domain })
    }

    pub fn motor_phase(&mut self, name: impl Into<String>) -> NetHandle {
        let name = name.into();
        self.spec.motor_phase(name.clone());
        NetHandle::new(name, NetKind::MotorPhase)
    }

    /// Connects pins to an existing logical net.
    ///
    /// Connection validity is reported by [`Self::validate`] or
    /// [`Self::finish`], allowing authors to assemble a design incrementally.
    pub fn connect<I>(&mut self, net: &NetHandle, pins: I) -> &mut Self
    where
        I: IntoIterator<Item = PinRef>,
    {
        net.connect_all(self, pins);
        self
    }

    /// Creates or reuses a plain net and connects pins to it.
    pub fn connect_named<I>(&mut self, name: impl Into<String>, pins: I) -> &mut Self
    where
        I: IntoIterator<Item = PinRef>,
    {
        self.net(name).connect_all(self, pins);
        self
    }

    /// Validates the design without consuming it.
    pub fn validate(&self, profile: ValidationProfile) -> ValidationReport {
        self.spec.board().validation_report(profile)
    }

    /// Validates the design and returns an error only when the report contains
    /// diagnostics with error severity.
    pub fn check(&self, profile: ValidationProfile) -> Result<()> {
        self.validate(profile).into_result()
    }

    pub fn board(&self) -> &Board {
        self.spec.board()
    }

    pub fn board_mut(&mut self) -> &mut Board {
        self.spec.board_mut()
    }

    pub fn build(self) -> Result<Board> {
        self.spec.build()
    }

    /// Completes a design using the requested validation policy.
    ///
    /// This is the preferred completion API. It validates exactly once, then
    /// returns the immutable board used by exporters and tests.
    pub fn finish(self, profile: ValidationProfile) -> Result<Board> {
        self.validate(profile).into_result()?;
        Ok(self.spec.into_unchecked_board())
    }

    pub fn to_checked_board(&self) -> Result<Board> {
        self.spec.clone().build()
    }

    pub fn export<E>(&self, exporter: E) -> Result<E::Output>
    where
        E: Exporter,
    {
        let board = self.to_checked_board()?;
        exporter.export_board(&board)
    }

    pub fn into_unchecked_board(self) -> Board {
        self.spec.into_unchecked_board()
    }

    fn net_mut_for_handle(&mut self, handle: &NetHandle) -> &mut Net {
        match &handle.kind {
            NetKind::Plain => self.spec.net(handle.name.clone()),
            NetKind::Ground => self.spec.ground(handle.name.clone()),
            NetKind::Power { domain } => self.spec.power(handle.name.clone(), domain.clone()),
            NetKind::Logic { domain } => self.spec.logic(handle.name.clone(), domain.clone()),
            NetKind::MotorPhase => self.spec.motor_phase(handle.name.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetHandle {
    name: String,
    kind: NetKind,
}

impl NetHandle {
    fn new(name: String, kind: NetKind) -> Self {
        Self { name, kind }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn connect(&self, design: &mut Design, pin: PinRef) -> &Self {
        design.net_mut_for_handle(self).connect(pin);
        self
    }

    pub fn connect_all<I>(&self, design: &mut Design, pins: I) -> &Self
    where
        I: IntoIterator<Item = PinRef>,
    {
        design.net_mut_for_handle(self).connect_all(pins);
        self
    }

    pub fn decouple<D>(&self, design: &mut Design, decoupler: D) -> &Self
    where
        D: DecouplerPins,
    {
        self.decouple_to(design, "GND", decoupler)
    }

    pub fn decouple_to<D>(
        &self,
        design: &mut Design,
        ground_net: impl Into<String>,
        decoupler: D,
    ) -> &Self
    where
        D: DecouplerPins,
    {
        let (power, ground) = decoupler.into_decoupler_pins();
        self.connect(design, power);
        let ground_handle = design.ground(ground_net);
        ground_handle.connect(design, ground);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum NetKind {
    Plain,
    Ground,
    Power { domain: String },
    Logic { domain: String },
    MotorPhase,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Part;

    #[test]
    fn voltage_domain_formats_common_power_rails() {
        assert_eq!(Voltage::dc(12.0).domain(), "12V");
        assert_eq!(Voltage::dc(5.0).domain(), "5V");
        assert_eq!(Voltage::dc(3.3).domain(), "3V3");
    }

    #[test]
    fn design_handles_allow_multiple_named_rails() {
        let mut design = Design::new("modern_api_smoke");
        let signal = design.net("SIGNAL");
        let ground = design.ground("GND");
        let power = design.power("3V3", Voltage::dc(3.3));

        let header = design
            .add_part(
                Part::new("J1", "header")
                    .footprint("Header_1x03")
                    .pins(["1", "2", "3"])
                    .logic_pin("1", "3V3")
                    .power_pin("2", "3V3")
                    .ground_pin("3"),
            )
            .unwrap();
        let load = design
            .add_part(
                Part::new("U1", "load")
                    .footprint("Load")
                    .pins(["SIG", "VCC", "GND"])
                    .logic_pin("SIG", "3V3")
                    .power_pin("VCC", "3V3")
                    .ground_pin("GND"),
            )
            .unwrap();

        signal.connect_all(&mut design, [header.pin("1"), load.pin("SIG")]);
        power.connect_all(&mut design, [header.pin("2"), load.pin("VCC")]);
        ground.connect_all(&mut design, [header.pin("3"), load.pin("GND")]);

        let board = design.build().unwrap();
        assert_eq!(board.name(), "modern_api_smoke");
        assert_eq!(board.nets().count(), 3);
    }

    #[test]
    fn net_handle_decouples_to_default_ground() {
        let mut design = Design::new("modern_decouple");
        let power = design.power("3V3", Voltage::dc(3.3));
        let load = design
            .add_part(
                Part::new("U1", "load")
                    .footprint("Load")
                    .pins(["VCC", "GND"])
                    .power_pin("VCC", "3V3")
                    .ground_pin("GND"),
            )
            .unwrap();
        let cap = design
            .add_part(Part::new("C1", "100nF").footprint("C").pins(["1", "2"]))
            .unwrap();

        power
            .connect(&mut design, load.pin("VCC"))
            .decouple(&mut design, (cap.pin("1"), cap.pin("2")));
        let ground = design.ground("GND");
        ground.connect(&mut design, load.pin("GND"));

        let board = design.build().unwrap();
        let power_net = board.nets().find(|net| net.name() == "3V3").unwrap();
        let ground_net = board.nets().find(|net| net.name() == "GND").unwrap();
        assert_eq!(power_net.connections().len(), 2);
        assert_eq!(ground_net.connections().len(), 2);
    }

    #[test]
    fn design_exports_through_exporter_boundary() {
        struct NamesExporter;

        impl Exporter for NamesExporter {
            type Output = String;

            fn export_board(&self, board: &Board) -> Result<Self::Output> {
                Ok(format!(
                    "{}:{}:{}",
                    board.name(),
                    board.modules().count(),
                    board.nets().count()
                ))
            }
        }

        let mut design = Design::new("exportable");
        let net = design.net("N");
        let a = design
            .add_part(Part::new("J1", "a").footprint("J").pins(["1"]))
            .unwrap();
        let b = design
            .add_part(Part::new("J2", "b").footprint("J").pins(["1"]))
            .unwrap();
        net.connect_all(&mut design, [a.pin("1"), b.pin("1")]);

        assert_eq!(design.export(NamesExporter).unwrap(), "exportable:2:1");
    }

    #[test]
    fn draft_finish_keeps_incomplete_nets_visible_as_warnings() {
        let mut design = Design::new("draft");
        design.net("UNFINISHED");

        let report = design.validate(ValidationProfile::Draft);
        assert!(!report.has_errors());
        assert_eq!(report.warnings().count(), 1);
        assert!(design.finish(ValidationProfile::Draft).is_ok());

        let mut prototype = Design::new("prototype");
        prototype.net("UNFINISHED");
        assert!(prototype.finish(ValidationProfile::Prototype).is_err());
    }
}
