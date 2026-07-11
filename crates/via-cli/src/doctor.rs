use std::path::PathBuf;

use clap::Args;
use serde::Serialize;
use via_core::{Board, Error, FootprintAsset};

#[derive(Debug, Args)]
pub struct DoctorArgs {
    #[arg(
        value_name = "DESIGN",
        help = "Design name from via.toml; defaults to the project default"
    )]
    pub design: Option<String>,
    #[arg(long, help = "Print machine-readable JSON doctor results")]
    pub json: bool,
    #[arg(long, help = "Run production-grade checks instead of prototype checks")]
    pub production: bool,
}

#[derive(Debug, Serialize)]
struct DoctorReport {
    ok: bool,
    version: String,
    checks: Vec<DoctorCheck>,
}

#[derive(Debug, Serialize)]
struct DoctorCheck {
    id: &'static str,
    status: DoctorStatus,
    message: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    details: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    hints: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
enum DoctorStatus {
    Ok,
    Warn,
    Fail,
    Skip,
}

impl DoctorStatus {
    fn label(self) -> &'static str {
        match self {
            DoctorStatus::Ok => "OK",
            DoctorStatus::Warn => "WARN",
            DoctorStatus::Fail => "FAIL",
            DoctorStatus::Skip => "SKIP",
        }
    }
}

pub fn run(project_path: Option<PathBuf>, args: DoctorArgs) -> via_core::Result<()> {
    let report = build_report(project_path, args.design.as_deref(), args.production);

    if args.json {
        let mut text = serde_json::to_string_pretty(&report)
            .map_err(|err| Error::Io(format!("failed to serialize doctor JSON: {err}")))?;
        text.push('\n');
        print!("{text}");
    } else {
        print_human_report(&report);
    }

    if !report.ok {
        std::process::exit(1);
    }

    Ok(())
}

fn build_report(
    project_path: Option<PathBuf>,
    requested_design: Option<&str>,
    production: bool,
) -> DoctorReport {
    let mut report = DoctorReport {
        ok: true,
        version: env!("CARGO_PKG_VERSION").to_owned(),
        checks: Vec::new(),
    };

    report.push(DoctorCheck::ok(
        "version",
        format!(
            "via-pcb-cli {}, default KiCad footprints {}",
            env!("CARGO_PKG_VERSION"),
            via_kicad_footprints::DEFAULT_KICAD_FOOTPRINTS_VERSION
        ),
    ));

    let project = match via_project::Project::discover(project_path) {
        Ok(project) => {
            report.push(DoctorCheck::ok(
                "project",
                project.manifest_path().display().to_string(),
            ));
            project
        }
        Err(err) => {
            report.push(
                DoctorCheck::fail("project", error_message(&err))
                    .with_details(error_details(&err))
                    .with_hints(error_hints(&err)),
            );
            skip_project_dependent_checks(&mut report);
            return report.finalize();
        }
    };

    let design_name = match project.resolve_design_name(requested_design) {
        Ok(name) => {
            let name = name.to_owned();
            report.push(DoctorCheck::ok("design", name.clone()));
            name
        }
        Err(err) => {
            report.push(
                DoctorCheck::fail("design", error_message(&err))
                    .with_details(error_details(&err))
                    .with_hints(error_hints(&err)),
            );
            skip_provider_dependent_checks(&mut report);
            check_footprint_cache(&mut report, &project);
            return report.finalize();
        }
    };

    let provider_stdout = match project.emit_design_ir_json(&design_name) {
        Ok(stdout) => {
            report.push(DoctorCheck::ok("provider", "provider completed"));
            stdout
        }
        Err(err) => {
            report.push(
                DoctorCheck::fail("provider", error_message(&err))
                    .with_details(error_details(&err))
                    .with_hints(error_hints(&err)),
            );
            report.push(DoctorCheck::skip("board-ir", "provider unavailable"));
            skip_board_dependent_checks(&mut report);
            check_footprint_cache(&mut report, &project);
            return report.finalize();
        }
    };

    let board_ir = match via_project::parse_board_ir_json(&design_name, &provider_stdout) {
        Ok(ir) => ir,
        Err(err) => {
            report.push(
                DoctorCheck::fail("board-ir", error_message(&err))
                    .with_details(error_details(&err))
                    .with_hints(error_hints(&err)),
            );
            skip_board_dependent_checks(&mut report);
            check_footprint_cache(&mut report, &project);
            return report.finalize();
        }
    };
    let board_ir_version = board_ir.version;
    let board = match Board::from_ir(board_ir) {
        Ok(board) => {
            report.push(DoctorCheck::ok(
                "board-ir",
                format!("via.board v{board_ir_version}, board {}", board.name()),
            ));
            board
        }
        Err(err) => {
            report.push(
                DoctorCheck::fail("board-ir", error_message(&err))
                    .with_details(error_details(&err))
                    .with_hints(error_hints(&err)),
            );
            skip_board_dependent_checks(&mut report);
            check_footprint_cache(&mut report, &project);
            return report.finalize();
        }
    };

    let diagnostics = if production {
        board.production_diagnostics()
    } else {
        board.diagnostics()
    };
    if diagnostics.is_empty() {
        report.push(DoctorCheck::ok(
            "checks",
            if production {
                "0 production diagnostics".to_owned()
            } else {
                "0 diagnostics".to_owned()
            },
        ));
    } else {
        let details = diagnostics
            .iter()
            .take(5)
            .map(|diagnostic| diagnostic.to_string())
            .collect::<Vec<_>>();
        report.push(
            DoctorCheck::fail("checks", format!("{} diagnostics", diagnostics.len()))
                .with_details(details)
                .with_hint(if production {
                    "run `via check --profile production` for full diagnostics"
                } else {
                    "run `via check` for full diagnostics"
                }),
        );
    }

    check_kicad_config(&mut report, &project, &board);
    check_footprint_cache(&mut report, &project);
    check_required_official_footprints(&mut report, &project, &board);

    report.finalize()
}

fn check_kicad_config(report: &mut DoctorReport, project: &via_project::Project, board: &Board) {
    let output_dir = match project.kicad_output_dir(None) {
        Ok(output_dir) => output_dir,
        Err(err) => {
            report.push(
                DoctorCheck::fail("kicad-config", error_message(&err))
                    .with_details(error_details(&err))
                    .with_hints(error_hints(&err)),
            );
            return;
        }
    };

    let project_name = match project.kicad_project_name(board) {
        Ok(project_name) => project_name,
        Err(err) => {
            report.push(
                DoctorCheck::fail("kicad-config", error_message(&err))
                    .with_details(error_details(&err))
                    .with_hints(error_hints(&err)),
            );
            return;
        }
    };
    let footprint_export =
        match project.kicad_footprint_export(&output_dir, &project_name, None, None, None) {
            Ok(footprint_export) => footprint_export,
            Err(err) => {
                report.push(
                    DoctorCheck::fail("kicad-config", error_message(&err))
                        .with_details(error_details(&err))
                        .with_hints(error_hints(&err)),
                );
                return;
            }
        };
    report.push(
        DoctorCheck::ok(
            "kicad-config",
            format!("{}, project {}", output_dir.display(), project_name),
        )
        .with_detail(format!(
            "footprint library {} at {}",
            footprint_export.library_name,
            footprint_export.output_dir.display()
        )),
    );
}

fn check_footprint_cache(report: &mut DoctorReport, project: &via_project::Project) {
    let version = footprint_version_for_project(project);
    let root = match via_kicad_footprints::cache_dir_for_version(&version) {
        Ok(root) => root,
        Err(err) => {
            report.push(DoctorCheck::warn(
                "footprint-cache",
                format!("KiCad cache version {version:?} is invalid: {err}"),
            ));
            return;
        }
    };
    match via_kicad_footprints::FootprintCache::open_at(&version, &root) {
        Ok(cache) => {
            report.push(DoctorCheck::ok(
                "footprint-cache",
                format!(
                    "KiCad cache {} manifest indexes {} footprints at {}",
                    cache.version(),
                    cache.manifest().footprints.len(),
                    cache.root().display()
                ),
            ));
        }
        Err(err) => {
            report.push(
                DoctorCheck::warn(
                    "footprint-cache",
                    format!(
                        "KiCad cache {version} manifest missing or invalid at {}: {err}",
                        root.display()
                    ),
                )
                .with_hint(format!(
                    "run `via footprints install --version {version}` or `via footprints status --version {version}`"
                )),
            );
        }
    }
}

fn check_required_official_footprints(
    report: &mut DoctorReport,
    project: &via_project::Project,
    board: &Board,
) {
    let required = required_official_footprint_count(board);
    if required == 0 {
        report.push(DoctorCheck::skip(
            "official-footprints",
            "no cache-backed official KiCad footprints required",
        ));
        return;
    }

    let version = footprint_version_for_project(project);
    match crate::kicad_export::load_required_official_footprint_texts(board, &version) {
        Ok(footprints) => {
            report.push(DoctorCheck::ok(
                "official-footprints",
                format!("{} required official footprints loaded", footprints.len()),
            ));
        }
        Err(err) => {
            report.push(
                DoctorCheck::fail("official-footprints", error_message(&err))
                    .with_hint("run `via footprints status` or import the required KiCad cache"),
            );
        }
    }
}

fn required_official_footprint_count(board: &Board) -> usize {
    board
        .footprints()
        .filter(|footprint| {
            footprint.ir().is_none()
                && matches!(footprint.asset(), Some(FootprintAsset::KicadLibrary { .. }))
        })
        .count()
}

fn skip_project_dependent_checks(report: &mut DoctorReport) {
    for id in [
        "design",
        "provider",
        "board-ir",
        "checks",
        "kicad-config",
        "footprint-cache",
        "official-footprints",
    ] {
        report.push(DoctorCheck::skip(id, "project unavailable"));
    }
}

fn skip_provider_dependent_checks(report: &mut DoctorReport) {
    for id in [
        "provider",
        "board-ir",
        "checks",
        "kicad-config",
        "official-footprints",
    ] {
        report.push(DoctorCheck::skip(id, "design unavailable"));
    }
}

fn skip_board_dependent_checks(report: &mut DoctorReport) {
    for id in ["checks", "kicad-config", "official-footprints"] {
        report.push(DoctorCheck::skip(id, "Board IR unavailable"));
    }
}

fn footprint_version_for_project(project: &via_project::Project) -> String {
    project
        .kicad_footprints_version()
        .unwrap_or(via_kicad_footprints::DEFAULT_KICAD_FOOTPRINTS_VERSION)
        .to_owned()
}

fn print_human_report(report: &DoctorReport) {
    println!("via doctor {}", report.version);
    println!();
    for check in &report.checks {
        println!(
            "{:<5} {:<20} {}",
            check.status.label(),
            check.id,
            check.message
        );
        for detail in &check.details {
            for line in detail.lines() {
                println!("      {line}");
            }
        }
        for hint in &check.hints {
            println!("      help: {hint}");
        }
    }
}

fn error_message(error: &Error) -> String {
    match error {
        Error::Diagnostic(diagnostic) => first_line(diagnostic.message()),
        Error::Validation(diagnostics) => format!("{} validation diagnostics", diagnostics.len()),
        Error::DuplicateModule(refdes) => format!("duplicate module refdes: {refdes}"),
        Error::Io(message) => first_line(message),
    }
}

fn error_details(error: &Error) -> Vec<String> {
    match error {
        Error::Diagnostic(diagnostic) => remaining_lines(diagnostic.message()),
        Error::Validation(diagnostics) => diagnostics.iter().map(ToString::to_string).collect(),
        _ => Vec::new(),
    }
}

fn error_hints(error: &Error) -> Vec<String> {
    let Some(code) = error_code(error) else {
        return Vec::new();
    };
    via_core::diagnostic_definition(code)
        .map(|definition| {
            definition
                .help
                .iter()
                .map(|hint| (*hint).to_owned())
                .collect()
        })
        .unwrap_or_default()
}

fn error_code(error: &Error) -> Option<&str> {
    match error {
        Error::Diagnostic(diagnostic) => diagnostic.code(),
        _ => None,
    }
}

fn first_line(text: &str) -> String {
    text.lines().next().unwrap_or(text).to_owned()
}

fn remaining_lines(text: &str) -> Vec<String> {
    text.lines().skip(1).map(str::to_owned).collect()
}

impl DoctorReport {
    fn push(&mut self, check: DoctorCheck) {
        self.checks.push(check);
    }

    fn finalize(mut self) -> Self {
        self.ok = !self
            .checks
            .iter()
            .any(|check| check.status == DoctorStatus::Fail);
        self
    }
}

impl DoctorCheck {
    fn ok(id: &'static str, message: impl Into<String>) -> Self {
        Self::new(id, DoctorStatus::Ok, message)
    }

    fn warn(id: &'static str, message: impl Into<String>) -> Self {
        Self::new(id, DoctorStatus::Warn, message)
    }

    fn fail(id: &'static str, message: impl Into<String>) -> Self {
        Self::new(id, DoctorStatus::Fail, message)
    }

    fn skip(id: &'static str, message: impl Into<String>) -> Self {
        Self::new(id, DoctorStatus::Skip, message)
    }

    fn new(id: &'static str, status: DoctorStatus, message: impl Into<String>) -> Self {
        Self {
            id,
            status,
            message: message.into(),
            details: Vec::new(),
            hints: Vec::new(),
        }
    }

    fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.details.push(detail.into());
        self
    }

    fn with_details(mut self, details: Vec<String>) -> Self {
        self.details.extend(details);
        self
    }

    fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hints.push(hint.into());
        self
    }

    fn with_hints(mut self, hints: Vec<String>) -> Self {
        self.hints.extend(hints);
        self
    }
}
