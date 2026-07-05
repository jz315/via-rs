use crate::{
    FootprintIr, FootprintProperty, FootprintValidationError, GraphicLine, GraphicText, Pad,
    PadKind, PadShape, TextJustify, TextKind,
};

pub fn render_kicad_mod(footprint: &FootprintIr) -> String {
    try_render_kicad_mod(footprint).expect("render_kicad_mod requires a valid footprint")
}

pub fn try_render_kicad_mod(footprint: &FootprintIr) -> Result<String, FootprintValidationError> {
    footprint.validate()?;

    let mut out = String::new();
    out.push_str(&format!("(footprint \"{}\"\n", escape(footprint.name())));
    out.push_str("  (version 20240108)\n");
    out.push_str("  (generator \"via-footprint-ir\")\n");
    if let Some(description) = footprint.description_text() {
        out.push_str(&format!("  (descr \"{}\")\n", escape(description)));
    }
    if !footprint.tags().is_empty() {
        out.push_str(&format!(
            "  (tags \"{}\")\n",
            escape(&footprint.tags().join(" "))
        ));
    }
    out.push_str("  (attr through_hole)\n");
    for property in footprint.properties() {
        out.push_str(&render_property(property));
    }
    for line in footprint.lines() {
        out.push_str(&render_line(line));
    }
    for text in footprint.texts() {
        out.push_str(&render_text(text));
    }
    for pad in footprint.pads() {
        out.push_str(&render_pad(pad));
    }
    out.push_str(")\n");
    Ok(out)
}

fn render_property(property: &FootprintProperty) -> String {
    format!(
        "  (property \"{}\" \"{}\" (at 0 0 0) (layer \"F.Fab\") hide (effects (font (size 1 1) (thickness 0.15))) (uuid \"{}\"))\n",
        escape(&property.name),
        escape(&property.value),
        stable_uuid(&format!("property:{}:{}", property.name, property.value))
    )
}

fn render_text(text: &GraphicText) -> String {
    let justify = match text.justify {
        Some(TextJustify::Left) => " (justify left)",
        Some(TextJustify::Right) => " (justify right)",
        Some(TextJustify::Center) | None => "",
    };
    let kind = match text.kind {
        TextKind::Reference => "reference",
        TextKind::Value => "value",
        TextKind::User => "user",
    };

    format!(
        "  (fp_text {} \"{}\" (at {} {} {}) (layer \"{}\") (effects (font (size {} {}) (thickness {})){}) (uuid \"{}\"))\n",
        kind,
        escape(&text.text),
        fmt_num(text.at.x),
        fmt_num(text.at.y),
        fmt_num(text.rotation),
        escape(&text.layer),
        fmt_num(text.size.x),
        fmt_num(text.size.y),
        fmt_num(text.thickness),
        justify,
        stable_uuid(&format!(
            "text:{}:{}:{}:{}",
            text.text, text.at.x, text.at.y, text.layer
        ))
    )
}

fn render_line(line: &GraphicLine) -> String {
    format!(
        "  (fp_line (start {} {}) (end {} {}) (stroke (width {}) (type solid)) (layer \"{}\") (uuid \"{}\"))\n",
        fmt_num(line.start.x),
        fmt_num(line.start.y),
        fmt_num(line.end.x),
        fmt_num(line.end.y),
        fmt_num(line.width),
        escape(&line.layer),
        stable_uuid(&format!(
            "line:{}:{}:{}:{}:{}",
            line.start.x, line.start.y, line.end.x, line.end.y, line.layer
        ))
    )
}

fn render_pad(pad: &Pad) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "  (pad \"{}\" {} {} (at {} {}) (size {} {})",
        escape(&pad.number),
        match pad.kind {
            PadKind::ThruHole => "thru_hole",
            PadKind::NpThruHole => "np_thru_hole",
            PadKind::Smd => "smd",
        },
        match pad.shape {
            PadShape::Circle => "circle",
            PadShape::Oval => "oval",
            PadShape::Rect => "rect",
            PadShape::RoundRect => "roundrect",
            PadShape::Trapezoid => "trapezoid",
        },
        fmt_num(pad.at.x),
        fmt_num(pad.at.y),
        fmt_num(pad.size.x),
        fmt_num(pad.size.y)
    ));
    if let Some(drill) = pad.drill {
        if drill.is_round() {
            out.push_str(&format!(" (drill {})", fmt_num(drill.x)));
        } else {
            out.push_str(&format!(
                " (drill oval {} {})",
                fmt_num(drill.x),
                fmt_num(drill.y)
            ));
        }
    }
    if matches!(pad.shape, PadShape::RoundRect) {
        out.push_str(" (roundrect_rratio 0.25)");
    }
    out.push_str(&format!(
        " (layers {})",
        pad.layers
            .iter()
            .map(|layer| format!("\"{}\"", escape(layer)))
            .collect::<Vec<_>>()
            .join(" ")
    ));
    out.push_str(&format!(
        " (uuid \"{}\"))\n",
        stable_uuid(&format!("pad:{}:{}:{}", pad.number, pad.at.x, pad.at.y))
    ));
    out
}

fn stable_uuid(seed: &str) -> String {
    let mut bytes = [0_u8; 16];
    bytes[..8].copy_from_slice(&fnv1a64(seed.as_bytes()).to_be_bytes());
    bytes[8..]
        .copy_from_slice(&fnv1a64(format!("via-footprint-ir:{seed}").as_bytes()).to_be_bytes());
    bytes[6] = (bytes[6] & 0x0f) | 0x50;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        bytes[6],
        bytes[7],
        bytes[8],
        bytes[9],
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15]
    )
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn fmt_num(value: f64) -> String {
    let mut text = format!("{value:.3}");
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    if text == "-0" { "0".to_owned() } else { text }
}
