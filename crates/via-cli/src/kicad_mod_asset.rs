use std::collections::BTreeMap;

use via_core::model::Part;

use crate::kicad_sexp::{self, Sexp};

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

pub(crate) fn render(input: AssetFootprintRender<'_>) -> via_core::Result<String> {
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

    let source = kicad_sexp::parse_one(kicad_mod).map_err(|err| {
        via_core::Error::Io(format!(
            "failed to parse KiCad footprint asset {footprint_name}: {err}"
        ))
    })?;
    let source_items = match source {
        Sexp::List(items) if items.first().and_then(Sexp::as_atom) == Some("footprint") => items,
        _ => {
            return Err(via_core::Error::Io(format!(
                "KiCad footprint asset {footprint_name} does not start with a footprint node"
            )));
        }
    };

    let mut children = vec![
        Sexp::atom("footprint"),
        Sexp::string(format!("{footprint_library_name}:{footprint_name}")),
        list1("layer", Sexp::string("F.Cu")),
        list1(
            "uuid",
            Sexp::string(stable_uuid(&format!("footprint:{}", module.refdes()))),
        ),
        Sexp::list(vec![
            Sexp::atom("at"),
            Sexp::atom(n(x)),
            Sexp::atom(n(y)),
            Sexp::atom(n(rotation)),
        ]),
    ];
    let mut uuid_index = 0usize;
    let mut has_datasheet_property = false;
    let mut has_description_property = false;

    for child in source_items.into_iter().skip(2) {
        let Some(head) = child.list_name().map(str::to_owned) else {
            children.push(child);
            continue;
        };
        if is_skipped_footprint_header(&head) {
            continue;
        }

        let mut child = child;
        match head.as_str() {
            "property" => {
                if let Some(name) = property_name(&child).map(str::to_owned) {
                    match name.as_str() {
                        "Datasheet" => has_datasheet_property = true,
                        "Description" => has_description_property = true,
                        "Reference" => rewrite_property(&mut child, module.refdes(), false),
                        "Value" => rewrite_property(&mut child, module.value(), true),
                        _ => {}
                    }
                }
            }
            "fp_text" => match fp_text_kind(&child) {
                Some("reference") => rewrite_fp_text(&mut child, module.refdes(), false),
                Some("value") => rewrite_fp_text(&mut child, module.value(), true),
                _ => {}
            },
            "pad" => rewrite_pad(&mut child, module, net_ids, pad_nets),
            _ => {}
        }

        if let Some(kind) = uuid_kind(&child) {
            let uuid = stable_uuid(&format!(
                "asset:{}:{}:{}:{}",
                module.refdes(),
                footprint_name,
                kind,
                uuid_index
            ));
            uuid_index += 1;
            set_child_list(&mut child, "uuid", vec![Sexp::string(uuid)]);
        }
        children.push(child);
    }

    if !has_datasheet_property {
        children.push(standard_property(
            "Datasheet",
            "",
            &stable_uuid(&format!("datasheet:{}", module.refdes())),
            1.27,
        ));
    }
    if !has_description_property {
        children.push(standard_property(
            "Description",
            "",
            &stable_uuid(&format!("description:{}", module.refdes())),
            1.27,
        ));
    }
    if module.requires_verification() {
        children.push(standard_property(
            "VIA_VERIFY",
            "true",
            &stable_uuid(&format!("verify:{}", module.refdes())),
            1.0,
        ));
    }

    Ok(kicad_sexp::render(&Sexp::list(children), 2))
}

fn is_skipped_footprint_header(head: &str) -> bool {
    matches!(head, "version" | "generator" | "layer")
}

fn property_name(node: &Sexp) -> Option<&str> {
    let Sexp::List(items) = node else {
        return None;
    };
    items.get(1).and_then(Sexp::as_atom)
}

fn fp_text_kind(node: &Sexp) -> Option<&str> {
    let Sexp::List(items) = node else {
        return None;
    };
    items.get(1).and_then(Sexp::as_atom)
}

fn rewrite_property(node: &mut Sexp, value: &str, hide: bool) {
    let Sexp::List(items) = node else {
        return;
    };
    if items.len() >= 3 {
        items[2] = Sexp::string(value);
    }
    if hide {
        ensure_property_hidden(items);
    }
}

fn rewrite_fp_text(node: &mut Sexp, value: &str, hide: bool) {
    let Sexp::List(items) = node else {
        return;
    };
    if items.len() >= 3 {
        items[2] = Sexp::string(value);
    }
    if hide && !items.iter().any(|item| item.as_atom() == Some("hide")) {
        items.push(Sexp::atom("hide"));
    }
}

fn rewrite_pad(
    node: &mut Sexp,
    module: &Part,
    net_ids: &BTreeMap<String, usize>,
    pad_nets: &BTreeMap<(String, String), (String, String)>,
) {
    let Some(pad) = pad_name(node).map(str::to_owned) else {
        return;
    };
    let (net_name, pin_name) = pad_nets
        .get(&(module.refdes().to_owned(), pad))
        .cloned()
        .unwrap_or_else(|| (String::new(), String::new()));
    let net = net_name
        .is_empty()
        .then_some(0)
        .or_else(|| net_ids.get(&net_name).copied())
        .unwrap_or(0);

    if net_name.is_empty() {
        remove_child_lists(node, "net");
    } else {
        set_child_list(
            node,
            "net",
            vec![Sexp::atom(net.to_string()), Sexp::string(net_name)],
        );
    }
    if pin_name.is_empty() {
        remove_child_lists(node, "pinfunction");
    } else {
        set_child_list(node, "pinfunction", vec![Sexp::string(pin_name)]);
    }
    set_child_list(node, "pintype", vec![Sexp::string("passive")]);
}

fn pad_name(node: &Sexp) -> Option<&str> {
    let Sexp::List(items) = node else {
        return None;
    };
    items.get(1).and_then(Sexp::as_atom)
}

fn ensure_property_hidden(items: &mut Vec<Sexp>) {
    if !items.iter().any(|item| item.list_name() == Some("hide")) {
        items.push(Sexp::list(vec![Sexp::atom("hide"), Sexp::atom("yes")]));
    }
}

fn set_child_list(node: &mut Sexp, name: &str, args: Vec<Sexp>) {
    let Sexp::List(items) = node else {
        return;
    };
    if let Some(child) = items
        .iter_mut()
        .find(|child| child.list_name() == Some(name))
    {
        let mut new_items = vec![Sexp::atom(name)];
        new_items.extend(args);
        *child = Sexp::list(new_items);
        return;
    }

    let mut new_items = vec![Sexp::atom(name)];
    new_items.extend(args);
    items.push(Sexp::list(new_items));
}

fn remove_child_lists(node: &mut Sexp, name: &str) {
    let Sexp::List(items) = node else {
        return;
    };
    items.retain(|child| child.list_name() != Some(name));
}

fn standard_property(name: &str, value: &str, uuid: &str, font_size: f64) -> Sexp {
    Sexp::list(vec![
        Sexp::atom("property"),
        Sexp::string(name),
        Sexp::string(value),
        Sexp::list(vec![
            Sexp::atom("at"),
            Sexp::atom("0"),
            Sexp::atom("0"),
            Sexp::atom("0"),
        ]),
        list1("layer", Sexp::string("F.Fab")),
        Sexp::list(vec![Sexp::atom("hide"), Sexp::atom("yes")]),
        list1("uuid", Sexp::string(uuid)),
        effects(font_size),
    ])
}

fn effects(font_size: f64) -> Sexp {
    Sexp::list(vec![
        Sexp::atom("effects"),
        Sexp::list(vec![
            Sexp::atom("font"),
            Sexp::list(vec![
                Sexp::atom("size"),
                Sexp::atom(n(font_size)),
                Sexp::atom(n(font_size)),
            ]),
            Sexp::list(vec![Sexp::atom("thickness"), Sexp::atom("0.15")]),
        ]),
    ])
}

fn list1(name: &str, value: Sexp) -> Sexp {
    Sexp::list(vec![Sexp::atom(name), value])
}

fn uuid_kind(node: &Sexp) -> Option<&'static str> {
    match node.list_name()? {
        "property" => Some("property"),
        "fp_text" => Some("fp_text"),
        "fp_line" => Some("fp_line"),
        "fp_rect" => Some("fp_rect"),
        "fp_circle" => Some("fp_circle"),
        "fp_arc" => Some("fp_arc"),
        "fp_poly" => Some("fp_poly"),
        "pad" => Some("pad"),
        "zone" => Some("zone"),
        _ => None,
    }
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
            })
            .unwrap(),
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
            .unwrap()
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

    #[test]
    fn asset_renderer_handles_quoted_parentheses_in_source_strings() {
        let kicad_mod = r#"(footprint "Fixture"
  (version 20240101)
  (generator "fixture")
  (layer "F.Cu")
  (descr "official text with (parentheses)")
  (tags "tag with ) and ( chars")
  (property "Reference" "REF**" (at 0 0 0) (layer "F.SilkS"))
  (property "Value" "Fixture" (at 0 1 0) (layer "F.Fab"))
  (property "VendorNote" "keep (this) intact" (at 0 2 0) (layer "F.Fab"))
  (pad "1" thru_hole circle (at 0 0) (size 1 1) (drill 0.5) (layers "*.Cu" "*.Mask"))
)"#;
        let net_ids = BTreeMap::from([("NET(1)".to_owned(), 7usize)]);
        let pad_nets = BTreeMap::from([(
            ("C1".to_owned(), "1".to_owned()),
            ("NET(1)".to_owned(), "P(+)".to_owned()),
        )]);
        let c1 = Part::new("C1", "100uF").pins(["P(+)"]).map_pin("P(+)", "1");

        let rendered = render(AssetFootprintRender {
            footprint_name: "Fixture",
            kicad_mod,
            module: &c1,
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            net_ids: &net_ids,
            pad_nets: &pad_nets,
            footprint_library_name: "FixtureLib",
        })
        .unwrap();

        assert!(rendered.contains("(descr \"official text with (parentheses)\""));
        assert!(rendered.contains("(tags \"tag with ) and ( chars\""));
        assert!(rendered.contains("(property \"VendorNote\" \"keep (this) intact\""));
        assert!(rendered.contains("(net 7 \"NET(1)\""));
        assert!(rendered.contains("(pinfunction \"P(+)\""));
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
