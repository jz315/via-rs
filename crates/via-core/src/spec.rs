use crate::error::Result;
use crate::footprint::FootprintPads;
use crate::model::{Board, ModuleId, Net, Part, PinRef, PinSpec};
use crate::rules::BoardRules;

pub trait Component {
    type Output;

    fn add_to(self, spec: &mut BoardSpec) -> Result<Self::Output>;
}

pub trait DecouplerPins {
    fn into_decoupler_pins(self) -> (PinRef, PinRef);
}

impl DecouplerPins for (PinRef, PinRef) {
    fn into_decoupler_pins(self) -> (PinRef, PinRef) {
        self
    }
}

impl DecouplerPins for [PinRef; 2] {
    fn into_decoupler_pins(self) -> (PinRef, PinRef) {
        let [power, ground] = self;
        (power, ground)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoardSpec {
    board: Board,
}

pub struct PowerRail<'a> {
    spec: &'a mut BoardSpec,
    name: String,
    domain: String,
    ground: String,
}

pub struct PartSpec<T, F>
where
    F: FnOnce(ModuleId) -> T,
{
    part: Part,
    output: F,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartSpecBuilder {
    part: Part,
}

pub fn part(refdes: impl Into<String>, value: impl Into<String>) -> PartSpecBuilder {
    PartSpecBuilder::new(refdes, value)
}

impl<T, F> PartSpec<T, F>
where
    F: FnOnce(ModuleId) -> T,
{
    pub fn new(part: Part, output: F) -> Self {
        Self { part, output }
    }

    pub fn part(&self) -> &Part {
        &self.part
    }
}

impl PartSpecBuilder {
    pub fn new(refdes: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            part: Part::new(refdes, value),
        }
    }

    pub fn from_part(part: Part) -> Self {
        Self { part }
    }

    pub fn footprint(mut self, footprint: impl Into<String>) -> Self {
        self.part = self.part.footprint(footprint);
        self
    }

    pub fn pins<const N: usize>(mut self, pins: [&str; N]) -> Self {
        self.part = self.part.pins(pins);
        self
    }

    pub fn pin(mut self, pin: PinSpec) -> Self {
        self.part = self.part.pin(pin);
        self
    }

    pub fn pin_specs<I>(mut self, pins: I) -> Self
    where
        I: IntoIterator<Item = PinSpec>,
    {
        self.part = self.part.pin_specs(pins);
        self
    }

    pub fn pinmap<const N: usize>(mut self, mappings: [(&str, &str); N]) -> Self {
        self.part = self.part.pinmap(mappings);
        self
    }

    pub fn verify(mut self) -> Self {
        self.part = self.part.verify();
        self
    }

    pub fn mpn(mut self, part_number: impl Into<String>) -> Self {
        self.part = self.part.mpn(part_number);
        self
    }

    pub fn supplier_part(
        mut self,
        supplier: impl Into<String>,
        part_number: impl Into<String>,
    ) -> Self {
        self.part = self.part.supplier_part(supplier, part_number);
        self
    }

    pub fn lcsc(mut self, part_number: impl Into<String>) -> Self {
        self.part = self.part.lcsc(part_number);
        self
    }

    pub fn production_note(mut self, note: impl Into<String>) -> Self {
        self.part = self.part.production_note(note);
        self
    }

    pub fn handle<T, F>(self, output: F) -> PartSpec<T, F>
    where
        F: FnOnce(ModuleId) -> T,
    {
        PartSpec::new(self.part, output)
    }

    pub fn untyped(self) -> Part {
        self.part
    }
}

impl BoardSpec {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            board: Board::new(name),
        }
    }

    pub fn name(&self) -> &str {
        self.board.name()
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn board_mut(&mut self) -> &mut Board {
        &mut self.board
    }

    pub fn rules(&self) -> &BoardRules {
        self.board.rules()
    }

    pub fn rules_mut(&mut self) -> &mut BoardRules {
        self.board.rules_mut()
    }

    pub fn add<C>(&mut self, component: C) -> Result<C::Output>
    where
        C: Component,
    {
        component.add_to(self)
    }

    pub fn add_part(&mut self, part: Part) -> Result<ModuleId> {
        self.board.add_module(part)
    }

    pub fn add_footprint_pads(&mut self, footprint: FootprintPads) {
        self.board.add_footprint_pads(footprint);
    }

    pub fn net(&mut self, name: impl Into<String>) -> &mut Net {
        self.board.net(name)
    }

    pub fn power(&mut self, name: impl Into<String>, domain: impl Into<String>) -> &mut Net {
        self.net(name).power(domain)
    }

    pub fn rail(&mut self, name: impl Into<String>, domain: impl Into<String>) -> PowerRail<'_> {
        PowerRail {
            spec: self,
            name: name.into(),
            domain: domain.into(),
            ground: "GND".to_owned(),
        }
    }

    pub fn logic(&mut self, name: impl Into<String>, domain: impl Into<String>) -> &mut Net {
        self.net(name).logic(domain)
    }

    pub fn ground(&mut self, name: impl Into<String>) -> &mut Net {
        self.net(name).ground()
    }

    pub fn motor_phase(&mut self, name: impl Into<String>) -> &mut Net {
        self.net(name).motor_phase()
    }

    pub fn build(self) -> Result<Board> {
        self.board.check()?;
        Ok(self.board)
    }

    pub fn into_unchecked_board(self) -> Board {
        self.board
    }
}

impl Component for Part {
    type Output = ModuleId;

    fn add_to(self, spec: &mut BoardSpec) -> Result<Self::Output> {
        spec.add_part(self)
    }
}

impl Component for PartSpecBuilder {
    type Output = ModuleId;

    fn add_to(self, spec: &mut BoardSpec) -> Result<Self::Output> {
        spec.add_part(self.part)
    }
}

impl<T, F> Component for PartSpec<T, F>
where
    F: FnOnce(ModuleId) -> T,
{
    type Output = T;

    fn add_to(self, spec: &mut BoardSpec) -> Result<Self::Output> {
        let id = spec.add_part(self.part)?;
        Ok((self.output)(id))
    }
}

impl PowerRail<'_> {
    pub fn ground_net(mut self, name: impl Into<String>) -> Self {
        self.ground = name.into();
        self
    }

    pub fn connect(&mut self, pin: PinRef) -> &mut Self {
        self.spec
            .power(self.name.clone(), self.domain.clone())
            .connect(pin);
        self
    }

    pub fn connect_all<I>(&mut self, pins: I) -> &mut Self
    where
        I: IntoIterator<Item = PinRef>,
    {
        self.spec
            .power(self.name.clone(), self.domain.clone())
            .connect_all(pins);
        self
    }

    pub fn decouple<D>(&mut self, decoupler: D) -> &mut Self
    where
        D: DecouplerPins,
    {
        let (power, ground) = decoupler.into_decoupler_pins();
        self.spec
            .power(self.name.clone(), self.domain.clone())
            .connect(power);
        self.spec.ground(self.ground.clone()).connect(ground);
        self
    }

    pub fn decouple_to<D>(&mut self, ground_net: impl Into<String>, decoupler: D) -> &mut Self
    where
        D: DecouplerPins,
    {
        let (power, ground) = decoupler.into_decoupler_pins();
        self.spec
            .power(self.name.clone(), self.domain.clone())
            .connect(power);
        self.spec.ground(ground_net).connect(ground);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::pin;

    #[derive(Debug, Clone)]
    struct Sensor {
        id: ModuleId,
    }

    impl Sensor {
        fn vcc(&self) -> PinRef {
            self.id.pin("VCC")
        }
    }

    #[test]
    fn part_builder_creates_typed_part_specs() {
        let mut spec = BoardSpec::new("typed_builder");
        let header = spec
            .add(
                part("J1", "power input")
                    .footprint("Header_1x02")
                    .pin(pin("3V3").power("3V3"))
                    .pin(pin("GND").ground()),
            )
            .unwrap();
        let sensor = spec
            .add(
                part("U1", "sensor")
                    .footprint("Sensor_1x02")
                    .pin(pin("VCC").power("3V3").pad("1"))
                    .pin(pin("GND").ground().pad("2"))
                    .handle(|id| Sensor { id }),
            )
            .unwrap();

        spec.power("3V3", "3V3")
            .connect_all([header.pin("3V3"), sensor.vcc()]);
        let board = spec.build().unwrap();
        let module = board.module("U1").unwrap();

        assert_eq!(module.footprint_name(), Some("Sensor_1x02"));
        assert_eq!(
            module.pads_for_pin("VCC"),
            std::collections::BTreeSet::from(["1".to_owned()])
        );
        assert_eq!(board.nets().count(), 1);
    }

    #[test]
    fn part_builder_can_be_added_without_a_typed_handle() {
        let mut spec = BoardSpec::new("untyped_builder");
        let header = spec
            .add(
                part("J1", "header")
                    .footprint("Header_1x02")
                    .pin(pin("1").logic("3V3"))
                    .pin(pin("2").ground()),
            )
            .unwrap();
        let load = spec
            .add(
                part("U1", "load")
                    .footprint("Load_2Pin")
                    .pin(pin("IN").logic("3V3"))
                    .pin(pin("GND").ground()),
            )
            .unwrap();

        spec.logic("SIGNAL", "3V3")
            .connect_all([header.pin("1"), load.pin("IN")]);
        spec.ground("GND")
            .connect_all([header.pin("2"), load.pin("GND")]);

        let board = spec.build().unwrap();
        assert_eq!(board.module("J1").unwrap().value(), "header");
    }
}
