use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    DuplicateModule(String),
    Validation(Vec<Diagnostic>),
    Io(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::DuplicateModule(refdes) => write!(f, "duplicate module refdes: {refdes}"),
            Error::Validation(diagnostics) => {
                writeln!(f, "board validation failed:")?;
                for diagnostic in diagnostics {
                    writeln!(f, "- {diagnostic}")?;
                }
                Ok(())
            }
            Error::Io(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io(value.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
}

impl DiagnosticSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagnosticSeverity::Error => "error",
            DiagnosticSeverity::Warning => "warning",
            DiagnosticSeverity::Info => "info",
        }
    }
}

impl fmt::Display for DiagnosticSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectRef {
    Board { name: String },
    Module { refdes: String },
    Pin { refdes: String, pin: String },
    Net { name: String },
    Footprint { name: String },
    Pad { refdes: String, pad: String },
}

impl ObjectRef {
    pub fn board(name: impl Into<String>) -> Self {
        Self::Board { name: name.into() }
    }

    pub fn module(refdes: impl Into<String>) -> Self {
        Self::Module {
            refdes: refdes.into(),
        }
    }

    pub fn pin(refdes: impl Into<String>, pin: impl Into<String>) -> Self {
        Self::Pin {
            refdes: refdes.into(),
            pin: pin.into(),
        }
    }

    pub fn net(name: impl Into<String>) -> Self {
        Self::Net { name: name.into() }
    }

    pub fn footprint(name: impl Into<String>) -> Self {
        Self::Footprint { name: name.into() }
    }

    pub fn pad(refdes: impl Into<String>, pad: impl Into<String>) -> Self {
        Self::Pad {
            refdes: refdes.into(),
            pad: pad.into(),
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            ObjectRef::Board { .. } => "board",
            ObjectRef::Module { .. } => "module",
            ObjectRef::Pin { .. } => "pin",
            ObjectRef::Net { .. } => "net",
            ObjectRef::Footprint { .. } => "footprint",
            ObjectRef::Pad { .. } => "pad",
        }
    }
}

impl fmt::Display for ObjectRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjectRef::Board { name } => write!(f, "board {name}"),
            ObjectRef::Module { refdes } => write!(f, "{refdes}"),
            ObjectRef::Pin { refdes, pin } => write!(f, "{refdes}.{pin}"),
            ObjectRef::Net { name } => write!(f, "net {name}"),
            ObjectRef::Footprint { name } => write!(f, "footprint {name}"),
            ObjectRef::Pad { refdes, pad } => write!(f, "{refdes}.{pad}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub code: Option<String>,
    pub message: String,
    pub object: Option<ObjectRef>,
    pub related: Vec<ObjectRef>,
}

impl Diagnostic {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            code: None,
            message: message.into(),
            object: None,
            related: Vec::new(),
        }
    }

    pub fn coded(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: Some(code.into()),
            ..Self::new(message)
        }
    }

    pub fn warning(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::coded(code, message).with_severity(DiagnosticSeverity::Warning)
    }

    pub fn info(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::coded(code, message).with_severity(DiagnosticSeverity::Info)
    }

    pub fn with_severity(mut self, severity: DiagnosticSeverity) -> Self {
        self.severity = severity;
        self
    }

    pub fn at(mut self, object: ObjectRef) -> Self {
        self.object = Some(object);
        self
    }

    pub fn relates_to(mut self, object: ObjectRef) -> Self {
        self.related.push(object);
        self
    }

    pub fn related_to<I>(mut self, objects: I) -> Self
    where
        I: IntoIterator<Item = ObjectRef>,
    {
        self.related.extend(objects);
        self
    }

    pub fn code(&self) -> Option<&str> {
        self.code.as_deref()
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn severity(&self) -> DiagnosticSeverity {
        self.severity
    }

    pub fn object(&self) -> Option<&ObjectRef> {
        self.object.as_ref()
    }

    pub fn related(&self) -> &[ObjectRef] {
        &self.related
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ", self.severity)?;
        if let Some(code) = &self.code {
            write!(f, "[{code}] ")?;
        }
        if let Some(object) = &self.object {
            write!(f, "{object}: ")?;
        }
        f.write_str(&self.message)
    }
}
