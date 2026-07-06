use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Resistance {
    value: f64,
    unit: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Capacitance {
    value: f64,
    unit: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RatedVoltage {
    value: f64,
}

pub trait QuantityExt {
    fn ohm(self) -> Resistance;
    fn kohm(self) -> Resistance;
    fn mohm(self) -> Resistance;
    fn pf(self) -> Capacitance;
    fn nf(self) -> Capacitance;
    fn uf(self) -> Capacitance;
    fn v(self) -> RatedVoltage;
}

impl Resistance {
    pub fn new(value: f64, unit: &'static str) -> Self {
        Self { value, unit }
    }
}

impl Capacitance {
    pub fn new(value: f64, unit: &'static str) -> Self {
        Self { value, unit }
    }
}

impl RatedVoltage {
    pub fn new(value: f64) -> Self {
        Self { value }
    }

    pub fn volts(&self) -> f64 {
        self.value
    }
}

impl fmt::Display for Resistance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", format_number(self.value), self.unit)
    }
}

impl fmt::Display for Capacitance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", format_number(self.value), self.unit)
    }
}

impl fmt::Display for RatedVoltage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}V", format_number(self.value))
    }
}

macro_rules! impl_quantity_ext {
    ($($ty:ty),* $(,)?) => {
        $(
            impl QuantityExt for $ty {
                fn ohm(self) -> Resistance {
                    Resistance::new(self as f64, "R")
                }

                fn kohm(self) -> Resistance {
                    Resistance::new(self as f64, "k")
                }

                fn mohm(self) -> Resistance {
                    Resistance::new(self as f64, "M")
                }

                fn pf(self) -> Capacitance {
                    Capacitance::new(self as f64, "pF")
                }

                fn nf(self) -> Capacitance {
                    Capacitance::new(self as f64, "nF")
                }

                fn uf(self) -> Capacitance {
                    Capacitance::new(self as f64, "uF")
                }

                fn v(self) -> RatedVoltage {
                    RatedVoltage::new(self as f64)
                }
            }
        )*
    };
}

impl_quantity_ext!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize, f32, f64);

fn format_number(value: f64) -> String {
    let rounded = value.round();
    if (value - rounded).abs() < 0.000_001 {
        return format!("{rounded:.0}");
    }

    let mut text = format!("{value:.3}");
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_common_quantities_for_part_values() {
        assert_eq!(1.kohm().to_string(), "1k");
        assert_eq!(4.7.kohm().to_string(), "4.7k");
        assert_eq!(100.nf().to_string(), "100nF");
        assert_eq!(10.uf().to_string(), "10uF");
        assert_eq!(25.v().to_string(), "25V");
    }
}
