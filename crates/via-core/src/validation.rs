//! Validation reports and policies shared by authoring APIs and exporters.

use crate::{Diagnostic, DiagnosticSeverity, Error, Result};

/// The policy used when deciding which design diagnostics block an operation.
///
/// `Draft` is intentionally permissive: incomplete nets and missing physical
/// metadata remain visible as warnings so a design can be developed in small
/// increments. `Prototype` requires a structurally complete circuit.
/// `Production` adds sourcing and physical-verification requirements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationProfile {
    /// Validate identity, pin, pad, and electrical consistency while allowing
    /// incomplete draft wiring as warnings.
    Draft,
    /// Require a structurally complete and electrically consistent prototype.
    Prototype,
    /// Require prototype correctness plus sourcing and verification metadata.
    Production,
}

/// A complete, ordered set of diagnostics produced by a validation pass.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValidationReport {
    diagnostics: Vec<Diagnostic>,
}

impl ValidationReport {
    /// Creates a report from diagnostics already ordered by the validator.
    pub fn new(diagnostics: Vec<Diagnostic>) -> Self {
        Self { diagnostics }
    }

    /// Returns every diagnostic, including warnings and informational notes.
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Returns diagnostics with error severity.
    pub fn errors(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity() == DiagnosticSeverity::Error)
    }

    /// Returns diagnostics with warning severity.
    pub fn warnings(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity() == DiagnosticSeverity::Warning)
    }

    /// Returns whether the report has no diagnostics at all.
    pub fn is_clean(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Returns whether an operation governed by this report may proceed.
    pub fn has_errors(&self) -> bool {
        self.errors().next().is_some()
    }

    /// Converts the report into the crate's conventional result type.
    pub fn into_result(self) -> Result<()> {
        if self.has_errors() {
            Err(Error::Validation(self.diagnostics))
        } else {
            Ok(())
        }
    }
}
