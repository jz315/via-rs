use std::collections::BTreeMap;

use via_core::model::Part;

use crate::json::escape_json;

pub(crate) struct AssetFootprintRender<'a> {
    pub(crate) footprint_name: &'a str,
    pub(crate) kicad_mod: &'a str,
    pub(crate) module: &'a Part,
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) rotation: f64,
    pub(crate) net_ids: &'a BTreeMap<String, usize>,
    pub(crate) pad_nets: &'a BTreeMap<(String, String), (String, String)>,
    pub(crate) footprint_library_name: &'a str,
}

pub(crate) fn render(input: AssetFootprintRender<'_>) -> String {
    let AssetFootprintRender {
        footprint_name,
        kicad_mod,
        module,
        x,
        y,
        rotation,
        net_ids,
        pad_nets,
        footprint_library_name,
    } = input;
    let mut out = String::new();
    out.push_str(&format!(
        "  (footprint \"{}:{}\"\n",
        escape_sexp(footprint_library_name),
        escape_sexp(footprint_name)
    ));
    out.push_str("    (layer \"F.Cu\")\n");
    out.push_str(&format!(
        "    (uuid \"{}\")\n",
        stable_uuid(&format!("footprint:{}", module.refdes()))
    ));
    out.push_str(&format!("    (at {} {} {})\n", n(x), n(y), n(rotation)));

    let lines = kicad_mod.lines().collect::<Vec<_>>();
    let mut idx = 1;
    let mut uuid_index = 0usize;
    let mut has_datasheet_property = false;
    let mut has_description_property = false;
    while idx + 1 < lines.len() {
        let line = lines[idx];
        let trimmed = line.trim_start();
        if trimmed.starts_with("(version ")
            || trimmed.starts_with("(generator ")
            || (idx <= 3 && trimmed.starts_with("(layer "))
        {
            idx += 1;
            continue;
        }

        if !trimmed.starts_with('(') {
            out.push_str("    ");
            out.push_str(line);
            out.push('\n');
            idx += 1;
            continue;
        }

        let block = collect_sexp_block(&lines, &mut idx);
        if trimmed.starts_with("(property \"Datasheet\" ") {
            has_datasheet_property = true;
        } else if trimmed.starts_with("(property \"Description\" ") {
            has_description_property = true;
        }
        let uuid = uuid_kind(block.first().copied().unwrap_or_default()).map(|kind| {
            let uuid = stable_uuid(&format!(
                "asset:{}:{}:{}:{}",
                module.refdes(),
                footprint_name,
                kind,
                uuid_index
            ));
            uuid_index += 1;
            uuid
        });

        if trimmed.starts_with("(pad ") {
            out.push_str(&render_asset_pad_block(
                &block,
                module,
                net_ids,
                pad_nets,
                uuid.as_deref(),
            ));
            continue;
        }

        if trimmed.starts_with("(property \"Value\" ") {
            out.push_str(&render_asset_property_block(
                &block,
                module.value(),
                true,
                uuid.as_deref(),
            ));
            continue;
        }

        if trimmed.starts_with("(fp_text value ") {
            out.push_str(&render_asset_fp_text_block(
                &block,
                module.value(),
                true,
                uuid.as_deref(),
            ));
            continue;
        }

        if trimmed.starts_with("(property \"Reference\" ") {
            out.push_str(&render_asset_property_block(
                &block,
                module.refdes(),
                false,
                uuid.as_deref(),
            ));
            continue;
        }

        if trimmed.starts_with("(fp_text reference ") {
            out.push_str(&render_asset_fp_text_block(
                &block,
                module.refdes(),
                false,
                uuid.as_deref(),
            ));
            continue;
        }

        out.push_str(&render_asset_block(&block, None, &[], uuid.as_deref()));
    }

    if !has_datasheet_property {
        out.push_str(&render_standard_property(
            "Datasheet",
            "",
            &stable_uuid(&format!("datasheet:{}", module.refdes())),
        ));
    }
    if !has_description_property {
        out.push_str(&render_standard_property(
            "Description",
            "",
            &stable_uuid(&format!("description:{}", module.refdes())),
        ));
    }
    if module.requires_verification() {
        out.push_str("    (property \"VIA_VERIFY\" \"true\" (at 0 0 0) (layer \"F.Fab\") (hide yes) (uuid \"");
        out.push_str(&stable_uuid(&format!("verify:{}", module.refdes())));
        out.push_str("\") (effects (font (size 1 1) (thickness 0.15))))\n");
    }
    out.push_str("  )\n");
    out
}

fn render_standard_property(name: &str, value: &str, uuid: &str) -> String {
    format!(
        "    (property \"{}\" \"{}\" (at 0 0 0) (layer \"F.Fab\") (hide yes) (uuid \"{}\") (effects (font (size 1.27 1.27))))\n",
        escape_sexp(name),
        escape_sexp(value),
        uuid
    )
}

fn collect_sexp_block<'a>(lines: &[&'a str], idx: &mut usize) -> Vec<&'a str> {
    let mut block = Vec::new();
    let mut depth = 0isize;
    while *idx < lines.len() {
        let line = lines[*idx];
        depth += line.chars().filter(|ch| *ch == '(').count() as isize;
        depth -= line.chars().filter(|ch| *ch == ')').count() as isize;
        block.push(line);
        *idx += 1;
        if depth <= 0 {
            break;
        }
    }
    block
}

fn render_asset_pad_block(
    block: &[&str],
    module: &Part,
    net_ids: &BTreeMap<String, usize>,
    pad_nets: &BTreeMap<(String, String), (String, String)>,
    uuid: Option<&str>,
) -> String {
    let Some(first_line) = block.first() else {
        return String::new();
    };
    let pad = parse_pad_name(first_line).unwrap_or_default();
    let (net_name, pin_name) = pad_nets
        .get(&(module.refdes().to_owned(), pad.clone()))
        .cloned()
        .unwrap_or_else(|| (String::new(), String::new()));
    let net = net_name
        .is_empty()
        .then_some(0)
        .or_else(|| net_ids.get(&net_name).copied())
        .unwrap_or(0);

    let mut insertions = Vec::new();
    if !net_name.is_empty() {
        insertions.push(format!("(net {net} \"{}\")", escape_sexp(&net_name)));
    }
    if !pin_name.is_empty() {
        insertions.push(format!("(pinfunction \"{}\")", escape_sexp(&pin_name)));
    }
    insertions.push("(pintype \"passive\")".to_owned());
    render_asset_block(block, None, &insertions, uuid)
}

fn render_asset_property_block(
    block: &[&str],
    value: &str,
    hide: bool,
    uuid: Option<&str>,
) -> String {
    let has_hide = block
        .iter()
        .any(|line| line.trim_start().starts_with("(hide "));
    let insertions = if hide && !has_hide {
        vec!["(hide yes)".to_owned()]
    } else {
        Vec::new()
    };
    render_asset_block(
        block,
        Some(&|line| replace_property_value_line(line, value)),
        &insertions,
        uuid,
    )
}

fn render_asset_fp_text_block(
    block: &[&str],
    value: &str,
    hide: bool,
    uuid: Option<&str>,
) -> String {
    let has_hide = block
        .iter()
        .any(|line| line.split_whitespace().any(|atom| atom == "hide"));
    let insertions = if hide && !has_hide {
        vec!["hide".to_owned()]
    } else {
        Vec::new()
    };
    render_asset_block(
        block,
        Some(&|line| replace_fp_text_value_line(line, value)),
        &insertions,
        uuid,
    )
}

fn render_asset_block(
    block: &[&str],
    rewrite_first_line: Option<&dyn Fn(&str) -> String>,
    insertions: &[String],
    uuid: Option<&str>,
) -> String {
    let mut lines = block
        .iter()
        .enumerate()
        .map(|(idx, line)| {
            if idx == 0 {
                rewrite_first_line
                    .map(|rewrite| rewrite(line))
                    .unwrap_or_else(|| (*line).to_owned())
            } else {
                (*line).to_owned()
            }
        })
        .collect::<Vec<_>>();

    let mut has_uuid = false;
    if let Some(uuid) = uuid {
        for line in &mut lines {
            if line.contains("(uuid \"") {
                *line = replace_uuid_line(line, uuid);
                has_uuid = true;
                break;
            }
        }
    }

    let mut child_insertions = insertions.to_vec();
    if let Some(uuid) = uuid
        && !has_uuid
    {
        child_insertions.push(format!("(uuid \"{uuid}\")"));
    }

    if !child_insertions.is_empty() {
        if lines.len() == 1 {
            let suffix = child_insertions
                .iter()
                .map(|insertion| format!(" {insertion}"))
                .collect::<String>();
            lines[0] = insert_before_final_paren(&lines[0], &suffix);
        } else {
            let insert_at = lines.len().saturating_sub(1);
            for insertion in child_insertions.into_iter().rev() {
                lines.insert(insert_at, format!("    {insertion}"));
            }
        }
    }

    let mut out = String::new();
    for line in lines {
        out.push_str("    ");
        out.push_str(&line);
        out.push('\n');
    }
    out
}

fn parse_pad_name(line: &str) -> Option<String> {
    let mut rest = line.trim_start().strip_prefix("(pad")?.trim_start();
    if rest.starts_with('"') {
        rest = &rest[1..];
        let end = rest.find('"')?;
        return Some(rest[..end].to_owned());
    }

    let end = rest
        .find(|ch: char| ch.is_ascii_whitespace() || ch == ')' || ch == '(')
        .unwrap_or(rest.len());
    (!rest[..end].is_empty()).then(|| rest[..end].to_owned())
}

fn replace_property_value_line(line: &str, value: &str) -> String {
    let Some(first_quote) = line.find('"') else {
        return line.to_owned();
    };
    let Some(second_quote) = line[first_quote + 1..].find('"') else {
        return line.to_owned();
    };
    let after_key = first_quote + 1 + second_quote + 1;
    let Some(value_start_offset) = line[after_key..].find('"') else {
        return line.to_owned();
    };
    let value_start = after_key + value_start_offset;
    let Some(value_end_offset) = line[value_start + 1..].find('"') else {
        return line.to_owned();
    };
    let value_end = value_start + 1 + value_end_offset;

    format!(
        "{}\"{}\"{}",
        &line[..value_start],
        escape_sexp(value),
        &line[value_end + 1..]
    )
}

fn replace_fp_text_value_line(line: &str, value: &str) -> String {
    let Some(value_start) = line.find('"') else {
        return line.to_owned();
    };
    let Some(value_end_offset) = line[value_start + 1..].find('"') else {
        return line.to_owned();
    };
    let value_end = value_start + 1 + value_end_offset;

    format!(
        "{}\"{}\"{}",
        &line[..value_start],
        escape_sexp(value),
        &line[value_end + 1..]
    )
}

fn replace_uuid_line(line: &str, uuid: &str) -> String {
    let Some(start) = line.find("(uuid \"") else {
        return line.to_owned();
    };
    let value_start = start + "(uuid \"".len();
    let Some(value_end_offset) = line[value_start..].find('"') else {
        return line.to_owned();
    };
    let value_end = value_start + value_end_offset;
    format!("{}{}{}", &line[..value_start], uuid, &line[value_end..])
}

fn insert_before_final_paren(line: &str, text: &str) -> String {
    let Some(close) = line.rfind(')') else {
        return format!("{line}{text}");
    };
    format!("{}{}{}", &line[..close], text, &line[close..])
}

fn uuid_kind(line: &str) -> Option<&'static str> {
    let trimmed = line.trim_start();
    if starts_sexp(trimmed, "property") {
        Some("property")
    } else if starts_sexp(trimmed, "fp_text") {
        Some("fp_text")
    } else if starts_sexp(trimmed, "fp_line") {
        Some("fp_line")
    } else if starts_sexp(trimmed, "fp_rect") {
        Some("fp_rect")
    } else if starts_sexp(trimmed, "fp_circle") {
        Some("fp_circle")
    } else if starts_sexp(trimmed, "fp_arc") {
        Some("fp_arc")
    } else if starts_sexp(trimmed, "fp_poly") {
        Some("fp_poly")
    } else if starts_sexp(trimmed, "pad") {
        Some("pad")
    } else if starts_sexp(trimmed, "zone") {
        Some("zone")
    } else {
        None
    }
}

fn starts_sexp(trimmed: &str, name: &str) -> bool {
    let Some(rest) = trimmed.strip_prefix('(') else {
        return false;
    };
    rest == name
        || rest
            .strip_prefix(name)
            .is_some_and(|suffix| suffix.starts_with(|ch: char| ch.is_ascii_whitespace()))
}

fn n(value: f64) -> String {
    let mut text = format!("{value:.4}");
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    if text == "-0" { "0".to_owned() } else { text }
}

fn escape_sexp(value: &str) -> String {
    escape_json(value)
}

fn stable_uuid(seed: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in seed.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!(
        "{:08x}-{:04x}-4{:03x}-8{:03x}-{:012x}",
        (hash >> 32) as u32,
        (hash >> 16) as u16,
        hash & 0x0fff,
        (hash >> 12) & 0x0fff,
        hash & 0x0000_ffff_ffff_ffff
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_renderer_reseeds_child_uuids_per_module() {
        let kicad_mod = r#"(footprint "Fixture"
  (version 20240101)
  (generator "fixture")
  (layer "F.Cu")
  (property "Reference" "REF**" (at 0 0 0) (layer "F.SilkS") (uuid "11111111-1111-4111-8111-111111111111"))
  (property "Value" "Fixture" (at 0 1 0) (layer "F.Fab"))
  (fp_text reference "REF**" (at 0 -1 0) (layer "F.SilkS") (uuid "22222222-2222-4222-8222-222222222222"))
  (fp_line
    (start 0 0)
    (end 1 1)
    (stroke (width 0.12) (type solid))
    (layer "F.SilkS")
    (uuid "33333333-3333-4333-8333-333333333333")
  )
  (pad "1" thru_hole circle
    (at 0 0)
    (size 1 1)
    (drill 0.5)
    (layers "*.Cu" "*.Mask")
    (uuid "44444444-4444-4444-8444-444444444444")
  )
)"#;
        let net_ids = BTreeMap::from([("NET".to_owned(), 1usize)]);
        let pad_nets = BTreeMap::from([
            (
                ("C1".to_owned(), "1".to_owned()),
                ("NET".to_owned(), "P".to_owned()),
            ),
            (
                ("C2".to_owned(), "1".to_owned()),
                ("NET".to_owned(), "P".to_owned()),
            ),
        ]);
        let c1 = Part::new("C1", "100uF").pins(["P"]).map_pin("P", "1");
        let c2 = Part::new("C2", "100uF").pins(["P"]).map_pin("P", "1");

        let rendered = format!(
            "{}{}",
            render(AssetFootprintRender {
                footprint_name: "Fixture",
                kicad_mod,
                module: &c1,
                x: 0.0,
                y: 0.0,
                rotation: 0.0,
                net_ids: &net_ids,
                pad_nets: &pad_nets,
                footprint_library_name: "FixtureLib",
            }),
            render(AssetFootprintRender {
                footprint_name: "Fixture",
                kicad_mod,
                module: &c2,
                x: 10.0,
                y: 0.0,
                rotation: 0.0,
                net_ids: &net_ids,
                pad_nets: &pad_nets,
                footprint_library_name: "FixtureLib",
            })
        );

        assert!(rendered.contains("(property \"Reference\" \"C1\""));
        assert!(rendered.contains("(property \"Reference\" \"C2\""));
        assert!(rendered.contains("(fp_text reference \"C1\""));
        assert!(rendered.contains("(fp_text reference \"C2\""));
        assert!(rendered.contains("(property \"Value\" \"100uF\""));
        assert!(rendered.contains("(property \"Datasheet\" \"\""));
        assert!(rendered.contains("(property \"Description\" \"\""));
        assert!(rendered.contains("(net 1 \"NET\")"));
        assert!(rendered.contains("(pinfunction \"P\")"));
        assert!(!rendered.contains("11111111-1111-4111-8111-111111111111"));
        assert!(!rendered.contains("22222222-2222-4222-8222-222222222222"));
        assert!(!rendered.contains("33333333-3333-4333-8333-333333333333"));
        assert!(!rendered.contains("44444444-4444-4444-8444-444444444444"));
        assert_unique_uuids(&rendered);
    }

    fn assert_unique_uuids(text: &str) {
        let mut seen = std::collections::BTreeSet::new();
        for uuid in text.lines().filter_map(extract_uuid) {
            assert_valid_uuid_shape(uuid);
            assert!(seen.insert(uuid.to_owned()), "duplicate uuid {uuid}");
        }
    }

    fn assert_valid_uuid_shape(uuid: &str) {
        let parts = uuid.split('-').collect::<Vec<_>>();
        assert_eq!(
            parts.iter().map(|part| part.len()).collect::<Vec<_>>(),
            [8, 4, 4, 4, 12],
            "invalid uuid shape {uuid}"
        );
    }

    fn extract_uuid(line: &str) -> Option<&str> {
        let start = line.find("(uuid \"")? + "(uuid \"".len();
        let rest = &line[start..];
        let end = rest.find('"')?;
        Some(&rest[..end])
    }
}
