use via_core::{Board, Design, NetHandle, PinRef, Result, Voltage};

pub trait DesignExt {
    fn rail(&mut self, name: impl Into<String>) -> RailBuilder<'_>;
    fn signal(&mut self, name: impl Into<String>, domain: impl Into<String>) -> NetHandle;
    fn connect<I>(&mut self, net: &NetHandle, pins: I) -> &mut Self
    where
        I: IntoIterator<Item = PinRef>;
    fn connect_named<I>(&mut self, name: impl Into<String>, pins: I) -> &mut Self
    where
        I: IntoIterator<Item = PinRef>;
    fn finish(self) -> Result<Board>;
}

pub struct RailBuilder<'a> {
    design: &'a mut Design,
    name: String,
}

impl<'a> RailBuilder<'a> {
    pub(crate) fn new(design: &'a mut Design, name: impl Into<String>) -> Self {
        Self {
            design,
            name: name.into(),
        }
    }

    pub fn dc(self, volts: f64) -> NetHandle {
        self.design.power(self.name, Voltage::dc(volts))
    }

    pub fn domain(self, domain: impl Into<String>) -> NetHandle {
        self.design.power_domain(self.name, domain)
    }
}

impl DesignExt for Design {
    fn rail(&mut self, name: impl Into<String>) -> RailBuilder<'_> {
        RailBuilder::new(self, name)
    }

    fn signal(&mut self, name: impl Into<String>, domain: impl Into<String>) -> NetHandle {
        self.logic(name, domain)
    }

    fn connect<I>(&mut self, net: &NetHandle, pins: I) -> &mut Self
    where
        I: IntoIterator<Item = PinRef>,
    {
        net.connect_all(self, pins);
        self
    }

    fn connect_named<I>(&mut self, name: impl Into<String>, pins: I) -> &mut Self
    where
        I: IntoIterator<Item = PinRef>,
    {
        self.net(name).connect_all(self, pins);
        self
    }

    fn finish(self) -> Result<Board> {
        self.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use via_core::{Unit, model::Part};

    #[test]
    fn design_ext_connects_named_and_classed_nets() {
        let mut design = Design::new("design_ext").units(Unit::Mm);
        let v3v3 = design.rail("3V3").dc(3.3);
        let signal = design.signal("GPIO", "3V3");

        let a = design
            .add_part(
                Part::new("J1", "header")
                    .footprint("J")
                    .pins(["1", "2"])
                    .power_pin("1", "3V3")
                    .logic_pin("2", "3V3"),
            )
            .unwrap();
        let b = design
            .add_part(
                Part::new("U1", "load")
                    .footprint("U")
                    .pins(["VCC", "IN"])
                    .power_pin("VCC", "3V3")
                    .logic_pin("IN", "3V3"),
            )
            .unwrap();

        design.connect(&v3v3, [a.pin("1"), b.pin("VCC")]);
        design.connect(&signal, [a.pin("2"), b.pin("IN")]);

        let board = design.finish().unwrap();
        assert_eq!(board.nets().count(), 2);
    }
}
