use std::env;
use std::path::{Path, PathBuf};

mod json;
mod pcb_export;
mod report;

const POLAR_ADJUSTER: &str = "polar-adjuster";

fn main() -> via_core::Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let mut arg_cursor = args.iter().map(String::as_str);

    match arg_cursor.next() {
        Some("check") => check(&args[1..]),
        Some("check-production") => check_production(&args[1..]),
        Some("inspect") | Some("snapshot") => inspect(&args[1..]),
        Some("export") => export(&args[1..]),
        Some("export-kicad-import") => export_kicad_import(&args[1..]),
        Some("export-lceda-pro") => export_lceda_pro(&args[1..]),
        Some("export-pcb") => export_pcb(&args[1..]),
        Some("help") | Some("--help") | Some("-h") | None => {
            print_help();
            Ok(())
        }
        Some(command) => {
            eprintln!("unknown command: {command}");
            print_help();
            std::process::exit(2);
        }
    }
}

fn inspect(args: &[String]) -> via_core::Result<()> {
    let example = parse_example(args);
    let out = parse_option_value(args, "--out").map(PathBuf::from);

    let board = load_example(example)?;
    let loaded = board.footprints().count();
    let diagnostics = board.diagnostics();
    let production_diagnostics = board.production_diagnostics();
    let snapshot = json::board_snapshot(&board, loaded, &diagnostics, &production_diagnostics);
    if let Some(out) = out {
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

fn check(args: &[String]) -> via_core::Result<()> {
    check_with(args, false)
}

fn check_production(args: &[String]) -> via_core::Result<()> {
    check_with(args, true)
}

fn check_with(args: &[String], production: bool) -> via_core::Result<()> {
    let example = parse_example(args);
    let json = args.iter().any(|arg| arg == "--json");

    let board = load_example(example)?;
    let loaded = board.footprints().count();
    let diagnostics = if production {
        board.production_diagnostics()
    } else {
        board.diagnostics()
    };

    if json {
        println!(
            "{}",
            json::check_summary(board.name(), diagnostics.is_empty(), loaded, &diagnostics)
        );
    }

    if diagnostics.is_empty() {
        if !json {
            println!(
                "{} {}ok; embedded {loaded} footprint pad maps",
                board.name(),
                if production { "production " } else { "" },
            );
        }
        Ok(())
    } else {
        if !json {
            for diagnostic in &diagnostics {
                eprintln!("{diagnostic}");
            }
        }
        std::process::exit(1);
    }
}

fn export(args: &[String]) -> via_core::Result<()> {
    let example = parse_example(args);
    let out = parse_option_value(args, "--out")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("../../../electronics/generated/via/polar_adjuster_v0"));
    let no_footprints = args.iter().any(|arg| arg == "--no-footprints");

    let board = load_example(example)?;
    println!("embedded {} footprint pad maps", board.footprints().count());

    let exported = write_kicad_artifacts(&board, &out, !no_footprints)?;
    report::write(&board, out.join(format!("{}_report.md", board.name())))?;
    println!(
        "wrote {} ({} generated footprints, {} manual footprints)",
        out.display(),
        exported.generated_footprints,
        exported.manual_footprints
    );
    Ok(())
}

fn export_kicad_import(args: &[String]) -> via_core::Result<()> {
    let example = parse_example(args);
    let out = parse_option_value(args, "--out")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from("../../../electronics/generated/lceda_kicad_import/polar_adjuster_v0")
        });

    let board = load_example(example)?;
    println!("embedded {} footprint pad maps", board.footprints().count());

    let exported = write_kicad_artifacts(&board, &out, true)?;
    write_kicad_import_readme(&board, &out)?;
    println!(
        "wrote KiCad import project {} ({} generated footprints, {} manual footprints)",
        out.display(),
        exported.generated_footprints,
        exported.manual_footprints
    );
    println!(
        "import in LCEDA Pro from {}",
        out.join(format!("{}.kicad_pro", board.name())).display()
    );
    Ok(())
}

fn export_lceda_pro(args: &[String]) -> via_core::Result<()> {
    let example = parse_example(args);
    let out = parse_option_value(args, "--out")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from("../../../electronics/generated/lceda_pro/polar_adjuster_v0.epro2")
        });

    let board = load_example(example)?;
    println!("embedded {} footprint pad maps", board.footprints().count());
    via_lceda_pro::write_lceda_pro_project(&board, &out)
        .map_err(|err| via_core::Error::Io(err.to_string()))?;
    println!("wrote LCEDA Pro package {}", out.display());
    Ok(())
}

fn export_pcb(args: &[String]) -> via_core::Result<()> {
    let example = parse_example(args);
    let layout = parse_option_value(args, "--layout")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from("../../../electronics/generated/via/polar_adjuster_v0/polar_adjuster_v0.via-layout.json")
        });
    let out = parse_option_value(args, "--out")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(
                "../../../electronics/generated/via/polar_adjuster_v0/polar_adjuster_v0.kicad_pcb",
            )
        });

    let board = load_example(example)?;
    let loaded = board.footprints().count();
    let layout_model = pcb_export::read_layout(&layout)?;
    if layout_model.board != board.name() {
        eprintln!(
            "warning: layout board {} does not match example board {}",
            layout_model.board,
            board.name()
        );
    }
    pcb_export::write_kicad_pcb(&board, &layout_model, &out)?;
    println!(
        "wrote KiCad PCB {} from {} (loaded {loaded} footprints)",
        out.display(),
        layout.display()
    );
    Ok(())
}

struct KicadExportSummary {
    generated_footprints: usize,
    manual_footprints: usize,
}

fn write_kicad_artifacts(
    board: &via_core::Board,
    out: &Path,
    write_footprints: bool,
) -> via_core::Result<KicadExportSummary> {
    let stem = board.name();
    let pretty_dir = out.join("via_generated.pretty");
    let generated_footprints = if write_footprints {
        via_parts_harmonic::write_generated_footprints(&pretty_dir)
            .map_err(|err| via_core::Error::Io(err.to_string()))?
    } else {
        0
    };
    via_kicad::write_netlist(board, out.join(format!("{stem}.net")))?;
    via_kicad::write_schematic_project(
        board,
        out,
        &via_kicad::SchematicProjectOptions::new(stem)
            .footprint_library(stem, "${KIPRJMOD}/via_generated.pretty"),
    )?;

    Ok(KicadExportSummary {
        generated_footprints,
        manual_footprints: 0,
    })
}

fn write_kicad_import_readme(board: &via_core::Board, out: &Path) -> via_core::Result<()> {
    let text = format!(
        concat!(
            "# KiCad import package for LCEDA Pro\n\n",
            "Open LCEDA Pro and import the KiCad project file:\n\n",
            "- `{name}.kicad_pro`\n\n",
            "This package is schematic-first. It contains:\n\n",
            "- `{name}.kicad_sch`\n",
            "- `{name}.kicad_sym`\n",
            "- `sym-lib-table`\n",
            "- `fp-lib-table`\n",
            "- `via_generated.pretty/`\n",
            "- `{name}.net`\n\n",
            "Footprints marked `VERIFY` must still be checked against purchased modules before fabrication.\n"
        ),
        name = board.name()
    );
    std::fs::write(out.join("README_LCEDA_KICAD_IMPORT.md"), text)
        .map_err(|err| via_core::Error::Io(err.to_string()))
}

fn load_example(example: &str) -> via_core::Result<via_core::Board> {
    match example {
        POLAR_ADJUSTER => via_examples::polar_adjuster::polar_adjuster_v0_board(),
        other => {
            eprintln!("unknown example: {other}");
            eprintln!("available examples: {POLAR_ADJUSTER}");
            std::process::exit(2);
        }
    }
}

fn parse_example(args: &[String]) -> &str {
    parse_option_value_ref(args, "--example").unwrap_or(POLAR_ADJUSTER)
}

fn parse_option_value(args: &[String], option: &str) -> Option<String> {
    parse_option_value_ref(args, option).map(str::to_owned)
}

fn parse_option_value_ref<'a>(args: &'a [String], option: &str) -> Option<&'a str> {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == option {
            return iter.next().map(String::as_str);
        }
    }
    None
}

fn print_help() {
    println!("via 0.1.0");
    println!();
    println!("Commands:");
    println!("  via check [--example NAME] [--json]");
    println!("      validate an example against embedded footprint pad maps");
    println!("  via check-production [--example NAME] [--json]");
    println!("      validate footprint verification and production sourcing gates");
    println!("  via inspect [--example NAME] [--out FILE.json]");
    println!("      print a machine-readable board snapshot for editor integrations");
    println!("  via snapshot [--example NAME] [--out FILE.json]");
    println!("      alias for inspect; preferred stable name for editor integrations");
    println!("  via export [--example NAME] [--out DIR] [--no-footprints]");
    println!("      export an example netlist and review report");
    println!("  via export-kicad-import [--example NAME] [--out DIR]");
    println!("      export a self-contained KiCad project directory for LCEDA Pro import");
    println!("  via export-lceda-pro [--example NAME] [--out FILE.epro2]");
    println!("      export an LCEDA Pro package from via's board model");
    println!("  via export-pcb [--example NAME] [--layout FILE] [--out FILE.kicad_pcb]");
    println!("      export a KiCad PCB draft from a VIA PCB layout JSON");
    println!();
    println!("Examples:");
    println!("  {POLAR_ADJUSTER}");
}
