use std::io::{self, IsTerminal, Write};

use clap::ValueEnum;
use via_core::{
    Diagnostic, DiagnosticDefinition, DiagnosticSeverity, Error, ObjectRef,
    all_diagnostic_definitions, diagnostic_definition,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ColorChoice {
    Auto,
    Always,
    Never,
}

impl ColorChoice {
    fn enabled_for_stderr(self) -> bool {
        match self {
            ColorChoice::Auto => io::stderr().is_terminal(),
            ColorChoice::Always => true,
            ColorChoice::Never => false,
        }
    }
}

pub fn write_error(error: &Error, color: ColorChoice, writer: &mut impl Write) -> io::Result<()> {
    match error {
        Error::Validation(diagnostics) => write_diagnostics(diagnostics, color, writer),
        Error::Diagnostic(diagnostic) => write_diagnostic(diagnostic, color, writer),
        Error::DuplicateModule(refdes) => {
            let diagnostic = Diagnostic::coded(
                "part.duplicate_refdes",
                format!("duplicate module refdes: {refdes}"),
            )
            .at(ObjectRef::module(refdes.as_str()));
            write_diagnostic(&diagnostic, color, writer)
        }
        Error::Io(message) => write_diagnostic(&Diagnostic::new(message.as_str()), color, writer),
    }
}

pub fn write_diagnostics(
    diagnostics: &[Diagnostic],
    color: ColorChoice,
    writer: &mut impl Write,
) -> io::Result<()> {
    for (idx, diagnostic) in diagnostics.iter().enumerate() {
        if idx > 0 {
            writeln!(writer)?;
        }
        write!(writer, "{}", render_diagnostic(diagnostic, color))?;
    }
    Ok(())
}

pub fn write_diagnostic(
    diagnostic: &Diagnostic,
    color: ColorChoice,
    writer: &mut impl Write,
) -> io::Result<()> {
    write!(writer, "{}", render_diagnostic(diagnostic, color))
}

pub fn render_diagnostic(diagnostic: &Diagnostic, color: ColorChoice) -> String {
    let enabled = color.enabled_for_stderr();
    let severity = diagnostic.severity();
    let code = diagnostic.code();
    let definition = code.and_then(diagnostic_definition);
    let title = definition
        .map(|definition| definition.title)
        .unwrap_or_else(|| diagnostic.message());

    let mut out = String::new();
    out.push_str(&paint(
        enabled,
        severity_style(severity),
        &severity_header(severity, code),
    ));
    out.push_str(": ");
    out.push_str(title);
    out.push('\n');

    if definition.is_some() {
        push_prefixed_lines(&mut out, "  = ", "    ", diagnostic.message());
    }

    if let Some(object) = diagnostic.object() {
        out.push_str(" --> ");
        out.push_str(&object_label(object));
        out.push('\n');
    }

    for related in diagnostic.related() {
        push_note(
            &mut out,
            enabled,
            "note",
            &format!("related {}", object_label(related)),
        );
    }

    if let Some(definition) = definition {
        for help in definition.help {
            push_note(&mut out, enabled, "help", help);
        }
        if let Some(code) = code {
            push_note(
                &mut out,
                enabled,
                "note",
                &format!("run `via explain {code}` for more detail"),
            );
        }
    }

    out
}

pub fn explain_text(definition: &DiagnosticDefinition) -> String {
    let mut out = String::new();
    out.push_str(definition.code);
    out.push_str(": ");
    out.push_str(definition.title);
    out.push_str("\n\n");
    out.push_str(definition.explanation);
    out.push_str("\n\n");

    if !definition.causes.is_empty() {
        out.push_str("Common causes:\n");
        for cause in definition.causes {
            out.push_str("- ");
            out.push_str(cause);
            out.push('\n');
        }
        out.push('\n');
    }

    if !definition.help.is_empty() {
        out.push_str("Help:\n");
        for help in definition.help {
            out.push_str("- ");
            out.push_str(help);
            out.push('\n');
        }
    }

    out
}

pub fn explain_list_text() -> String {
    let mut out = String::new();
    for definition in all_diagnostic_definitions() {
        out.push_str(definition.code);
        out.push_str(" - ");
        out.push_str(definition.title);
        out.push('\n');
    }
    out
}

pub fn unknown_code_diagnostic(code: &str) -> Diagnostic {
    let suggestions = diagnostic_code_suggestions(code);
    let suffix = if suggestions.is_empty() {
        String::new()
    } else {
        format!("; similar codes: {}", suggestions.join(", "))
    };
    Diagnostic::coded(
        "diagnostic.unknown_code",
        format!("unknown diagnostic code `{code}`{suffix}"),
    )
}

fn diagnostic_code_suggestions(code: &str) -> Vec<&'static str> {
    let prefix = code.split('.').next().unwrap_or(code);
    all_diagnostic_definitions()
        .iter()
        .filter(|definition| definition.code.starts_with(prefix) || definition.code.contains(code))
        .map(|definition| definition.code)
        .take(8)
        .collect()
}

fn push_prefixed_lines(out: &mut String, first_prefix: &str, next_prefix: &str, text: &str) {
    let mut lines = text.lines();
    if let Some(first) = lines.next() {
        out.push_str(first_prefix);
        out.push_str(first);
        out.push('\n');
    }
    for line in lines {
        out.push_str(next_prefix);
        out.push_str(line);
        out.push('\n');
    }
}

fn push_note(out: &mut String, color: bool, label: &str, message: &str) {
    out.push_str(&paint(color, "\x1b[1m", label));
    out.push_str(": ");
    out.push_str(message);
    out.push('\n');
}

fn severity_header(severity: DiagnosticSeverity, code: Option<&str>) -> String {
    match code {
        Some(code) => format!("{severity}[{code}]"),
        None => severity.to_string(),
    }
}

fn severity_style(severity: DiagnosticSeverity) -> &'static str {
    match severity {
        DiagnosticSeverity::Error => "\x1b[1;31m",
        DiagnosticSeverity::Warning => "\x1b[1;33m",
        DiagnosticSeverity::Info => "\x1b[1;36m",
    }
}

fn paint(enabled: bool, style: &'static str, text: &str) -> String {
    if enabled {
        format!("{style}{text}\x1b[0m")
    } else {
        text.to_owned()
    }
}

fn object_label(object: &ObjectRef) -> String {
    match object {
        ObjectRef::Board { name } => format!("board {name}"),
        ObjectRef::Module { refdes } => format!("module {refdes}"),
        ObjectRef::Pin { refdes, pin } => format!("pin {refdes}.{pin}"),
        ObjectRef::Net { name } => format!("net {name}"),
        ObjectRef::Footprint { name } => format!("footprint {name}"),
        ObjectRef::Pad { refdes, pad } => format!("pad {refdes}.{pad}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_rich_diagnostic_without_color() {
        let diagnostic = Diagnostic::coded(
            "net.unknown_pin",
            "net BROKEN references unknown pin B on U1",
        )
        .at(ObjectRef::pin("U1", "B"))
        .relates_to(ObjectRef::net("BROKEN"));

        let text = render_diagnostic(&diagnostic, ColorChoice::Never);

        assert!(text.contains("error[net.unknown_pin]: Net references an unknown pin"));
        assert!(text.contains("  = net BROKEN references unknown pin B on U1"));
        assert!(text.contains(" --> pin U1.B"));
        assert!(text.contains("note: related net BROKEN"));
        assert!(text.contains("help: Define the pin on the part"));
        assert!(text.contains("note: run `via explain net.unknown_pin` for more detail"));
    }

    #[test]
    fn renders_explain_text() {
        let definition = diagnostic_definition("net.unknown_pin").unwrap();
        let text = explain_text(definition);

        assert!(text.contains("net.unknown_pin: Net references an unknown pin"));
        assert!(text.contains("Common causes:"));
        assert!(text.contains("Help:"));
    }
}
