use std::fs;
use std::path::Path;

use via_core::{Board, Result};

pub fn write(board: &Board, path: impl AsRef<Path>) -> Result<()> {
    board.check()?;

    let mut out = String::new();
    out.push_str(&format!("# {}\n\n", board.name()));
    out.push_str("## Modules\n\n");
    for module in board.modules() {
        out.push_str(&format!("- {}: {}", module.refdes(), module.value()));
        if let Some(footprint) = module.footprint_name() {
            out.push_str(&format!(" [{}]", footprint));
        }
        if module.requires_verification() {
            out.push_str(" VERIFY");
        }
        if let Some(mpn) = module.manufacturer_part_number() {
            out.push_str(&format!(" MPN={mpn}"));
        }
        for (supplier, part_number) in module.supplier_parts() {
            out.push_str(&format!(" {supplier}={part_number}"));
        }
        for note in module.production_notes() {
            out.push_str(&format!(" NOTE={note}"));
        }
        out.push('\n');
    }

    out.push_str("\n## Pinmaps\n\n");
    for module in board.modules() {
        let mappings = module
            .pins_iter()
            .map(|pin| {
                let pads = module
                    .pads_for_pin(pin)
                    .into_iter()
                    .collect::<Vec<_>>()
                    .join(", ");
                let class = module
                    .class_for_pin(pin)
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "unclassified".to_owned());
                format!("{pin} -> {pads} [{class}]")
            })
            .collect::<Vec<_>>()
            .join("; ");
        out.push_str(&format!("- {}: {}\n", module.refdes(), mappings));
    }

    out.push_str("\n## Footprints Loaded\n\n");
    for footprint in board.footprints() {
        out.push_str(&format!(
            "- {}: {} pads",
            footprint.name(),
            footprint.pads().len()
        ));
        if let Some(source) = footprint.source() {
            out.push_str(&format!(" ({})", source.display()));
        }
        out.push('\n');
    }

    out.push_str("\n## Nets\n\n");
    for net in board.nets() {
        let class = net
            .electrical_class()
            .map(ToString::to_string)
            .unwrap_or_else(|| "unclassified".to_owned());
        let pins = net
            .connections()
            .iter()
            .map(|pin_ref| {
                if let Some(module) = board.module(&pin_ref.module) {
                    let pads = module
                        .pads_for_pin(&pin_ref.pin)
                        .into_iter()
                        .collect::<Vec<_>>()
                        .join(",");
                    format!("{pin_ref}[{pads}]")
                } else {
                    pin_ref.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("- {} [{}]: {}\n", net.name(), class, pins));
    }

    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, out)?;
    Ok(())
}
