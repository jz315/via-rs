mod model;
mod render;
mod util;

use std::path::Path;

use via_core::{Board, Result, atomic_write};

use model::{pin_net_map, place_modules, symbol_templates};
use render::{
    render_fp_lib_table, render_project_file, render_schematic, render_sym_lib_table,
    render_symbol_library,
};

const SCHEMATIC_VERSION: &str = "20250610";
const SYMBOL_LIB_VERSION: &str = "20211014";
const GENERATOR: &str = "via-kicad";
const PIN_SPACING: f64 = 2.54;
const PIN_LENGTH: f64 = 5.08;
const BODY_HALF_WIDTH: f64 = 20.0;
const PIN_X: f64 = 25.4;
const LABEL_STUB: f64 = 7.62;

#[derive(Debug, Clone)]
pub struct SchematicProjectOptions {
    pub symbol_library_name: String,
    pub project_name: Option<String>,
    pub footprint_library_name: Option<String>,
    pub footprint_library_uri: Option<String>,
}

impl SchematicProjectOptions {
    pub fn new(symbol_library_name: impl Into<String>) -> Self {
        Self {
            symbol_library_name: symbol_library_name.into(),
            project_name: None,
            footprint_library_name: None,
            footprint_library_uri: None,
        }
    }

    pub fn project_name(mut self, name: impl Into<String>) -> Self {
        self.project_name = Some(name.into());
        self
    }

    pub fn footprint_library(mut self, name: impl Into<String>, uri: impl Into<String>) -> Self {
        self.footprint_library_name = Some(name.into());
        self.footprint_library_uri = Some(uri.into());
        self
    }
}

impl Default for SchematicProjectOptions {
    fn default() -> Self {
        Self::new("VIA")
    }
}

pub fn write_schematic_project(
    board: &Board,
    out_dir: impl AsRef<Path>,
    options: &SchematicProjectOptions,
) -> Result<()> {
    board.check()?;

    let out_dir = out_dir.as_ref();
    let stem = options
        .project_name
        .as_deref()
        .unwrap_or_else(|| board.name());
    via_core::validate_file_stem(stem)?;
    let symbol_library_file = format!("{stem}.kicad_sym");
    let schematic_file = out_dir.join(format!("{stem}.kicad_sch"));
    let project_file = out_dir.join(format!("{stem}.kicad_pro"));

    let templates = symbol_templates(board);
    let placed = place_modules(board, &templates);
    let pin_nets = pin_net_map(board);

    atomic_write(
        out_dir.join(&symbol_library_file),
        render_symbol_library(&templates),
    )?;
    atomic_write(
        out_dir.join("sym-lib-table"),
        render_sym_lib_table(options, &symbol_library_file),
    )?;
    atomic_write(out_dir.join("fp-lib-table"), render_fp_lib_table(options))?;
    atomic_write(project_file, render_project_file(board, stem))?;
    atomic_write(
        schematic_file,
        render_schematic(board, options, &templates, &placed, &pin_nets),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schematic::model::{PlacedModule, SymbolPin};
    use crate::schematic::render::render_pin_connection;
    use crate::schematic::util::stable_uuid;
    use std::collections::BTreeMap;
    use std::process::Command;
    use via_core::{Design, SymbolSide, model::Part};

    fn test_board() -> Board {
        let mut design = Design::new("demo");
        let u1 = design
            .add(
                Part::new("U1", "Demo MCU")
                    .footprint("Demo_MCU")
                    .pins(["GPIO4", "GND"])
                    .map_pin("GPIO4", "4")
                    .map_pin_to_pads("GND", ["1", "2"]),
            )
            .unwrap();
        let j1 = design
            .add(
                Part::new("J1", "Connector")
                    .footprint("Conn_01x02")
                    .pins(["1", "2"]),
            )
            .unwrap();
        design
            .net("SIGNAL")
            .connect_all(&mut design, [u1.pin("GPIO4"), j1.pin("1")]);
        design
            .ground("GND")
            .connect_all(&mut design, [u1.pin("GND"), j1.pin("2")]);
        design
            .rules_mut()
            .set_default_track_width_mm(0.42)
            .set_clearance_mm(0.23);
        design.build().unwrap()
    }

    #[test]
    fn writes_openable_project_files() {
        let board = test_board();

        let out = std::env::temp_dir().join(format!("via_sch_test_{}", stable_uuid("demo")));
        let options = SchematicProjectOptions::new("DEMO")
            .footprint_library("demo_footprints", "${KIPRJMOD}/demo.pretty");
        write_schematic_project(&board, &out, &options).unwrap();

        let schematic = std::fs::read_to_string(out.join("demo.kicad_sch")).unwrap();
        assert!(schematic.contains("(kicad_sch"));
        assert!(schematic.contains("(label \"SIGNAL\""));
        assert!(schematic.contains("(label \"GND\""));
        let project = std::fs::read_to_string(out.join("demo.kicad_pro")).unwrap();
        assert!(project.contains("\"copper_line_width\": 0.42"));
        assert!(project.contains("\"min_clearance\": 0.23"));
        assert!(project.contains("\"min_track_width\": 0.42"));
        assert!(out.join("demo.kicad_sym").exists());
        assert!(out.join("sym-lib-table").exists());
        assert!(out.join("fp-lib-table").exists());

        let _ = std::fs::remove_dir_all(out);
    }

    #[test]
    #[ignore = "requires VIA_KICAD_CLI pointing to a real KiCad CLI executable"]
    fn kicad_cli_accepts_generated_schematic() {
        let kicad_cli = std::env::var_os("VIA_KICAD_CLI")
            .expect("set VIA_KICAD_CLI to the KiCad CLI executable");
        let board = test_board();
        let out = std::env::temp_dir().join(format!(
            "via_kicad_cli_test_{}",
            stable_uuid("kicad-cli-smoke")
        ));
        let _ = std::fs::remove_dir_all(&out);
        let options = SchematicProjectOptions::new("DEMO")
            .footprint_library("demo_footprints", "${KIPRJMOD}/demo.pretty");
        write_schematic_project(&board, &out, &options).unwrap();

        let schematic = out.join("demo.kicad_sch");
        let netlist = out.join("demo.net");
        let output = Command::new(kicad_cli)
            .args(["sch", "export", "netlist", "--output"])
            .arg(&netlist)
            .arg(&schematic)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "stdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(netlist.is_file());
        std::fs::remove_dir_all(out).unwrap();
    }

    #[test]
    fn configured_project_name_controls_all_schematic_file_stems() {
        let board = Design::new("board_name").into_unchecked_board();
        let out = std::env::temp_dir().join(format!(
            "via_sch_project_name_{}",
            stable_uuid("configured-project-name")
        ));
        let options = SchematicProjectOptions::new("SYMBOLS").project_name("configured_name");

        write_schematic_project(&board, &out, &options).unwrap();

        assert!(out.join("configured_name.kicad_sch").is_file());
        assert!(out.join("configured_name.kicad_pro").is_file());
        assert!(out.join("configured_name.kicad_sym").is_file());
        assert!(!out.join("board_name.kicad_sch").exists());
        let project = std::fs::read_to_string(out.join("configured_name.kicad_pro")).unwrap();
        assert!(project.contains("\"filename\": \"configured_name.kicad_pro\""));

        let _ = std::fs::remove_dir_all(out);
    }

    #[test]
    fn rejects_project_names_that_escape_the_output_directory() {
        let board = Design::new("board_name").into_unchecked_board();
        let out = std::env::temp_dir().join(format!(
            "via_sch_unsafe_name_{}",
            stable_uuid("unsafe-project-name")
        ));
        let _ = std::fs::remove_dir_all(&out);
        let options = SchematicProjectOptions::new("SYMBOLS").project_name("../escape");

        let err = write_schematic_project(&board, &out, &options).unwrap_err();

        let via_core::Error::Diagnostic(diagnostic) = err else {
            panic!("expected a diagnostic error");
        };
        assert_eq!(diagnostic.code(), Some("export.invalid_file_stem"));
        assert!(!out.exists());
    }

    #[test]
    fn same_footprint_modules_keep_their_own_logical_symbol_pins() {
        let mut design = Design::new("shared_footprint");
        let u1 = design
            .add(
                Part::new("U1", "Input")
                    .footprint("Shared_1Pin")
                    .pins(["A"])
                    .map_pin("A", "1"),
            )
            .unwrap();
        let u2 = design
            .add(
                Part::new("U2", "Output")
                    .footprint("Shared_1Pin")
                    .pins(["B"])
                    .map_pin("B", "1"),
            )
            .unwrap();
        let j1 = design
            .add(
                Part::new("J1", "Input Tap")
                    .footprint("Tap_1Pin")
                    .pins(["1"]),
            )
            .unwrap();
        let j2 = design
            .add(
                Part::new("J2", "Output Tap")
                    .footprint("Tap_1Pin")
                    .pins(["1"]),
            )
            .unwrap();
        design
            .net("NET_A")
            .connect_all(&mut design, [u1.pin("A"), j1.pin("1")]);
        design
            .net("NET_B")
            .connect_all(&mut design, [u2.pin("B"), j2.pin("1")]);
        let board = design.build().unwrap();

        let out = std::env::temp_dir().join(format!("via_sch_shared_{}", stable_uuid("shared")));
        write_schematic_project(&board, &out, &SchematicProjectOptions::new("SHARED")).unwrap();

        let schematic = std::fs::read_to_string(out.join("shared_footprint.kicad_sch")).unwrap();
        assert!(schematic.contains("(label \"NET_A\""));
        assert!(schematic.contains("(label \"NET_B\""));
        assert!(!schematic.contains("(no_connect "));
        assert!(schematic.contains("SHARED:Shared_1Pin_U1"));
        assert!(schematic.contains("SHARED:Shared_1Pin_U2"));

        let _ = std::fs::remove_dir_all(out);
    }

    #[test]
    fn pin_connection_uses_kicad_symbol_y_axis() {
        let module = PlacedModule {
            refdes: "U1".to_owned(),
            value: "Demo".to_owned(),
            footprint: None,
            symbol_name: "Demo".to_owned(),
            x: 100.0,
            y: 100.0,
            pins: Vec::new(),
            half_height: 10.0,
        };
        let pin = SymbolPin {
            logical_pin: "A".to_owned(),
            number: "1".to_owned(),
            name: "A".to_owned(),
            side: SymbolSide::Left,
            x: -25.4,
            y: 5.08,
            rotation: 0,
        };
        let pin_nets = BTreeMap::from([(("U1".to_owned(), "A".to_owned()), "NET_A".to_owned())]);

        let rendered = render_pin_connection(&module, &pin, &pin_nets);

        assert!(rendered.contains("(xy 74.6 94.92)"));
        assert!(rendered.contains("(label \"NET_A\""));
    }
}
