use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use via_core::{Design, FootprintPads, Result};

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

    Ok(footprint_pads_from_kicad_mod_text(name, &text).with_source(path.to_path_buf()))
}

pub fn footprint_pads_from_kicad_mod_text(name: impl Into<String>, text: &str) -> FootprintPads {
    FootprintPads::new(name, parse_kicad_mod_pad_names(text))
}

pub fn parse_kicad_mod_pad_names(text: &str) -> BTreeSet<String> {
    let mut pads = BTreeSet::new();
    let bytes = text.as_bytes();
    let mut index = 0;

    while let Some(offset) = text[index..].find("(pad") {
        index += offset + "(pad".len();
        skip_ascii_whitespace(bytes, &mut index);

        if index >= bytes.len() {
            break;
        }

        let pad_name = if bytes[index] == b'"' {
            parse_quoted_atom(bytes, &mut index)
        } else {
            parse_bare_atom(bytes, &mut index)
        };

        if let Some(pad_name) = pad_name
            && !pad_name.is_empty()
        {
            pads.insert(pad_name);
        }
    }

    pads
}

fn skip_ascii_whitespace(bytes: &[u8], index: &mut usize) {
    while *index < bytes.len() && bytes[*index].is_ascii_whitespace() {
        *index += 1;
    }
}

fn parse_quoted_atom(bytes: &[u8], index: &mut usize) -> Option<String> {
    if bytes.get(*index) != Some(&b'"') {
        return None;
    }
    *index += 1;

    let mut out = String::new();
    while *index < bytes.len() {
        let byte = bytes[*index];
        *index += 1;

        match byte {
            b'\\' if *index < bytes.len() => {
                out.push(bytes[*index] as char);
                *index += 1;
            }
            b'"' => return Some(out),
            _ => out.push(byte as char),
        }
    }

    Some(out)
}

fn parse_bare_atom(bytes: &[u8], index: &mut usize) -> Option<String> {
    let start = *index;
    while *index < bytes.len() {
        let byte = bytes[*index];
        if byte.is_ascii_whitespace() || byte == b')' || byte == b'(' {
            break;
        }
        *index += 1;
    }

    if *index == start {
        None
    } else {
        Some(String::from_utf8_lossy(&bytes[start..*index]).into_owned())
    }
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
        );

        assert!(pads.contains("1"));
        assert!(pads.contains("A2"));
        assert!(pads.contains("GND 1"));
    }
}
