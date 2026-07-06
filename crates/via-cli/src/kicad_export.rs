use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(crate) struct KicadExportSummary {
    pub(crate) generated_footprints: usize,
    pub(crate) manual_footprints: usize,
}

pub(crate) fn write_artifacts(
    board: &via_core::Board,
    out: &Path,
    footprint_export: Option<via_project::KicadFootprintExport>,
    project_name: &str,
    footprint_cache_version: &str,
) -> via_core::Result<KicadExportSummary> {
    let stem = project_name;
    let footprint_summary = if let Some(footprint_export) = &footprint_export {
        write_embedded_footprints(board, &footprint_export.output_dir, footprint_cache_version)?
    } else {
        KicadExportSummary {
            generated_footprints: 0,
            manual_footprints: 0,
        }
    };
    via_kicad::write_netlist(board, out.join(format!("{stem}.net")))?;
    let mut options = via_kicad::SchematicProjectOptions::new(stem);
    if let Some(footprint_export) = footprint_export {
        options =
            options.footprint_library(footprint_export.library_name, footprint_export.library_path);
    }
    via_kicad::write_schematic_project(board, out, &options)?;

    Ok(footprint_summary)
}

pub(crate) fn load_required_official_footprint_texts(
    board: &via_core::Board,
    version: &str,
) -> via_core::Result<BTreeMap<String, String>> {
    let official_ids = required_official_footprint_map(board)?;
    if official_ids.is_empty() {
        return Ok(BTreeMap::new());
    }

    let cache = via_kicad_footprints::FootprintCache::open(version)
        .map_err(|err| via_core::Error::Io(err.to_string()))?;
    let mut texts = BTreeMap::new();
    for (local_name, id) in official_ids {
        let text = cache
            .footprint_text(&id)
            .map_err(|err| via_core::Error::Io(err.to_string()))?;
        texts.insert(local_name, text);
    }
    Ok(texts)
}

fn required_official_footprint_map(
    board: &via_core::Board,
) -> via_core::Result<BTreeMap<String, via_kicad_footprints::FootprintId>> {
    let mut official = BTreeMap::new();
    for footprint in board
        .footprints()
        .filter(|footprint| footprint.ir().is_none())
    {
        let Some(via_core::FootprintAsset::KicadLibrary { library, name }) = footprint.asset()
        else {
            continue;
        };
        if name != footprint.name() {
            return Err(via_core::Error::Io(format!(
                "KiCad footprint aliasing is not supported: local footprint {} points to {}:{}",
                footprint.name(),
                library,
                name
            )));
        }
        official.insert(
            footprint.name().to_owned(),
            via_kicad_footprints::FootprintId::new(library.clone(), name.clone()),
        );
    }
    Ok(official)
}

fn write_embedded_footprints(
    board: &via_core::Board,
    pretty_dir: &Path,
    footprint_cache_version: &str,
) -> via_core::Result<KicadExportSummary> {
    let exportable = validate_local_footprint_exports(board)?;
    std::fs::create_dir_all(pretty_dir).map_err(|err| via_core::Error::Io(err.to_string()))?;
    let official_ids = required_official_footprint_map(board)?;
    let official_cache = if official_ids.is_empty() {
        None
    } else {
        Some(
            via_kicad_footprints::FootprintCache::open(footprint_cache_version)
                .map_err(|err| via_core::Error::Io(err.to_string()))?,
        )
    };
    let mut generated_footprints = 0;
    let mut manual_footprints = 0;
    for footprint in board.footprints() {
        if let Some(ir) = footprint.ir() {
            let file_name = via_kicad_footprints::footprint_file_name(footprint.name())
                .map_err(|err| via_core::Error::Io(err.to_string()))?;
            ir.write_kicad_mod(pretty_dir.join(file_name))
                .map_err(|err| via_core::Error::Io(err.to_string()))?;
            generated_footprints += 1;
            continue;
        }

        if let Some(id) = official_ids.get(footprint.name())
            && let Some(cache) = &official_cache
        {
            cache
                .copy_footprint_to_pretty_dir(id, pretty_dir)
                .map_err(|err| via_core::Error::Io(err.to_string()))?;
            manual_footprints += 1;
        }
    }
    prune_stale_footprints(pretty_dir, &exportable)?;
    Ok(KicadExportSummary {
        generated_footprints,
        manual_footprints,
    })
}

fn validate_local_footprint_exports(board: &via_core::Board) -> via_core::Result<BTreeSet<String>> {
    let mut exportable = BTreeSet::new();
    for footprint in board.footprints() {
        let can_export = footprint.ir().is_some()
            || matches!(
                footprint.asset(),
                Some(via_core::FootprintAsset::KicadLibrary { .. })
            );
        if can_export {
            exportable.insert(footprint.name().to_owned());
        } else {
            return Err(via_core::Error::Io(format!(
                "footprint {} only has pad metadata; it cannot be written to the local KiCad library",
                footprint.name()
            )));
        }
    }

    for module in board.modules() {
        let Some(name) = module.footprint_name() else {
            continue;
        };
        if !exportable.contains(name) {
            return Err(via_core::Error::Io(format!(
                "{} references footprint {} but no exportable footprint asset or generated IR was loaded",
                module.refdes(),
                name
            )));
        }
    }
    Ok(exportable)
}

fn prune_stale_footprints(
    pretty_dir: &Path,
    expected: &BTreeSet<String>,
) -> via_core::Result<usize> {
    let mut removed = 0usize;
    for entry in
        std::fs::read_dir(pretty_dir).map_err(|err| via_core::Error::Io(err.to_string()))?
    {
        let entry = entry.map_err(|err| via_core::Error::Io(err.to_string()))?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("kicad_mod") {
            continue;
        }
        let Some(name) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        if expected.contains(name) {
            continue;
        }
        std::fs::remove_file(&path).map_err(|err| via_core::Error::Io(err.to_string()))?;
        removed += 1;
    }
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use via_core::{Design, FootprintPads, part, pin};

    #[test]
    fn local_footprint_export_rejects_missing_footprint_data() {
        let mut design = Design::new("missing_export_footprint");
        let module = design
            .add(
                part("J1", "connector")
                    .footprint("Missing")
                    .pin(pin("1").passive())
                    .pin(pin("2").passive()),
            )
            .unwrap();
        design
            .net("N")
            .connect_all(&mut design, [module.pin("1"), module.pin("2")]);
        let board = design.build().unwrap();

        let err = validate_local_footprint_exports(&board).unwrap_err();
        assert!(format!("{err}").contains("no exportable footprint asset or generated IR"));
    }

    #[test]
    fn local_footprint_export_rejects_pad_metadata_only() {
        let mut design = Design::new("pad_metadata_only");
        design.add_footprint_pads(FootprintPads::new("PadOnly", ["1", "2"]));
        let module = design
            .add(
                part("J1", "connector")
                    .footprint("PadOnly")
                    .pin(pin("1").passive())
                    .pin(pin("2").passive()),
            )
            .unwrap();
        design
            .net("N")
            .connect_all(&mut design, [module.pin("1"), module.pin("2")]);
        let board = design.build().unwrap();

        let err = validate_local_footprint_exports(&board).unwrap_err();
        assert!(format!("{err}").contains("only has pad metadata"));
    }

    #[test]
    fn stale_local_footprint_files_are_pruned() {
        let root =
            std::env::temp_dir().join(format!("via_kicad_export_prune_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("Keep.kicad_mod"), "").unwrap();
        std::fs::write(root.join("Stale.kicad_mod"), "").unwrap();
        std::fs::write(root.join("notes.txt"), "").unwrap();
        let expected = BTreeSet::from(["Keep".to_owned()]);

        let removed = prune_stale_footprints(&root, &expected).unwrap();

        assert_eq!(removed, 1);
        assert!(root.join("Keep.kicad_mod").exists());
        assert!(!root.join("Stale.kicad_mod").exists());
        assert!(root.join("notes.txt").exists());
        let _ = std::fs::remove_dir_all(root);
    }
}
