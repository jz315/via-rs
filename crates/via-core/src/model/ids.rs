use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModuleId {
    pub(crate) refdes: String,
}

/// Preferred name for a board component identifier.
///
/// The historical `ModuleId` spelling remains available for source
/// compatibility.
pub type PartId = ModuleId;

impl ModuleId {
    pub(crate) fn new(refdes: impl Into<String>) -> Self {
        Self {
            refdes: refdes.into(),
        }
    }

    pub fn refdes(&self) -> &str {
        &self.refdes
    }

    pub fn pin(&self, pin: impl Into<String>) -> PinRef {
        PinRef {
            module: self.refdes.clone(),
            pin: pin.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinRef {
    pub module: String,
    pub pin: String,
}

impl fmt::Display for PinRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.module, self.pin)
    }
}
