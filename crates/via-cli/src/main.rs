use std::path::PathBuf;

use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};

mod json;
mod kicad_export;
mod kicad_mod_asset;
mod pcb_export;
mod report;

#[cfg(test)]
mod test_fixtures {
    use via_core::{Board, FootprintPads, part, pin};
    use via_footprint_ir::{FootprintIr, Pad, PadShape, Point, Size};

    pub fn debug_io_board() -> via_core::Result<Board> {
        let mut design = via_core::Design::new("debug_io_demo");
        let vin = design.power_domain("5V_IN", "5V");
        let v3v3 = design.power_domain("3V3", "3V3");
        let ground = design.ground("GND");
        let i2c_scl = design.logic("I2C_SCL", "3V3");
        let led_status = design.logic("LED_STATUS", "3V3");

        let regulator = design.add(
            part("U1", "fixture regulator")
                .footprint(smd_footprint("SOT-223", &["1", "2", "3", "4"]))
                .pin(pin("VIN").power("5V").pad("1"))
                .pin(pin("GND").ground().pad("2"))
                .pin(pin("VOUT").power("3V3").pad("3"))
                .pin(pin("TAB").power("3V3").pad("4")),
        )?;

        let bus = design.add(
            part("U2", "fixture TSSOP-20 device")
                .footprint(smd_footprint(
                    "TSSOP-20",
                    &[
                        "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14",
                        "15", "16", "17", "18", "19", "20",
                    ],
                ))
                .pin_specs((1..=20).map(|number| {
                    let pin_name = number.to_string();
                    let spec = pin(pin_name.clone()).pad(pin_name);
                    match number {
                        1 => spec.logic("3V3"),
                        10 => spec.ground(),
                        20 => spec.power("3V3"),
                        _ => spec.passive(),
                    }
                })),
        )?;

        let led = design.add(
            part("D1", "fixture LED")
                .footprint(smd_footprint("LED_0805", &["1", "2"]))
                .pin(pin("K").passive().pad("1"))
                .pin(pin("A").passive().pad("2")),
        )?;

        let tp_5v = design.add(testpad("TP1", "5V test pad"))?;
        let tp_scl = design.add(testpad("TP2", "SCL test pad"))?;
        let tp_led = design.add(testpad("TP3", "LED test pad"))?;

        vin.connect_all(&mut design, [regulator.pin("VIN"), tp_5v.pin("1")]);
        v3v3.connect_all(
            &mut design,
            [regulator.pin("VOUT"), regulator.pin("TAB"), bus.pin("20")],
        );
        ground.connect_all(
            &mut design,
            [regulator.pin("GND"), bus.pin("10"), led.pin("K")],
        );
        i2c_scl.connect_all(&mut design, [bus.pin("1"), tp_scl.pin("1")]);
        led_status.connect_all(&mut design, [led.pin("A"), tp_led.pin("1")]);

        design.build()
    }

    fn testpad(refdes: &str, value: &str) -> impl via_core::Component<Output = via_core::ModuleId> {
        part(refdes, value)
            .footprint(smd_footprint("TESTPAD_1", &["1"]))
            .pin(pin("1").passive().pad("1"))
    }

    fn smd_footprint(name: &str, pads: &[&str]) -> FootprintPads {
        let mut ir = FootprintIr::new(name);
        for (idx, pad) in pads.iter().enumerate() {
            ir.add_pad(Pad::smd(
                *pad,
                PadShape::Rect,
                Point::new(idx as f64 * 1.2, 0.0),
                Size::new(1.0, 0.7),
            ));
        }
        FootprintPads::from_ir(ir)
    }
}

#[derive(Debug, Parser)]
#[command(
    name = "via",
    version,
    about = "Validate via circuit designs and export reviewable EDA artifacts",
    long_about = "Validate via circuit designs, render Board IR and snapshots, manage KiCad footprint caches, and export KiCad / LCEDA Pro artifacts."
)]
struct Cli {
    #[arg(
        long,
        global = true,
        value_name = "FILE_OR_DIR",
        help = "Path to via.toml or a directory containing it"
    )]
    project: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(about = "Emit Board IR JSON for a design")]
    Ir(IrArgs),
    #[command(about = "Validate a design")]
    Check(CheckArgs),
    #[command(about = "Emit a JSON snapshot for tooling and CI")]
    Snapshot(SnapshotArgs),
    #[command(about = "List designs declared by the via project")]
    Designs,
    #[command(about = "Render a bill of materials")]
    Bom(BomArgs),
    #[command(about = "Manage the KiCad footprint cache")]
    Footprints {
        #[command(subcommand)]
        target: FootprintsTarget,
    },
    #[command(about = "Export design artifacts")]
    Export {
        #[command(subcommand)]
        target: ExportTarget,
    },
}

#[derive(Debug, Args)]
struct IrArgs {
    #[arg(
        value_name = "DESIGN",
        help = "Design name from via.toml; defaults to the project default"
    )]
    design: Option<String>,
    #[arg(
        long,
        value_name = "FILE",
        help = "Write Board IR JSON to a file instead of stdout"
    )]
    out: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct CheckArgs {
    #[arg(
        value_name = "DESIGN",
        help = "Design name from via.toml; defaults to the project default"
    )]
    design: Option<String>,
    #[arg(long, help = "Print machine-readable JSON diagnostics")]
    json: bool,
    #[arg(long, help = "Run production-grade checks instead of prototype checks")]
    production: bool,
}

#[derive(Debug, Args)]
struct SnapshotArgs {
    #[arg(
        value_name = "DESIGN",
        help = "Design name from via.toml; defaults to the project default"
    )]
    design: Option<String>,
    #[arg(
        long,
        value_name = "FILE",
        help = "Write snapshot JSON to a file instead of stdout"
    )]
    out: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct BomArgs {
    #[arg(
        value_name = "DESIGN",
        help = "Design name from via.toml; defaults to the project default"
    )]
    design: Option<String>,
    #[arg(
        long,
        value_name = "FILE",
        help = "Write the BOM to a file instead of stdout"
    )]
    out: Option<PathBuf>,
    #[arg(long, value_enum, help = "BOM output format")]
    format: BomFormat,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum BomFormat {
    Csv,
    Json,
    Md,
}

#[derive(Debug, Subcommand)]
enum ExportTarget {
    #[command(about = "Export a reviewable KiCad project")]
    Kicad(ExportKicadArgs),
    #[command(name = "lceda-pro", about = "Export an LCEDA Pro package")]
    LcedaPro(ExportLcedaArgs),
    #[command(about = "EXPERIMENTAL: render a KiCad PCB from a layout model")]
    Pcb(ExportPcbArgs),
}

#[derive(Debug, Args)]
struct ExportKicadArgs {
    #[arg(
        value_name = "DESIGN",
        help = "Design name from via.toml; defaults to the project default"
    )]
    design: Option<String>,
    #[arg(long, value_name = "DIR", help = "Override the KiCad output directory")]
    out: Option<PathBuf>,
    #[arg(long, help = "Override the generated KiCad footprint library name")]
    footprint_library_name: Option<String>,
    #[arg(
        long,
        help = "Override the KiCad footprint library path recorded in the project"
    )]
    footprint_library_path: Option<String>,
    #[arg(
        long,
        value_name = "DIR",
        help = "Override where generated footprint files are written"
    )]
    footprint_output_dir: Option<PathBuf>,
    #[arg(long, help = "Skip generated/local footprint library output")]
    no_footprints: bool,
    #[arg(long, help = "Run production checks before exporting")]
    production: bool,
}

#[derive(Debug, Args)]
struct ExportLcedaArgs {
    #[arg(
        value_name = "DESIGN",
        help = "Design name from via.toml; defaults to the project default"
    )]
    design: Option<String>,
    #[arg(
        long,
        value_name = "FILE",
        help = "Write the LCEDA Pro package to this file"
    )]
    out: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct ExportPcbArgs {
    #[arg(
        value_name = "DESIGN",
        help = "Design name from via.toml; defaults to the project default"
    )]
    design: Option<String>,
    #[arg(long, value_name = "FILE", help = "Layout JSON file to render")]
    layout: Option<PathBuf>,
    #[arg(long, value_name = "FILE", help = "KiCad PCB output file")]
    out: Option<PathBuf>,
    #[arg(long, help = "Override the local KiCad footprint library name")]
    footprint_library_name: Option<String>,
}

#[derive(Debug, Subcommand)]
enum FootprintsTarget {
    #[command(about = "Show KiCad footprint cache status")]
    Status(FootprintsStatusArgs),
    #[command(about = "Import a local KiCad footprint directory into the cache")]
    Import(FootprintsImportArgs),
    #[command(about = "EXPERIMENTAL: fetch a KiCad footprint cache bundle from a URL")]
    Fetch(FootprintsFetchArgs),
}

#[derive(Debug, Args)]
struct FootprintsStatusArgs {
    #[arg(long, help = "KiCad footprint library version")]
    version: Option<String>,
    #[arg(
        long,
        value_name = "DIR",
        help = "Override the footprint cache directory"
    )]
    cache_dir: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct FootprintsImportArgs {
    #[arg(
        long,
        value_name = "DIR",
        help = "KiCad footprint library directory to import"
    )]
    from: PathBuf,
    #[arg(long, help = "KiCad footprint library version")]
    version: Option<String>,
    #[arg(
        long,
        value_name = "DIR",
        help = "Override the footprint cache directory"
    )]
    cache_dir: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct FootprintsFetchArgs {
    #[arg(long, help = "KiCad footprint library version")]
    version: Option<String>,
    #[arg(long, help = "URL of a footprint cache bundle")]
    url: Option<String>,
    #[arg(
        long,
        value_name = "DIR",
        help = "Override the footprint cache directory"
    )]
    cache_dir: Option<PathBuf>,
}

fn main() -> via_core::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Ir(args)) => ir_command(cli.project, args),
        Some(Command::Check(args)) => check_command(cli.project, args),
        Some(Command::Snapshot(args)) => snapshot_command(cli.project, args),
        Some(Command::Designs) => designs_command(cli.project),
        Some(Command::Bom(args)) => bom_command(cli.project, args),
        Some(Command::Footprints { target }) => match target {
            FootprintsTarget::Status(args) => footprints_status_command(cli.project, args),
            FootprintsTarget::Import(args) => footprints_import_command(cli.project, args),
            FootprintsTarget::Fetch(args) => footprints_fetch_command(cli.project, args),
        },
        Some(Command::Export { target }) => match target {
            ExportTarget::Kicad(args) => export_kicad_command(cli.project, args),
            ExportTarget::LcedaPro(args) => export_lceda_command(cli.project, args),
            ExportTarget::Pcb(args) => export_pcb_command(cli.project, args),
        },
        None => {
            Cli::command().print_help().map_err(via_core::Error::from)?;
            println!();
            Ok(())
        }
    }
}

fn ir_command(project_path: Option<PathBuf>, args: IrArgs) -> via_core::Result<()> {
    let project = via_project::Project::discover(project_path)?;
    let (design_name, board) = project.build_design(args.design.as_deref())?;
    if let Some(out) = args.out.map(|path| project.resolve_path(path)) {
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&out, via_project::board_ir_json(&board)?)?;
        println!("wrote {} for design {design_name}", out.display());
    } else {
        println!("{}", via_project::board_ir_json(&board)?);
    }
    Ok(())
}

fn snapshot_command(project_path: Option<PathBuf>, args: SnapshotArgs) -> via_core::Result<()> {
    let project = via_project::Project::discover(project_path)?;
    let (_, board) = project.build_design(args.design.as_deref())?;
    let loaded = board.footprints().count();
    let diagnostics = board.diagnostics();
    let production_diagnostics = board.production_diagnostics();
    let snapshot = json::board_snapshot(&board, loaded, &diagnostics, &production_diagnostics);
    if let Some(out) = args.out.map(|path| project.resolve_path(path)) {
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&out, snapshot)?;
        eprintln!("wrote snapshot {}", out.display());
    } else {
        println!("{snapshot}");
    }
    Ok(())
}

fn check_command(project_path: Option<PathBuf>, args: CheckArgs) -> via_core::Result<()> {
    let production = args.production;
    let (_, board) = load_board(project_path, args.design)?;
    let loaded = board.footprints().count();
    let diagnostics = if production {
        board.production_diagnostics()
    } else {
        board.diagnostics()
    };

    if args.json {
        println!(
            "{}",
            json::check_summary(board.name(), diagnostics.is_empty(), loaded, &diagnostics)
        );
    }

    if diagnostics.is_empty() {
        if !args.json {
            println!(
                "{} {}ok; embedded {loaded} footprint pad maps",
                board.name(),
                if production { "production " } else { "" },
            );
        }
        Ok(())
    } else {
        if !args.json {
            for diagnostic in &diagnostics {
                eprintln!("{diagnostic}");
            }
        }
        std::process::exit(1);
    }
}

fn designs_command(project_path: Option<PathBuf>) -> via_core::Result<()> {
    let project = via_project::Project::discover(project_path)?;
    for name in project.design_names() {
        println!("{name}");
    }
    Ok(())
}

fn bom_command(project_path: Option<PathBuf>, args: BomArgs) -> via_core::Result<()> {
    let project = via_project::Project::discover(project_path)?;
    let (_, board) = project.build_design(args.design.as_deref())?;
    let bom = render_bom(&board, args.format);
    if let Some(out) = args.out.map(|path| project.resolve_path(path)) {
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&out, bom)?;
        println!("wrote BOM {}", out.display());
    } else {
        print!("{bom}");
    }
    Ok(())
}

fn footprints_status_command(
    project_path: Option<PathBuf>,
    args: FootprintsStatusArgs,
) -> via_core::Result<()> {
    let version = resolve_footprint_version(project_path.as_ref(), args.version)?;
    if let Some(cache_dir) = args.cache_dir {
        match via_kicad_footprints::FootprintCache::open_at(&version, &cache_dir) {
            Ok(cache) => {
                println!("version: {}", cache.version());
                println!("cache: {}", cache.root().display());
                println!("manifest: present");
                println!("footprints: {}", cache.manifest().footprints.len());
            }
            Err(err) => {
                println!("version: {version}");
                println!("cache: {}", cache_dir.display());
                println!("manifest: missing or invalid ({err})");
            }
        }
        return Ok(());
    }

    let status = via_kicad_footprints::cache_status(&version)
        .map_err(|err| via_core::Error::Io(err.to_string()))?;
    println!("version: {}", status.version);
    println!("cache: {}", status.root.display());
    println!(
        "manifest: {}",
        if status.manifest_exists {
            "present"
        } else {
            "missing"
        }
    );
    println!("footprints: {}", status.footprint_count);
    Ok(())
}

fn footprints_import_command(
    project_path: Option<PathBuf>,
    args: FootprintsImportArgs,
) -> via_core::Result<()> {
    let version = resolve_footprint_version(project_path.as_ref(), args.version)?;
    let cache_dir = args.cache_dir.clone();
    let root = cache_dir
        .clone()
        .unwrap_or_else(|| via_kicad_footprints::cache_dir_for_version(&version));
    let manifest = via_kicad_footprints::import_from_kicad_dir(&args.from, &version, cache_dir)
        .map_err(|err| via_core::Error::Io(err.to_string()))?;
    println!(
        "imported {} KiCad footprints for version {} into {}",
        manifest.footprints.len(),
        manifest.version,
        root.display()
    );
    Ok(())
}

fn footprints_fetch_command(
    project_path: Option<PathBuf>,
    args: FootprintsFetchArgs,
) -> via_core::Result<()> {
    let version = resolve_footprint_version(project_path.as_ref(), args.version)?;
    let project_url = match project_path.as_ref() {
        Some(path) => via_project::Project::discover(Some(path.clone()))
            .ok()
            .and_then(|project| project.kicad_footprints_url().map(str::to_owned)),
        None => via_project::Project::discover(None)
            .ok()
            .and_then(|project| project.kicad_footprints_url().map(str::to_owned)),
    };
    let url = args
        .url
        .or(project_url)
        .or_else(|| std::env::var(via_kicad_footprints::VIA_KICAD_FOOTPRINTS_URL_ENV).ok())
        .ok_or_else(|| {
            via_core::Error::Io(format!(
                "footprints fetch requires --url, [kicad-footprints].url, or {}",
                via_kicad_footprints::VIA_KICAD_FOOTPRINTS_URL_ENV
            ))
        })?;
    let cache_dir = args.cache_dir.clone();
    let root = cache_dir
        .clone()
        .unwrap_or_else(|| via_kicad_footprints::cache_dir_for_version(&version));
    let manifest = via_kicad_footprints::fetch_from_url(&url, &version, cache_dir)
        .map_err(|err| via_core::Error::Io(err.to_string()))?;
    println!(
        "fetched {} KiCad footprints for version {} into {}",
        manifest.footprints.len(),
        manifest.version,
        root.display()
    );
    Ok(())
}

fn export_kicad_command(
    project_path: Option<PathBuf>,
    args: ExportKicadArgs,
) -> via_core::Result<()> {
    let project = via_project::Project::discover(project_path)?;
    let (_, board) = project.build_design(args.design.as_deref())?;
    let out = project.kicad_output_dir(args.out)?;
    let project_name = project.kicad_project_name(&board);
    let footprint_cache_version = footprint_version_for_project(&project);
    let footprint_export = if args.no_footprints {
        None
    } else {
        Some(project.kicad_footprint_export(
            args.footprint_library_name,
            args.footprint_library_path,
            args.footprint_output_dir,
        )?)
    };
    export_kicad_board(
        &board,
        out,
        &project_name,
        footprint_export,
        &footprint_cache_version,
        args.production,
    )
}

fn export_kicad_board(
    board: &via_core::Board,
    out: PathBuf,
    project_name: &str,
    footprint_export: Option<via_project::KicadFootprintExport>,
    footprint_cache_version: &str,
    production: bool,
) -> via_core::Result<()> {
    if production {
        board.check_production()?;
    } else {
        board.check()?;
    }

    println!("embedded {} footprint pad maps", board.footprints().count());

    let exported = kicad_export::write_artifacts(
        board,
        &out,
        footprint_export,
        project_name,
        footprint_cache_version,
    )?;
    report::write(board, out.join(format!("{project_name}_report.md")))?;
    println!(
        "wrote {} ({} generated footprints, {} manual footprints)",
        out.display(),
        exported.generated_footprints,
        exported.manual_footprints
    );
    Ok(())
}

fn export_lceda_command(
    project_path: Option<PathBuf>,
    args: ExportLcedaArgs,
) -> via_core::Result<()> {
    let project = via_project::Project::discover(project_path)?;
    let (_, board) = project.build_design(args.design.as_deref())?;
    let out = project.lceda_output_file(args.out)?;

    board.check()?;
    println!("embedded {} footprint pad maps", board.footprints().count());
    via_lceda_pro::write_lceda_pro_project(&board, &out)
        .map_err(|err| via_core::Error::Io(err.to_string()))?;
    println!("wrote LCEDA Pro package {}", out.display());
    Ok(())
}

fn export_pcb_command(project_path: Option<PathBuf>, args: ExportPcbArgs) -> via_core::Result<()> {
    let project = via_project::Project::discover(project_path)?;
    let (_, board) = project.build_design(args.design.as_deref())?;
    board.check()?;
    let layout = args
        .layout
        .map(|path| project.resolve_path(path))
        .ok_or_else(|| via_core::Error::Io("export pcb requires --layout".to_owned()))?;
    let out = args
        .out
        .map(|path| project.resolve_path(path))
        .ok_or_else(|| via_core::Error::Io("export pcb requires --out".to_owned()))?;
    let footprint_library_name =
        project.kicad_footprint_library_name(args.footprint_library_name)?;
    let footprint_cache_version = footprint_version_for_project(&project);
    let official_footprints =
        kicad_export::load_required_official_footprint_texts(&board, &footprint_cache_version)?;
    let loaded = board.footprints().count();
    let layout_model = pcb_export::read_layout(&layout)?;
    pcb_export::write_kicad_pcb(
        &board,
        &layout_model,
        &out,
        &footprint_library_name,
        &official_footprints,
    )?;
    println!(
        "wrote KiCad PCB {} from {} (loaded {loaded} footprints)",
        out.display(),
        layout.display()
    );
    Ok(())
}

fn load_board(
    project_path: Option<PathBuf>,
    design: Option<String>,
) -> via_core::Result<(String, via_core::Board)> {
    let project = via_project::Project::discover(project_path)?;
    project.build_design(design.as_deref())
}

fn footprint_version_for_project(project: &via_project::Project) -> String {
    project
        .kicad_footprints_version()
        .unwrap_or(via_kicad_footprints::DEFAULT_KICAD_FOOTPRINTS_VERSION)
        .to_owned()
}

fn resolve_footprint_version(
    project_path: Option<&PathBuf>,
    override_version: Option<String>,
) -> via_core::Result<String> {
    if let Some(version) = override_version {
        return Ok(version);
    }

    let project = match project_path {
        Some(path) => via_project::Project::discover(Some(path.clone())).ok(),
        None => via_project::Project::discover(None).ok(),
    };
    Ok(project
        .as_ref()
        .and_then(|project| project.kicad_footprints_version())
        .unwrap_or(via_kicad_footprints::DEFAULT_KICAD_FOOTPRINTS_VERSION)
        .to_owned())
}

#[cfg(test)]
mod cli_tests {
    use super::*;

    fn help_for(mut command: clap::Command) -> String {
        let mut bytes = Vec::new();
        command.write_long_help(&mut bytes).unwrap();
        String::from_utf8(bytes).unwrap()
    }

    #[test]
    fn top_level_help_lists_current_commands() {
        let help = help_for(Cli::command());

        for command in ["ir", "check", "snapshot", "export", "footprints"] {
            assert!(
                help.contains(command),
                "expected top-level help to contain {command:?}:\n{help}"
            );
        }

        for removed in ["check-production", "inspect", "build"] {
            assert!(
                !help.contains(removed),
                "expected top-level help not to contain removed command {removed:?}:\n{help}"
            );
        }
    }

    #[test]
    fn check_help_exposes_production_flag() {
        let mut command = Cli::command();
        let check = command.find_subcommand_mut("check").unwrap().clone();
        let help = help_for(check);

        assert!(help.contains("--production"), "{help}");
    }

    #[test]
    fn export_help_uses_lceda_pro_and_marks_pcb_experimental() {
        let mut command = Cli::command();
        let export = command.find_subcommand_mut("export").unwrap().clone();
        let help = help_for(export);

        assert!(help.contains("lceda-pro"), "{help}");
        assert!(help.contains("EXPERIMENTAL"), "{help}");
        assert!(!help.contains("lceda "), "{help}");
    }
}

fn render_bom(board: &via_core::Board, format: BomFormat) -> String {
    match format {
        BomFormat::Csv => render_bom_csv(board),
        BomFormat::Json => render_bom_json(board),
        BomFormat::Md => render_bom_md(board),
    }
}

fn render_bom_csv(board: &via_core::Board) -> String {
    let mut out = String::from(
        "refdes,value,footprint,manufacturer_part_number,supplier_parts,requires_verification,production_notes\n",
    );
    for module in board.modules() {
        let row = [
            module.refdes().to_owned(),
            module.value().to_owned(),
            module.footprint_name().unwrap_or_default().to_owned(),
            module
                .manufacturer_part_number()
                .unwrap_or_default()
                .to_owned(),
            supplier_parts_text(module),
            module.requires_verification().to_string(),
            module.production_notes().join("; "),
        ];
        out.push_str(
            &row.iter()
                .map(|cell| csv_cell(cell))
                .collect::<Vec<_>>()
                .join(","),
        );
        out.push('\n');
    }
    out
}

fn render_bom_json(board: &via_core::Board) -> String {
    let mut out = String::from("[\n");
    for (idx, module) in board.modules().enumerate() {
        let comma = if idx + 1 == board.modules().count() {
            ""
        } else {
            ","
        };
        out.push_str(&format!(
            concat!(
                "  {{",
                "\"refdes\":\"{}\",",
                "\"value\":\"{}\",",
                "\"footprint\":\"{}\",",
                "\"manufacturerPartNumber\":\"{}\",",
                "\"supplierParts\":\"{}\",",
                "\"requiresVerification\":{},",
                "\"productionNotes\":\"{}\"",
                "}}{}\n"
            ),
            json::escape_json(module.refdes()),
            json::escape_json(module.value()),
            json::escape_json(module.footprint_name().unwrap_or_default()),
            json::escape_json(module.manufacturer_part_number().unwrap_or_default()),
            json::escape_json(&supplier_parts_text(module)),
            module.requires_verification(),
            json::escape_json(&module.production_notes().join("; ")),
            comma
        ));
    }
    out.push_str("]\n");
    out
}

fn render_bom_md(board: &via_core::Board) -> String {
    let mut out = String::from(
        "| Refdes | Value | Footprint | MPN | Supplier parts | Verify | Notes |\n| --- | --- | --- | --- | --- | --- | --- |\n",
    );
    for module in board.modules() {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            md_cell(module.refdes()),
            md_cell(module.value()),
            md_cell(module.footprint_name().unwrap_or_default()),
            md_cell(module.manufacturer_part_number().unwrap_or_default()),
            md_cell(&supplier_parts_text(module)),
            module.requires_verification(),
            md_cell(&module.production_notes().join("; ")),
        ));
    }
    out
}

fn supplier_parts_text(module: &via_core::model::Part) -> String {
    module
        .supplier_parts()
        .map(|(supplier, part)| format!("{supplier}:{part}"))
        .collect::<Vec<_>>()
        .join("; ")
}

fn csv_cell(value: &str) -> String {
    if value
        .chars()
        .any(|ch| matches!(ch, ',' | '"' | '\n' | '\r'))
    {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}

fn md_cell(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}
