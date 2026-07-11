use std::path::{Path, PathBuf};

use clap::Args;
use serde::Serialize;
use via_core::{Board, PinRef};

#[derive(Debug, Args)]
pub struct ShowArgs {
    #[arg(
        value_name = "DESIGN",
        help = "Design name from via.toml; defaults to the project default"
    )]
    pub design: Option<String>,
}

#[derive(Debug, Args)]
pub struct NetsArgs {
    #[arg(
        value_name = "DESIGN",
        help = "Design name from via.toml; defaults to the project default"
    )]
    pub design: Option<String>,
    #[arg(long, help = "Print machine-readable JSON net data")]
    pub json: bool,
}

#[derive(Debug, Serialize)]
struct NetsReport {
    design: String,
    board: String,
    nets: Vec<NetReport>,
}

#[derive(Debug, Serialize)]
struct NetReport {
    name: String,
    class: Option<String>,
    connection_count: usize,
    connections: Vec<ConnectionReport>,
}

#[derive(Debug, Serialize)]
struct ConnectionReport {
    module: String,
    pin: String,
    pads: Vec<String>,
    known_module: bool,
    known_pin: bool,
}

pub fn show(project_path: Option<PathBuf>, args: ShowArgs) -> via_core::Result<()> {
    let project = via_project::Project::discover(project_path)?;
    let (design_name, board) = project.build_design(args.design.as_deref())?;
    print!("{}", render_show(&project, &design_name, &board)?);
    Ok(())
}

pub fn nets(project_path: Option<PathBuf>, args: NetsArgs) -> via_core::Result<()> {
    let project = via_project::Project::discover(project_path)?;
    let (design_name, board) = project.build_design(args.design.as_deref())?;
    let report = nets_report(&design_name, &board);

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(|err| via_core::Error::Io(format!(
                "failed to serialize nets JSON: {err}"
            )))?
        );
    } else {
        print!("{}", render_nets_human(&report));
    }

    Ok(())
}

fn render_show(
    project: &via_project::Project,
    design_name: &str,
    board: &Board,
) -> via_core::Result<String> {
    let config = project.config();
    let mut out = String::new();
    let version = config.project.version.as_deref().unwrap_or("not set");
    let designs = config
        .designs
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(", ");
    let default_design = config
        .project
        .default_design
        .as_deref()
        .unwrap_or("not set");

    out.push_str("via inspect summary\n\n");
    line(&mut out, "project", &config.project.name);
    line(&mut out, "version", version);
    line(
        &mut out,
        "manifest",
        &display_project_path(project, project.manifest_path()),
    );
    line(&mut out, "designs", &designs);
    line(&mut out, "selected", design_name);
    line(&mut out, "default", default_design);
    out.push('\n');
    line(&mut out, "board", board.name());
    line(&mut out, "modules", &board.modules().count().to_string());
    line(&mut out, "nets", &board.nets().count().to_string());
    line(
        &mut out,
        "footprints",
        &board.footprints().count().to_string(),
    );
    out.push('\n');
    out.push_str("outputs\n");
    render_kicad_output(project, board, &mut out)?;
    render_lceda_output(project, &mut out);
    Ok(out)
}

fn render_kicad_output(
    project: &via_project::Project,
    board: &Board,
    out: &mut String,
) -> via_core::Result<()> {
    let Some(kicad) = project.config().outputs.kicad.as_ref() else {
        line(out, "  kicad", "not configured");
        return Ok(());
    };
    let Some(dir) = kicad.dir.clone() else {
        line(out, "  kicad", "not configured");
        return Ok(());
    };

    let output_dir = project.resolve_path(dir);
    let project_name = project.kicad_project_name(board)?;
    let footprint_export =
        project.kicad_footprint_export(&output_dir, &project_name, None, None, None)?;

    line(
        out,
        "  kicad dir",
        &display_project_path(project, &output_dir),
    );
    line(out, "  kicad project", &project_name);
    line(out, "  footprint lib", &footprint_export.library_name);
    line(
        out,
        "  footprint dir",
        &display_project_path(project, &footprint_export.output_dir),
    );
    Ok(())
}

fn render_lceda_output(project: &via_project::Project, out: &mut String) {
    if let Some(lceda) = project.config().outputs.lceda_pro.as_ref() {
        let file = project.resolve_path(lceda.file.clone());
        line(out, "  lceda-pro", &display_project_path(project, &file));
    } else {
        line(out, "  lceda-pro", "not configured");
    }
}

fn render_nets_human(report: &NetsReport) -> String {
    let mut out = format!("{} / {} nets\n\n", report.design, report.board);
    for net in &report.nets {
        let class = net.class.as_deref().unwrap_or("unclassified");
        out.push_str(&format!(
            "{} [{}] {} connections\n",
            net.name, class, net.connection_count
        ));
        for connection in &net.connections {
            out.push_str("  ");
            out.push_str(&connection.module);
            out.push('.');
            out.push_str(&connection.pin);
            if !connection.known_module {
                out.push_str(" -> unknown module\n");
            } else if !connection.known_pin {
                out.push_str(" -> unknown pin\n");
            } else if connection.pads.is_empty() {
                out.push_str(" -> pads none\n");
            } else {
                out.push_str(" -> pads ");
                out.push_str(&connection.pads.join(", "));
                out.push('\n');
            }
        }
        out.push('\n');
    }
    out
}

fn nets_report(design_name: &str, board: &Board) -> NetsReport {
    NetsReport {
        design: design_name.to_owned(),
        board: board.name().to_owned(),
        nets: board
            .nets()
            .map(|net| NetReport {
                name: net.name().to_owned(),
                class: net.electrical_class().map(ToString::to_string),
                connection_count: net.connections().len(),
                connections: net
                    .connections()
                    .iter()
                    .map(|pin| connection_report(board, pin))
                    .collect(),
            })
            .collect(),
    }
}

fn connection_report(board: &Board, pin: &PinRef) -> ConnectionReport {
    let module = board.module(&pin.module);
    let known_module = module.is_some();
    let known_pin = module
        .map(|module| module.pins_iter().any(|known| known == &pin.pin))
        .unwrap_or(false);
    let pads = if let Some(module) = module.filter(|_| known_pin) {
        module.pads_for_pin(&pin.pin).into_iter().collect()
    } else {
        Vec::new()
    };

    ConnectionReport {
        module: pin.module.clone(),
        pin: pin.pin.clone(),
        pads,
        known_module,
        known_pin,
    }
}

fn line(out: &mut String, label: &str, value: &str) {
    out.push_str(&format!("{label:<16} {value}\n"));
}

fn display_project_path(project: &via_project::Project, path: &Path) -> String {
    path.strip_prefix(project.root())
        .ok()
        .filter(|relative| !relative.as_os_str().is_empty())
        .unwrap_or(path)
        .display()
        .to_string()
}
