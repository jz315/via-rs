use std::path::PathBuf;

use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};

mod json;
mod kicad_export;
mod kicad_mod_asset;
mod pcb_export;
mod report;

#[derive(Debug, Parser)]
#[command(
    name = "via",
    version,
    about = "Project-oriented CLI for via circuit designs"
)]
struct Cli {
    #[arg(long, global = true, value_name = "FILE_OR_DIR")]
    project: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    Build(BuildArgs),
    Check(CheckArgs),
    #[command(name = "check-production")]
    CheckProduction(CheckArgs),
    Inspect(InspectArgs),
    Snapshot(InspectArgs),
    Designs,
    Bom(BomArgs),
    Footprints {
        #[command(subcommand)]
        target: FootprintsTarget,
    },
    Export {
        #[command(subcommand)]
        target: ExportTarget,
    },
}

#[derive(Debug, Args)]
struct BuildArgs {
    design: Option<String>,
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct CheckArgs {
    design: Option<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct InspectArgs {
    design: Option<String>,
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct BomArgs {
    design: Option<String>,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long, value_enum)]
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
    Kicad(ExportKicadArgs),
    Lceda(ExportLcedaArgs),
    Pcb(ExportPcbArgs),
}

#[derive(Debug, Args)]
struct ExportKicadArgs {
    design: Option<String>,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long)]
    footprint_library_name: Option<String>,
    #[arg(long)]
    footprint_library_path: Option<String>,
    #[arg(long)]
    footprint_output_dir: Option<PathBuf>,
    #[arg(long)]
    no_footprints: bool,
    #[arg(long)]
    production: bool,
}

#[derive(Debug, Args)]
struct ExportLcedaArgs {
    design: Option<String>,
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct ExportPcbArgs {
    design: Option<String>,
    #[arg(long)]
    layout: Option<PathBuf>,
    #[arg(long)]
    out: Option<PathBuf>,
    #[arg(long)]
    footprint_library_name: Option<String>,
}

#[derive(Debug, Subcommand)]
enum FootprintsTarget {
    Status(FootprintsStatusArgs),
    Import(FootprintsImportArgs),
    Fetch(FootprintsFetchArgs),
}

#[derive(Debug, Args)]
struct FootprintsStatusArgs {
    #[arg(long)]
    version: Option<String>,
    #[arg(long)]
    cache_dir: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct FootprintsImportArgs {
    #[arg(long)]
    from: PathBuf,
    #[arg(long)]
    version: Option<String>,
    #[arg(long)]
    cache_dir: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct FootprintsFetchArgs {
    #[arg(long)]
    version: Option<String>,
    #[arg(long)]
    url: Option<String>,
    #[arg(long)]
    cache_dir: Option<PathBuf>,
}

fn main() -> via_core::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Build(args)) => build_command(cli.project, args),
        Some(Command::Check(args)) => check_command(cli.project, args, false),
        Some(Command::CheckProduction(args)) => check_command(cli.project, args, true),
        Some(Command::Inspect(args)) | Some(Command::Snapshot(args)) => {
            inspect_command(cli.project, args)
        }
        Some(Command::Designs) => designs_command(cli.project),
        Some(Command::Bom(args)) => bom_command(cli.project, args),
        Some(Command::Footprints { target }) => match target {
            FootprintsTarget::Status(args) => footprints_status_command(cli.project, args),
            FootprintsTarget::Import(args) => footprints_import_command(cli.project, args),
            FootprintsTarget::Fetch(args) => footprints_fetch_command(cli.project, args),
        },
        Some(Command::Export { target }) => match target {
            ExportTarget::Kicad(args) => export_kicad_command(cli.project, args),
            ExportTarget::Lceda(args) => export_lceda_command(cli.project, args),
            ExportTarget::Pcb(args) => export_pcb_command(cli.project, args),
        },
        None => {
            Cli::command().print_help().map_err(via_core::Error::from)?;
            println!();
            Ok(())
        }
    }
}

fn build_command(project_path: Option<PathBuf>, args: BuildArgs) -> via_core::Result<()> {
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

fn inspect_command(project_path: Option<PathBuf>, args: InspectArgs) -> via_core::Result<()> {
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

fn check_command(
    project_path: Option<PathBuf>,
    args: CheckArgs,
    production: bool,
) -> via_core::Result<()> {
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
