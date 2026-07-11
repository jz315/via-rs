use std::path::Path;

use via_core::{Board, Result, atomic_write};

pub fn write_netlist(board: &Board, path: impl AsRef<Path>) -> Result<()> {
    board.check()?;
    let mut out = String::new();

    out.push_str("(export (version \"E\")\n");
    out.push_str(&format!(
        "  (design (source \"via\") (title \"{}\"))\n",
        escape(board.name())
    ));
    out.push_str("  (components\n");
    for module in board.modules() {
        out.push_str(&format!(
            "    (comp (ref \"{}\") (value \"{}\")",
            escape(module.refdes()),
            escape(module.value())
        ));
        if let Some(footprint) = module.footprint_name() {
            out.push_str(&format!(" (footprint \"{}\")", escape(footprint)));
        }
        if module.requires_verification() {
            out.push_str(" (property (name \"VIA_VERIFY\") (value \"true\"))");
        }
        out.push_str(")\n");
    }
    out.push_str("  )\n");
    out.push_str("  (nets\n");
    for (idx, net) in board.nets().enumerate() {
        out.push_str(&format!(
            "    (net (code \"{}\") (name \"{}\")\n",
            idx + 1,
            escape(net.name())
        ));
        for pin_ref in net.connections() {
            let Some(module) = board.module(&pin_ref.module) else {
                continue;
            };

            for pad in module.pads_for_pin(&pin_ref.pin) {
                out.push_str(&format!(
                    "      (node (ref \"{}\") (pin \"{}\"))\n",
                    escape(&pin_ref.module),
                    escape(&pad)
                ));
            }
        }
        out.push_str("    )\n");
    }
    out.push_str("  )\n");
    out.push_str(")\n");

    atomic_write(path, out)
}

fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use via_core::{Design, model::Part};

    #[test]
    fn exports_physical_pad_numbers() {
        let mut design = Design::new("physical_netlist");
        let module = design
            .add(
                Part::new("U1", "module")
                    .footprint("Module")
                    .pins(["GPIO4", "GND"])
                    .map_pin("GPIO4", "4")
                    .map_pin_to_pads("GND", ["22", "23"]),
            )
            .unwrap();
        design
            .net("SIGNAL")
            .connect_all(&mut design, [module.pin("GPIO4"), module.pin("GND")]);
        let board = design.build().unwrap();

        let path = std::env::temp_dir().join("via_physical_netlist_test.net");
        write_netlist(&board, &path).unwrap();
        let netlist = std::fs::read_to_string(&path).unwrap();
        let _ = std::fs::remove_file(path);

        assert!(netlist.contains(r#"(node (ref "U1") (pin "4"))"#));
        assert!(netlist.contains(r#"(node (ref "U1") (pin "22"))"#));
        assert!(netlist.contains(r#"(node (ref "U1") (pin "23"))"#));
        assert!(!netlist.contains(r#"(pin "GPIO4")"#));
    }
}
