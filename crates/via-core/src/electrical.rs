use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ElectricalClass {
    Passive,
    Ground,
    Power(String),
    Logic(String),
    MotorPhase,
}

impl ElectricalClass {
    pub fn power(domain: impl Into<String>) -> Self {
        Self::Power(domain.into())
    }

    pub fn logic(domain: impl Into<String>) -> Self {
        Self::Logic(domain.into())
    }

    pub fn is_compatible_with(&self, other: &Self) -> bool {
        use ElectricalClass::{Ground, Logic, MotorPhase, Passive, Power};

        match (self, other) {
            (Passive, _) | (_, Passive) => true,
            (Ground, Ground) => true,
            (Power(a), Power(b)) => a == b,
            (Logic(a), Logic(b)) => a == b,
            (MotorPhase, MotorPhase) => true,
            _ => false,
        }
    }
}

impl fmt::Display for ElectricalClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ElectricalClass::Passive => f.write_str("passive"),
            ElectricalClass::Ground => f.write_str("ground"),
            ElectricalClass::Power(domain) => write!(f, "power:{domain}"),
            ElectricalClass::Logic(domain) => write!(f, "logic:{domain}"),
            ElectricalClass::MotorPhase => f.write_str("motor-phase"),
        }
    }
}
