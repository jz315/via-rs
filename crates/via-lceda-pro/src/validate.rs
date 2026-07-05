use std::{error, fmt};

use via_footprint_ir::{PadShape, TextKind};

use crate::context::ExportContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LcedaExportError {
    diagnostics: Vec<String>,
}

impl LcedaExportError {
    fn new(diagnostics: Vec<String>) -> Self {
        Self { diagnostics }
    }

    pub(crate) fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }
}

impl fmt::Display for LcedaExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LCEDA Pro export validation failed: {}",
            self.diagnostics.join("; ")
        )
    }
}

impl error::Error for LcedaExportError {}

pub(crate) fn validate_lceda_export(ctx: &ExportContext<'_>) -> Result<(), LcedaExportError> {
    let mut diagnostics = Vec::new();

    validate_component_footprints(ctx, &mut diagnostics);
    validate_footprint_geometry(ctx, &mut diagnostics);

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(LcedaExportError::new(diagnostics))
    }
}

fn validate_component_footprints(ctx: &ExportContext<'_>, diagnostics: &mut Vec<String>) {
    for module in ctx.board().modules() {
        let Some(footprint_name) = module.footprint_name() else {
            diagnostics.push(format!("{} has no footprint", module.refdes()));
            continue;
        };
        let Some(footprint) = ctx.footprint(footprint_name) else {
            diagnostics.push(format!(
                "{} references missing footprint {}",
                module.refdes(),
                footprint_name
            ));
            continue;
        };

        for pin in module.pins_iter() {
            for pad in module.pads_for_pin(pin) {
                if !footprint.contains_pad(&pad) {
                    diagnostics.push(format!(
                        "{} pin {} maps to missing LCEDA pad {} on footprint {}",
                        module.refdes(),
                        pin,
                        pad,
                        footprint_name
                    ));
                }
            }
        }

        let modeled_pads = module.modeled_pads();
        let uncovered = footprint
            .pads()
            .difference(&modeled_pads)
            .cloned()
            .collect::<Vec<_>>();
        if !uncovered.is_empty() {
            diagnostics.push(format!(
                "{} does not map all LCEDA pads on footprint {}: {}",
                module.refdes(),
                footprint_name,
                uncovered.join(", ")
            ));
        }
    }
}

fn validate_footprint_geometry(ctx: &ExportContext<'_>, diagnostics: &mut Vec<String>) {
    for footprint in ctx.footprints() {
        let Some(ir) = footprint.ir() else {
            diagnostics.push(format!(
                "footprint {} has no geometry IR; refusing 55x55 placeholder pad export",
                footprint.name()
            ));
            continue;
        };

        if let Err(error) = ir.validate() {
            diagnostics.push(format!(
                "footprint {} is invalid: {error}",
                footprint.name()
            ));
        }

        for text in ir.texts() {
            if matches!(text.kind, TextKind::User) && text.text == "REF**" {
                diagnostics.push(format!(
                    "footprint {} contains REF** as user text; use Reference text kind instead",
                    footprint.name()
                ));
            }
            if matches!(text.kind, TextKind::User) && text.text == footprint.name() {
                diagnostics.push(format!(
                    "footprint {} contains its value/name as user text; use Value text kind instead",
                    footprint.name()
                ));
            }
        }

        for pad in ir.pads() {
            if let Some(drill) = pad.drill {
                if !drill.is_round() && pad.shape != PadShape::Oval {
                    diagnostics.push(format!(
                        "footprint {} pad {} uses a slotted drill but is not an oval pad",
                        footprint.name(),
                        pad.number
                    ));
                }
            }
        }
    }
}
