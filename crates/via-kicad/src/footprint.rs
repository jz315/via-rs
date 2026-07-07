use std::fs;
use std::path::Path;

use via_core::{Design, FootprintPads, Result};
use via_kicad_sexp::{self, Sexp};

pub fn load_kicad_footprint(design: &mut Design, path: impl AsRef<Path>) -> Result<()> {
    design.add_footprint_pads(footprint_pads_from_kicad_mod(path)?);
    Ok(())
}

pub fn load_kicad_footprint_dir(design: &mut Design, path: impl AsRef<Path>) -> Result<usize> {
    let mut count = 0;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("kicad_mod") {
            load_kicad_footprint(design, path)?;
            count += 1;
        }
    }
    Ok(count)
}

pub fn footprint_pads_from_kicad_mod(path: impl AsRef<Path>) -> Result<FootprintPads> {
    let path = path.as_ref();
    let text = fs::read_to_string(path)?;
    let name = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unknown".to_owned());

    Ok(footprint_pads_from_kicad_mod_text(name, &text)?.with_source(path.to_path_buf()))
}

pub fn footprint_pads_from_kicad_mod_text(
    name: impl Into<String>,
    text: &str,
) -> Result<FootprintPads> {
    Ok(FootprintPads::new(name, parse_kicad_mod_pad_names(text)?))
}

pub fn parse_kicad_mod_pad_names(text: &str) -> Result<std::collections::BTreeSet<String>> {
    let source = via_kicad_sexp::parse_one(text).map_err(|err| {
        via_core::Error::Io(format!(
            "failed to parse KiCad footprint S-expression: {err}"
        ))
    })?;
    let Sexp::List(items) = source else {
        return Err(via_core::Error::Io(
            "KiCad footprint does not contain a footprint node".to_owned(),
        ));
    };
    if items.first().and_then(Sexp::as_atom) != Some("footprint") {
        return Err(via_core::Error::Io(
            "KiCad footprint does not start with a footprint node".to_owned(),
        ));
    }

    Ok(items
        .iter()
        .filter(|item| item.list_name() == Some("pad"))
        .filter_map(pad_name)
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect())
}

fn pad_name(node: &Sexp) -> Option<&str> {
    let Sexp::List(items) = node else {
        return None;
    };
    items.get(1).and_then(Sexp::as_atom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_quoted_and_bare_pad_names() {
        let pads = parse_kicad_mod_pad_names(
            r#"
            (footprint "Demo"
              (pad "1" thru_hole rect (at 0 0))
              (pad A2 smd rect (at 1 0))
              (pad "GND 1" thru_hole circle (at 2 0))
            )
            "#,
        )
        .unwrap();

        assert!(pads.contains("1"));
        assert!(pads.contains("A2"));
        assert!(pads.contains("GND 1"));
    }

    #[test]
    fn ignores_pad_text_in_comments_and_strings() {
        let pads = parse_kicad_mod_pad_names(
            r#"
            (footprint "Demo"
              ; (pad "COMMENT" smd rect (at 0 0))
              (descr "not real: (pad \"STRING\" smd rect)")
              (fp_text user "(pad FAKE)" (at 0 0) (layer "F.SilkS"))
              (pad "1" thru_hole rect (at 0 0))
            )
            "#,
        )
        .unwrap();

        assert_eq!(pads, std::collections::BTreeSet::from(["1".to_owned()]));
    }

    #[test]
    fn rejects_non_footprint_text() {
        let err = parse_kicad_mod_pad_names("(not_footprint)").unwrap_err();

        assert!(format!("{err}").contains("does not start with a footprint node"));
    }
}
