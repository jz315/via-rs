use std::path::{Path, PathBuf};

use clap::Args;

#[derive(Debug, Args)]
pub struct InitArgs {
    #[arg(
        value_name = "DIR",
        help = "Project directory to create; defaults to the current directory"
    )]
    path: Option<PathBuf>,
    #[arg(long, value_name = "NAME", help = "Project/package name")]
    name: Option<String>,
    #[arg(
        long,
        default_value = "main",
        value_name = "DESIGN",
        help = "Default via design name"
    )]
    design: String,
    #[arg(long, help = "Overwrite generated files if they already exist")]
    force: bool,
}

pub fn run(args: InitArgs) -> via_core::Result<()> {
    let target = args.path.unwrap_or_else(|| PathBuf::from("."));
    let root = if target.is_absolute() {
        target
    } else {
        std::env::current_dir()?.join(target)
    };
    let scaffold = Scaffold::new(&root, args.name.as_deref(), &args.design);
    write_scaffold(&root, &scaffold, args.force)?;
    println!("initialized via project {}", root.display());
    println!(
        "next: cd {} && via check {}",
        root.display(),
        scaffold.design
    );
    Ok(())
}

struct Scaffold {
    package_name: String,
    crate_name: String,
    design: String,
    kicad_name: String,
}

impl Scaffold {
    fn new(root: &Path, name: Option<&str>, design: &str) -> Self {
        let raw_name = name
            .map(str::to_owned)
            .or_else(|| {
                root.file_name()
                    .map(|name| name.to_string_lossy().to_string())
            })
            .unwrap_or_else(|| "via-board".to_owned());
        let package_name = sanitize_package_name(&raw_name);
        let crate_name = rust_crate_name(&package_name);
        let design = sanitize_design_name(design);
        let kicad_name = package_name.replace('-', "_");
        Self {
            package_name,
            crate_name,
            design,
            kicad_name,
        }
    }
}

fn write_scaffold(root: &Path, scaffold: &Scaffold, force: bool) -> via_core::Result<()> {
    if root.exists() && !root.is_dir() {
        return Err(via_core::Error::Io(format!(
            "{} exists and is not a directory",
            root.display()
        )));
    }

    std::fs::create_dir_all(root)?;
    std::fs::create_dir_all(root.join("src").join("bin"))?;

    write_file(root.join("via.toml"), &via_toml(scaffold), force)?;
    write_file(root.join("Cargo.toml"), &cargo_toml(scaffold), force)?;
    write_file(root.join("src").join("lib.rs"), &lib_rs(scaffold), force)?;
    write_file(
        root.join("src").join("bin").join("emit-ir.rs"),
        &emit_ir_rs(scaffold),
        force,
    )?;
    write_file(root.join(".gitignore"), GITIGNORE, force)?;
    Ok(())
}

fn write_file(path: PathBuf, contents: &str, force: bool) -> via_core::Result<()> {
    if path.exists() && !force {
        return Err(via_core::Error::Io(format!(
            "{} already exists; pass --force to overwrite generated files",
            path.display()
        )));
    }
    std::fs::write(path, contents)?;
    Ok(())
}

fn via_toml(scaffold: &Scaffold) -> String {
    format!(
        r#"[project]
name = "{package}"
version = "0.1.0"
default-design = "{design}"

[designs.{design}]
provider = "cargo"
package = "{package}"
bin = "emit-ir"

[outputs.kicad]
dir = "generated/kicad"
project = "{kicad}"
footprint-library-name = "{kicad}"
footprint-library-path = "{kicad}.pretty"
footprint-output-dir = "generated/kicad/{kicad}.pretty"

[kicad-footprints]
version = "10.0.4"
source = "github-release"
"#,
        package = scaffold.package_name,
        design = scaffold.design,
        kicad = scaffold.kicad_name,
    )
}

fn cargo_toml(scaffold: &Scaffold) -> String {
    format!(
        r#"[package]
name = "{package}"
version = "0.1.0"
edition = "2024"

[workspace]

[dependencies]
via = {{ package = "via-pcb", version = "0.1.1" }}
"#,
        package = scaffold.package_name,
    )
}

fn lib_rs(scaffold: &Scaffold) -> String {
    format!(
        r#"use via::prelude::*;

pub fn board() -> Result<Board> {{
    let mut design = Design::new("{design}")
        .rules(Rules::new())
        .units(Unit::Mm);

    let signal = design.signal("SIGNAL", "3V3");
    let ground = design.ground("GND");

    let input = design.add(
        part("J1", "Signal input")
            .footprint(fp::pin_1x02())
            .symbol(sym::connector().left(["SIG", "GND"]))
            .pin(pin("SIG").logic("3V3").pad("1"))
            .pin(pin("GND").ground().pad("2")),
    )?;

    let output = design.add(
        part("J2", "Signal output")
            .footprint(fp::pin_1x02())
            .symbol(sym::connector().left(["SIG", "GND"]))
            .pin(pin("SIG").logic("3V3").pad("1"))
            .pin(pin("GND").ground().pad("2")),
    )?;

    design.connect(&signal, [input.pin("SIG"), output.pin("SIG")]);
    design.connect(&ground, [input.pin("GND"), output.pin("GND")]);

    design.finish()
}}
"#,
        design = scaffold.design,
    )
}

fn emit_ir_rs(scaffold: &Scaffold) -> String {
    format!(
        r#"fn main() -> via::core::Result<()> {{
    let board = {crate_name}::board()?;
    via::project::emit_ir(&board)
}}
"#,
        crate_name = scaffold.crate_name,
    )
}

const GITIGNORE: &str = r#"/target
/generated
"#;

fn sanitize_package_name(raw: &str) -> String {
    let mut out = String::new();
    let mut previous_dash = false;
    for ch in raw.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            previous_dash = false;
        } else if matches!(ch, '-' | '_' | ' ' | '.') && !previous_dash && !out.is_empty() {
            out.push('-');
            previous_dash = true;
        }
    }

    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        out.push_str("via-board");
    }
    if out.starts_with(|ch: char| ch.is_ascii_digit()) {
        out.insert_str(0, "via-");
    }
    out
}

fn rust_crate_name(package_name: &str) -> String {
    let candidate = package_name.replace('-', "_");
    if is_rust_keyword(&candidate) {
        format!("{candidate}_project")
    } else {
        candidate
    }
}

fn sanitize_design_name(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        "main".to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn is_rust_keyword(candidate: &str) -> bool {
    matches!(
        candidate,
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "async"
            | "await"
            | "dyn"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn init_writes_minimal_project_scaffold() {
        let root = temp_root("via_init_scaffold");
        let scaffold = Scaffold::new(&root, Some("My Board"), "main");

        write_scaffold(&root, &scaffold, false).unwrap();

        assert!(root.join("via.toml").is_file());
        assert!(root.join("Cargo.toml").is_file());
        assert!(root.join("src").join("lib.rs").is_file());
        assert!(root.join("src").join("bin").join("emit-ir.rs").is_file());

        let via_toml = std::fs::read_to_string(root.join("via.toml")).unwrap();
        assert!(via_toml.contains("name = \"my-board\""));
        assert!(via_toml.contains("package = \"my-board\""));
        assert!(via_toml.contains("bin = \"emit-ir\""));
        let cargo_toml = std::fs::read_to_string(root.join("Cargo.toml")).unwrap();
        assert!(cargo_toml.contains("[workspace]"));

        let emit_ir =
            std::fs::read_to_string(root.join("src").join("bin").join("emit-ir.rs")).unwrap();
        assert!(emit_ir.contains("my_board::board()"));

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn init_refuses_to_overwrite_existing_files() {
        let root = temp_root("via_init_no_overwrite");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("via.toml"), "existing").unwrap();
        let scaffold = Scaffold::new(&root, Some("demo"), "main");

        let err = write_scaffold(&root, &scaffold, false).unwrap_err();

        assert!(format!("{err}").contains("already exists"));
        assert_eq!(
            std::fs::read_to_string(root.join("via.toml")).unwrap(),
            "existing"
        );

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn sanitizes_package_and_crate_names() {
        let root = PathBuf::from("ignored");
        let scaffold = Scaffold::new(&root, Some("123 Fancy.Board"), "  ");

        assert_eq!(scaffold.package_name, "via-123-fancy-board");
        assert_eq!(scaffold.crate_name, "via_123_fancy_board");
        assert_eq!(scaffold.design, "main");
    }

    fn temp_root(prefix: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "{prefix}_{}_{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }
}
