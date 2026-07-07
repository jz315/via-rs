use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::Deserialize;
use via_core::{Board, model::Part};

use crate::json::escape_json;
use crate::kicad_mod_asset::{self, AssetFootprintRender};

#[derive(Debug, Deserialize)]
pub struct Layout {
    pub board: String,
    pub modules: Vec<LayoutModule>,
    #[serde(default)]
    pub outline: Option<LayoutOutline>,
    #[serde(default)]
    pub copper: LayoutCopper,
    #[serde(default)]
    pub tracks: Vec<LegacyTrack>,
}

#[derive(Debug, Deserialize)]
pub struct LayoutModule {
    pub refdes: String,
    pub x: f64,
    pub y: f64,
    #[serde(default)]
    pub rotation: f64,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LayoutOutline {
    pub points: Vec<Point>,
}

#[derive(Debug, Deserialize, Default)]
pub struct LayoutCopper {
    #[serde(default)]
    pub segments: Vec<LayoutSegment>,
    #[serde(default)]
    pub vias: Vec<LayoutVia>,
}

#[derive(Debug, Deserialize)]
pub struct LayoutSegment {
    pub id: String,
    pub net: String,
    pub layer: String,
    pub width: f64,
    pub a: Point,
    pub b: Point,
}

#[derive(Debug, Deserialize)]
pub struct LegacyTrack {
    pub net: String,
    pub layer: String,
    pub width: f64,
    pub points: Vec<Point>,
}

#[derive(Debug, Deserialize)]
pub struct LayoutVia {
    pub id: String,
    pub net: String,
    pub x: f64,
    pub y: f64,
    pub drill: f64,
    pub diameter: f64,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

pub fn read_layout(path: &Path) -> via_core::Result<Layout> {
    let text = std::fs::read_to_string(path).map_err(|err| via_core::Error::Io(err.to_string()))?;
    let mut layout: Layout =
        serde_json::from_str(&text).map_err(|err| via_core::Error::Io(err.to_string()))?;
    if layout.copper.segments.is_empty() && !layout.tracks.is_empty() {
        for (track_idx, track) in layout.tracks.iter().enumerate() {
            for idx in 1..track.points.len() {
                layout.copper.segments.push(LayoutSegment {
                    id: format!("legacy-{track_idx}-{idx}"),
                    net: track.net.clone(),
                    layer: track.layer.clone(),
                    width: track.width,
                    a: track.points[idx - 1],
                    b: track.points[idx],
                });
            }
        }
    }
    Ok(layout)
}

pub fn write_kicad_pcb(
    board: &Board,
    layout: &Layout,
    out: &Path,
    footprint_library_name: &str,
    official_footprints: &BTreeMap<String, String>,
) -> via_core::Result<()> {
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).map_err(|err| via_core::Error::Io(err.to_string()))?;
    }
    let text = render_kicad_pcb(board, layout, footprint_library_name, official_footprints)?;
    std::fs::write(out, text).map_err(|err| via_core::Error::Io(err.to_string()))
}

fn render_kicad_pcb(
    board: &Board,
    layout: &Layout,
    footprint_library_name: &str,
    official_footprints: &BTreeMap<String, String>,
) -> via_core::Result<String> {
    let net_ids = net_ids(board);
    validate_layout(board, layout, &net_ids)?;
    let pad_nets = pad_net_map(board);
    let mut out = String::new();
    out.push_str("(kicad_pcb\n");
    out.push_str("  (version 20240108)\n");
    out.push_str("  (generator \"via\")\n");
    out.push_str(&format!(
        "  (generator_version \"{}\")\n",
        env!("CARGO_PKG_VERSION")
    ));
    out.push_str("  (general\n    (thickness 1.6)\n  )\n");
    out.push_str("  (paper \"A4\")\n");
    out.push_str("  (layers\n");
    out.push_str("    (0 \"F.Cu\" signal)\n");
    out.push_str("    (31 \"B.Cu\" signal)\n");
    out.push_str("    (32 \"B.Adhes\" user)\n");
    out.push_str("    (33 \"F.Adhes\" user)\n");
    out.push_str("    (34 \"B.Paste\" user)\n");
    out.push_str("    (35 \"F.Paste\" user)\n");
    out.push_str("    (36 \"B.SilkS\" user)\n");
    out.push_str("    (37 \"F.SilkS\" user)\n");
    out.push_str("    (38 \"B.Mask\" user)\n");
    out.push_str("    (39 \"F.Mask\" user)\n");
    out.push_str("    (44 \"Edge.Cuts\" user)\n");
    out.push_str("  )\n");
    out.push_str(&render_setup(board));
    for (name, id) in &net_ids {
        out.push_str(&format!("  (net {id} \"{}\")\n", escape_sexp(name)));
    }
    out.push('\n');

    let footprint_irs = generated_footprint_irs(board);
    for placement in &layout.modules {
        if placement.status.as_deref() == Some("missing") {
            continue;
        }
        let module = board.module(&placement.refdes).ok_or_else(|| {
            via_core::Error::Io(format!(
                "PCB layout references unknown module {}",
                placement.refdes
            ))
        })?;
        out.push_str(&render_footprint(
            module,
            placement,
            &net_ids,
            &pad_nets,
            footprint_irs.get(module.footprint_name().unwrap_or("")),
            footprint_library_name,
            official_footprints,
        )?);
    }

    let outline_points = outline_points(layout);
    if !outline_points.is_empty() {
        for (idx, start) in outline_points.iter().enumerate() {
            if outline_points.len() < 2 {
                break;
            }
            let end = outline_points[(idx + 1) % outline_points.len()];
            out.push_str(&format!(
                "  (gr_line (start {} {}) (end {} {}) (stroke (width 0.1) (type default)) (layer \"Edge.Cuts\") (uuid \"{}\"))\n",
                n(start.x),
                n(start.y),
                n(end.x),
                n(end.y),
                stable_uuid(&format!("edge:{idx}:{}:{}", start.x, start.y)),
            ));
        }
    }

    for segment in &layout.copper.segments {
        let net = layout_net_id(&net_ids, &segment.net, "segment", &segment.id)?;
        out.push_str(&format!(
            "  (segment (start {} {}) (end {} {}) (width {}) (layer \"{}\") (net {net}) (uuid \"{}\"))\n",
            n(segment.a.x),
            n(segment.a.y),
            n(segment.b.x),
            n(segment.b.y),
            n(segment.width),
            escape_sexp(&segment.layer),
            stable_uuid(&format!("segment:{}", segment.id)),
        ));
    }
    for via in &layout.copper.vias {
        let net = layout_net_id(&net_ids, &via.net, "via", &via.id)?;
        out.push_str(&format!(
            "  (via (at {} {}) (size {}) (drill {}) (layers \"F.Cu\" \"B.Cu\") (net {net}) (uuid \"{}\"))\n",
            n(via.x),
            n(via.y),
            n(via.diameter),
            n(via.drill),
            stable_uuid(&format!("via:{}", via.id)),
        ));
    }

    out.push_str(")\n");
    Ok(out)
}

fn validate_layout(
    board: &Board,
    layout: &Layout,
    net_ids: &BTreeMap<String, usize>,
) -> via_core::Result<()> {
    if layout.board != board.name() {
        return Err(via_core::Error::Io(format!(
            "PCB layout board {} does not match design board {}",
            layout.board,
            board.name()
        )));
    }

    let board_refs = board
        .modules()
        .map(|module| module.refdes().to_owned())
        .collect::<BTreeSet<_>>();
    let mut layout_refs = BTreeSet::new();
    let mut covered_refs = BTreeSet::new();

    for placement in &layout.modules {
        if !board_refs.contains(&placement.refdes) {
            return Err(via_core::Error::Io(format!(
                "PCB layout references unknown module {}",
                placement.refdes
            )));
        }
        if !layout_refs.insert(placement.refdes.clone()) {
            return Err(via_core::Error::Io(format!(
                "PCB layout places module {} more than once",
                placement.refdes
            )));
        }
        covered_refs.insert(placement.refdes.clone());
    }

    let missing = board
        .modules()
        .filter(|module| module.footprint_name().is_some())
        .map(|module| module.refdes().to_owned())
        .filter(|refdes| !covered_refs.contains(refdes))
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(via_core::Error::Io(format!(
            "PCB layout is missing placement entries for modules: {}",
            missing.join(", ")
        )));
    }

    for segment in &layout.copper.segments {
        layout_net_id(net_ids, &segment.net, "segment", &segment.id)?;
    }
    for via in &layout.copper.vias {
        layout_net_id(net_ids, &via.net, "via", &via.id)?;
    }

    Ok(())
}

fn layout_net_id(
    net_ids: &BTreeMap<String, usize>,
    net: &str,
    item_kind: &str,
    item_id: &str,
) -> via_core::Result<usize> {
    net_ids.get(net).copied().ok_or_else(|| {
        via_core::Error::Io(format!(
            "PCB layout {item_kind} {item_id} references unknown net {net}"
        ))
    })
}

fn render_setup(board: &Board) -> String {
    let rules = board.rules();
    let track_width = rules.default_track_width_mm();
    let clearance = rules.clearance_mm();
    let via_size = rules.via_diameter_mm();
    let via_drill = rules.via_drill_mm();
    let via_min_size = (via_drill + 0.1).min(via_size).max(via_drill);
    let via_min_drill = (via_drill * 0.75).max(0.1);
    format!(
        concat!(
            "  (setup\n",
            "    (last_trace_width {})\n",
            "    (trace_clearance {})\n",
            "    (trace_min {})\n",
            "    (via_size {})\n",
            "    (via_drill {})\n",
            "    (via_min_size {})\n",
            "    (via_min_drill {})\n",
            "    (uvias_allowed no)\n",
            "    (pad_to_mask_clearance 0)\n",
            "    (pcbplotparams (layerselection 0x00010fc_ffffffff) (plot_on_all_layers_selection 0x0000000_00000000) (disableapertmacros false) (usegerberextensions false) (usegerberattributes true) (usegerberadvancedattributes true) (creategerberjobfile true) (dashed_line_dash_ratio 12.000000) (dashed_line_gap_ratio 3.000000) (svguseinch false) (svgprecision 4) (excludeedgelayer true) (plotframeref false) (viasonmask false) (mode 1) (useauxorigin false) (hpglpennumber 1) (hpglpenspeed 20) (hpglpendiameter 15.000000) (pdf_front_fp_property_popups true) (pdf_back_fp_property_popups true) (dxfpolygonmode true) (dxfimperialunits true) (dxfusepcbnewfont true) (psnegative false) (psa4output false) (plot_black_and_white false) (plotinvisibletext false) (sketchpadsonfab false) (subtractmaskfromsilk false) (outputformat 1) (mirror false) (drillshape 1) (scaleselection 1) (outputdirectory \"\"))\n",
            "  )\n"
        ),
        n(track_width),
        n(clearance),
        n(track_width),
        n(via_size),
        n(via_drill),
        n(via_min_size),
        n(via_min_drill),
    )
}

fn outline_points(layout: &Layout) -> Vec<Point> {
    if let Some(outline) = &layout.outline
        && outline.points.len() >= 3
    {
        return outline.points.clone();
    }
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    for module in &layout.modules {
        if module.status.as_deref() == Some("missing") {
            continue;
        }
        xs.push(module.x);
        ys.push(module.y);
    }
    for segment in &layout.copper.segments {
        xs.extend([segment.a.x, segment.b.x]);
        ys.extend([segment.a.y, segment.b.y]);
    }
    if xs.is_empty() {
        return vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 100.0, y: 0.0 },
            Point { x: 100.0, y: 80.0 },
            Point { x: 0.0, y: 80.0 },
        ];
    }
    let min_x = xs.iter().copied().fold(f64::INFINITY, f64::min) - 18.0;
    let max_x = xs.iter().copied().fold(f64::NEG_INFINITY, f64::max) + 18.0;
    let min_y = ys.iter().copied().fold(f64::INFINITY, f64::min) - 18.0;
    let max_y = ys.iter().copied().fold(f64::NEG_INFINITY, f64::max) + 18.0;
    vec![
        Point { x: min_x, y: min_y },
        Point { x: max_x, y: min_y },
        Point { x: max_x, y: max_y },
        Point { x: min_x, y: max_y },
    ]
}

fn render_footprint(
    module: &Part,
    placement: &LayoutModule,
    net_ids: &BTreeMap<String, usize>,
    pad_nets: &BTreeMap<(String, String), (String, String)>,
    generated: Option<&GeneratedFootprintIr>,
    footprint_library_name: &str,
    official_footprints: &BTreeMap<String, String>,
) -> via_core::Result<String> {
    let footprint_name = module.footprint_name().ok_or_else(|| {
        via_core::Error::Io(format!(
            "{} cannot be exported to PCB because it has no footprint",
            module.refdes()
        ))
    })?;
    if generated.is_none() {
        if let Some(kicad_mod) = official_footprints.get(footprint_name) {
            return kicad_mod_asset::render(AssetFootprintRender {
                footprint_name,
                kicad_mod,
                module,
                x: placement.x,
                y: placement.y,
                rotation: placement.rotation,
                net_ids,
                pad_nets,
                footprint_library_name,
            });
        }
        return Err(via_core::Error::Io(format!(
            "{} references footprint {} but PCB export has no generated footprint IR or loaded KiCad asset",
            module.refdes(),
            footprint_name
        )));
    }

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
    out.push_str(&format!(
        "    (at {} {} {})\n",
        n(placement.x),
        n(placement.y),
        n(placement.rotation)
    ));
    out.push_str(&format!(
        "    (property \"Reference\" \"{}\" (at 0 -3 0) (layer \"F.SilkS\") (effects (font (size 1 1) (thickness 0.15))))\n",
        escape_sexp(module.refdes()),
    ));
    out.push_str(&format!(
        "    (property \"Value\" \"{}\" (at 0 3 0) (layer \"F.Fab\") (hide yes) (effects (font (size 1 1) (thickness 0.15))))\n",
        escape_sexp(module.value()),
    ));
    out.push_str(&format!(
        "    (property \"Datasheet\" \"\" (at 0 0 0) (layer \"F.Fab\") (hide yes) (uuid \"{}\") (effects (font (size 1.27 1.27))))\n",
        stable_uuid(&format!("prop-datasheet:{}", module.refdes())),
    ));
    out.push_str(&format!(
        "    (property \"Description\" \"\" (at 0 0 0) (layer \"F.Fab\") (hide yes) (uuid \"{}\") (effects (font (size 1.27 1.27))))\n",
        stable_uuid(&format!("prop-description:{}", module.refdes())),
    ));
    if module.requires_verification() {
        out.push_str("    (property \"VIA_VERIFY\" \"true\" (at 0 0 0) (layer \"F.Fab\") (hide yes) (uuid \"");
        out.push_str(&stable_uuid(&format!("verify:{}", module.refdes())));
        out.push_str("\") (effects (font (size 1 1) (thickness 0.15))))\n");
    }
    out.push_str(&format!(
        "    (fp_text reference \"{}\" (at 0 -3 0) (layer \"F.SilkS\") (uuid \"{}\") (effects (font (size 1 1) (thickness 0.15))))\n",
        escape_sexp(module.refdes()),
        stable_uuid(&format!("text-ref:{}", module.refdes())),
    ));
    out.push_str(&format!(
        "    (fp_text value \"{}\" (at 0 3 0) (layer \"F.Fab\") hide (uuid \"{}\") (effects (font (size 1 1) (thickness 0.15))))\n",
        escape_sexp(module.value()),
        stable_uuid(&format!("text-val:{}", module.refdes())),
    ));

    if let Some(ir) = generated {
        for (line_idx, line) in ir.lines.iter().enumerate() {
            out.push_str(&format!(
                "    (fp_line (start {} {}) (end {} {}) (stroke (width {}) (type solid)) (layer \"{}\") (uuid \"{}\"))\n",
                n(line.x1),
                n(line.y1),
                n(line.x2),
                n(line.y2),
                n(line.width),
                escape_sexp(&line.layer),
                stable_uuid(&format!(
                    "line:{}:{}:{}:{}:{}:{}:{}:{}",
                    module.refdes(),
                    line_idx,
                    line.layer,
                    line.x1,
                    line.y1,
                    line.x2,
                    line.y2,
                    line.width
                )),
            ));
        }
        for pad in &ir.pads {
            out.push_str(&render_pad(PadRender {
                module,
                pad: &pad.number,
                x: pad.x,
                y: pad.y,
                w: pad.w,
                h: pad.h,
                drill: pad.drill,
                drill_w: pad.drill_w,
                drill_h: pad.drill_h,
                kind: &pad.kind,
                shape: &pad.shape,
                layers: &pad.layers,
                net_ids,
                pad_nets,
            }));
        }
    }
    out.push_str("  )\n");
    Ok(out)
}

struct PadRender<'a> {
    module: &'a Part,
    pad: &'a str,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    drill: Option<f64>,
    drill_w: Option<f64>,
    drill_h: Option<f64>,
    kind: &'a str,
    shape: &'a str,
    layers: &'a [String],
    net_ids: &'a BTreeMap<String, usize>,
    pad_nets: &'a BTreeMap<(String, String), (String, String)>,
}

fn render_pad(input: PadRender<'_>) -> String {
    let PadRender {
        module,
        pad,
        x,
        y,
        w,
        h,
        drill,
        drill_w,
        drill_h,
        kind,
        shape,
        layers,
        net_ids,
        pad_nets,
    } = input;
    let (net_name, pin_name) = pad_nets
        .get(&(module.refdes().to_owned(), pad.to_owned()))
        .cloned()
        .unwrap_or_else(|| (String::new(), String::new()));
    let net = net_name
        .is_empty()
        .then_some(0)
        .or_else(|| net_ids.get(&net_name).copied())
        .unwrap_or(0);
    let net_text = if net_name.is_empty() {
        String::new()
    } else {
        format!(" (net {net} \"{}\")", escape_sexp(&net_name))
    };
    let pin_text = if pin_name.is_empty() {
        String::new()
    } else {
        format!(" (pinfunction \"{}\")", escape_sexp(&pin_name))
    };
    let drill_text = drill_text(drill, drill_w, drill_h);
    let roundrect_text = if shape == "RoundRect" {
        " (roundrect_rratio 0.25)"
    } else {
        ""
    };
    format!(
        "    (pad \"{}\" {} {} (at {} {}) (size {} {}){}{} (layers {}){}{} (pintype \"passive\") (uuid \"{}\"))\n",
        escape_sexp(pad),
        pad_kind(kind),
        pad_shape(shape),
        n(x),
        n(y),
        n(w),
        n(h),
        drill_text,
        roundrect_text,
        layers
            .iter()
            .map(|layer| format!("\"{}\"", escape_sexp(layer)))
            .collect::<Vec<_>>()
            .join(" "),
        net_text,
        pin_text,
        stable_uuid(&format!("pad:{}:{}", module.refdes(), pad)),
    )
}

#[derive(Debug)]
struct GeneratedFootprintIr {
    pads: Vec<GeneratedPad>,
    lines: Vec<GeneratedLine>,
}

#[derive(Debug)]
struct GeneratedPad {
    number: String,
    kind: String,
    shape: String,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    drill: Option<f64>,
    drill_w: Option<f64>,
    drill_h: Option<f64>,
    layers: Vec<String>,
}

#[derive(Debug)]
struct GeneratedLine {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    layer: String,
    width: f64,
}

fn generated_footprint_irs(board: &Board) -> BTreeMap<String, GeneratedFootprintIr> {
    let mut map = BTreeMap::new();
    for footprint in board.footprints() {
        let Some(ir) = footprint.ir() else {
            continue;
        };
        let name = footprint.name().to_owned();
        map.insert(
            name,
            GeneratedFootprintIr {
                pads: ir
                    .pads()
                    .iter()
                    .map(|pad| GeneratedPad {
                        number: pad.number.clone(),
                        kind: format!("{:?}", pad.kind),
                        shape: format!("{:?}", pad.shape),
                        x: pad.at.x,
                        y: pad.at.y,
                        w: pad.size.x,
                        h: pad.size.y,
                        drill: pad.drill.map(|drill| drill.x.min(drill.y)),
                        drill_w: pad.drill.map(|drill| drill.x),
                        drill_h: pad.drill.map(|drill| drill.y),
                        layers: pad.layers.clone(),
                    })
                    .collect(),
                lines: ir
                    .lines()
                    .iter()
                    .map(|line| GeneratedLine {
                        x1: line.start.x,
                        y1: line.start.y,
                        x2: line.end.x,
                        y2: line.end.y,
                        layer: line.layer.clone(),
                        width: line.width,
                    })
                    .collect(),
            },
        );
    }
    map
}

fn net_ids(board: &Board) -> BTreeMap<String, usize> {
    let mut nets = BTreeMap::from([(String::new(), 0)]);
    for (idx, net) in board.nets().enumerate() {
        nets.insert(net.name().to_owned(), idx + 1);
    }
    nets
}

fn pad_net_map(board: &Board) -> BTreeMap<(String, String), (String, String)> {
    let mut map = BTreeMap::new();
    for net in board.nets() {
        for pin_ref in net.connections() {
            let Some(module) = board.module(&pin_ref.module) else {
                continue;
            };
            for pad in module.pads_for_pin(&pin_ref.pin) {
                map.insert(
                    (pin_ref.module.clone(), pad),
                    (net.name().to_owned(), pin_ref.pin.clone()),
                );
            }
        }
    }
    map
}

fn pad_kind(kind: &str) -> &'static str {
    match kind {
        "Smd" => "smd",
        "NpThruHole" => "np_thru_hole",
        _ => "thru_hole",
    }
}

fn drill_text(drill: Option<f64>, drill_w: Option<f64>, drill_h: Option<f64>) -> String {
    match (drill_w, drill_h, drill) {
        (Some(w), Some(h), _) if (w - h).abs() >= 0.001 => {
            format!(" (drill oval {} {})", n(w), n(h))
        }
        (_, _, Some(diameter)) => format!(" (drill {})", n(diameter)),
        (Some(w), Some(_), _) => format!(" (drill {})", n(w)),
        _ => String::new(),
    }
}

fn pad_shape(shape: &str) -> &'static str {
    match shape {
        "Rect" => "rect",
        "RoundRect" => "roundrect",
        "Trapezoid" => "trapezoid",
        "Oval" => "oval",
        _ => "circle",
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
    use via_core::Design;

    #[test]
    fn pcb_export_setup_uses_board_rules() {
        let mut design = Design::new("rules_demo");
        design
            .rules_mut()
            .set_default_track_width_mm(0.42)
            .set_clearance_mm(0.23)
            .set_via(0.9, 0.45);
        let board = design.build().unwrap();
        let layout = Layout {
            board: "rules_demo".to_owned(),
            modules: Vec::new(),
            outline: Some(LayoutOutline {
                points: vec![
                    Point { x: 0.0, y: 0.0 },
                    Point { x: 10.0, y: 0.0 },
                    Point { x: 10.0, y: 10.0 },
                    Point { x: 0.0, y: 10.0 },
                ],
            }),
            copper: LayoutCopper::default(),
            tracks: Vec::new(),
        };

        let text = render_kicad_pcb(&board, &layout, "FixtureLib", &BTreeMap::new()).unwrap();

        assert!(text.contains("(last_trace_width 0.42)"));
        assert!(text.contains("(trace_clearance 0.23)"));
        assert!(text.contains("(trace_min 0.42)"));
        assert!(text.contains("(via_size 0.9)"));
        assert!(text.contains("(via_drill 0.45)"));
    }

    #[test]
    fn debug_io_demo_pcb_export_contains_rules_geometry_and_copper() {
        let board = crate::test_fixtures::debug_io_board().unwrap();
        let modules = board
            .modules()
            .enumerate()
            .map(|(idx, module)| LayoutModule {
                refdes: module.refdes().to_owned(),
                x: 20.0 + (idx % 5) as f64 * 18.0,
                y: 20.0 + (idx / 5) as f64 * 18.0,
                rotation: 0.0,
                status: None,
            })
            .collect();
        let layout = Layout {
            board: "debug_io_demo".to_owned(),
            modules,
            outline: Some(LayoutOutline {
                points: vec![
                    Point { x: 0.0, y: 0.0 },
                    Point { x: 120.0, y: 0.0 },
                    Point { x: 120.0, y: 90.0 },
                    Point { x: 0.0, y: 90.0 },
                ],
            }),
            copper: LayoutCopper {
                segments: vec![LayoutSegment {
                    id: "seg-12v".to_owned(),
                    net: "5V_IN".to_owned(),
                    layer: "F.Cu".to_owned(),
                    width: 0.8,
                    a: Point { x: 10.0, y: 10.0 },
                    b: Point { x: 30.0, y: 10.0 },
                }],
                vias: vec![LayoutVia {
                    id: "via-12v".to_owned(),
                    net: "5V_IN".to_owned(),
                    x: 30.0,
                    y: 10.0,
                    drill: 0.4,
                    diameter: 0.8,
                }],
            },
            tracks: Vec::new(),
        };

        let text = render_kicad_pcb(&board, &layout, "FixtureLib", &BTreeMap::new()).unwrap();

        assert!(text.contains("(last_trace_width 0.3)"));
        assert!(text.contains("(trace_clearance 0.2)"));
        assert!(text.contains("(via_size 0.8)"));
        assert!(text.contains("(via_drill 0.4)"));
        assert!(text.contains("(net "));
        assert!(text.contains("\"5V_IN\""));
        assert!(text.contains("(footprint \"FixtureLib:SOT-223\""));
        assert!(text.contains("(footprint \"FixtureLib:TSSOP-20\""));
        assert!(text.contains("(footprint \"FixtureLib:LED_0805\""));
        assert!(text.contains("(segment (start 10 10) (end 30 10) (width 0.8) (layer \"F.Cu\")"));
        assert!(text.contains("(via (at 30 10) (size 0.8) (drill 0.4)"));
        assert!(text.contains("(layer \"Edge.Cuts\")"));
        assert_unique_uuids(&text);
    }

    #[test]
    fn pcb_export_rejects_missing_footprint_geometry() {
        let mut design = Design::new("missing_geometry");
        design
            .add(
                via_core::part("J1", "external")
                    .footprint("External_Footprint")
                    .pin(via_core::pin("1").passive().pad("1")),
            )
            .unwrap();
        let board = design.build().unwrap();
        let layout = Layout {
            board: "missing_geometry".to_owned(),
            modules: vec![LayoutModule {
                refdes: "J1".to_owned(),
                x: 0.0,
                y: 0.0,
                rotation: 0.0,
                status: None,
            }],
            outline: None,
            copper: LayoutCopper::default(),
            tracks: Vec::new(),
        };

        let err = render_kicad_pcb(&board, &layout, "FixtureLib", &BTreeMap::new()).unwrap_err();

        assert!(format!("{err}").contains("no generated footprint IR or loaded KiCad asset"));
    }

    #[test]
    fn pcb_export_rejects_unknown_layout_module() {
        let board = Design::new("unknown_module").build().unwrap();
        let layout = Layout {
            board: "unknown_module".to_owned(),
            modules: vec![LayoutModule {
                refdes: "J404".to_owned(),
                x: 0.0,
                y: 0.0,
                rotation: 0.0,
                status: None,
            }],
            outline: None,
            copper: LayoutCopper::default(),
            tracks: Vec::new(),
        };

        let err = render_kicad_pcb(&board, &layout, "FixtureLib", &BTreeMap::new()).unwrap_err();

        assert!(format!("{err}").contains("unknown module J404"));
    }

    #[test]
    fn pcb_export_rejects_layout_board_mismatch() {
        let board = Design::new("current_board").build().unwrap();
        let layout = Layout {
            board: "stale_board".to_owned(),
            modules: Vec::new(),
            outline: None,
            copper: LayoutCopper::default(),
            tracks: Vec::new(),
        };

        let err = render_kicad_pcb(&board, &layout, "FixtureLib", &BTreeMap::new()).unwrap_err();

        assert!(format!("{err}").contains("layout board stale_board"));
    }

    #[test]
    fn pcb_export_rejects_unplaced_board_module() {
        let mut design = Design::new("unplaced");
        design
            .add(
                via_core::part("J1", "external")
                    .footprint("External_Footprint")
                    .pin(via_core::pin("1").passive().pad("1")),
            )
            .unwrap();
        let board = design.build().unwrap();
        let layout = Layout {
            board: "unplaced".to_owned(),
            modules: Vec::new(),
            outline: None,
            copper: LayoutCopper::default(),
            tracks: Vec::new(),
        };

        let err = render_kicad_pcb(&board, &layout, "FixtureLib", &BTreeMap::new()).unwrap_err();

        assert!(format!("{err}").contains("missing placement entries for modules: J1"));
    }

    #[test]
    fn pcb_export_allows_explicitly_missing_board_module() {
        let mut design = Design::new("explicit_missing");
        design
            .add(
                via_core::part("J1", "external")
                    .footprint("External_Footprint")
                    .pin(via_core::pin("1").passive().pad("1")),
            )
            .unwrap();
        let board = design.build().unwrap();
        let layout = Layout {
            board: "explicit_missing".to_owned(),
            modules: vec![LayoutModule {
                refdes: "J1".to_owned(),
                x: 0.0,
                y: 0.0,
                rotation: 0.0,
                status: Some("missing".to_owned()),
            }],
            outline: None,
            copper: LayoutCopper::default(),
            tracks: Vec::new(),
        };

        render_kicad_pcb(&board, &layout, "FixtureLib", &BTreeMap::new()).unwrap();
    }

    #[test]
    fn pcb_export_rejects_duplicate_layout_module() {
        let mut design = Design::new("duplicate");
        design
            .add(
                via_core::part("J1", "external")
                    .footprint("External_Footprint")
                    .pin(via_core::pin("1").passive().pad("1")),
            )
            .unwrap();
        let board = design.build().unwrap();
        let layout = Layout {
            board: "duplicate".to_owned(),
            modules: vec![
                LayoutModule {
                    refdes: "J1".to_owned(),
                    x: 0.0,
                    y: 0.0,
                    rotation: 0.0,
                    status: Some("missing".to_owned()),
                },
                LayoutModule {
                    refdes: "J1".to_owned(),
                    x: 10.0,
                    y: 0.0,
                    rotation: 0.0,
                    status: Some("missing".to_owned()),
                },
            ],
            outline: None,
            copper: LayoutCopper::default(),
            tracks: Vec::new(),
        };

        let err = render_kicad_pcb(&board, &layout, "FixtureLib", &BTreeMap::new()).unwrap_err();

        assert!(format!("{err}").contains("places module J1 more than once"));
    }

    #[test]
    fn pcb_export_rejects_unknown_copper_nets() {
        let board = Design::new("bad_copper").build().unwrap();
        let layout = Layout {
            board: "bad_copper".to_owned(),
            modules: Vec::new(),
            outline: None,
            copper: LayoutCopper {
                segments: vec![LayoutSegment {
                    id: "seg-bad".to_owned(),
                    net: "TYPO_NET".to_owned(),
                    layer: "F.Cu".to_owned(),
                    width: 0.25,
                    a: Point { x: 0.0, y: 0.0 },
                    b: Point { x: 1.0, y: 0.0 },
                }],
                vias: Vec::new(),
            },
            tracks: Vec::new(),
        };

        let err = render_kicad_pcb(&board, &layout, "FixtureLib", &BTreeMap::new()).unwrap_err();

        assert!(format!("{err}").contains("segment seg-bad references unknown net TYPO_NET"));
    }

    #[test]
    fn pcb_export_rejects_unknown_via_nets() {
        let board = Design::new("bad_via").build().unwrap();
        let layout = Layout {
            board: "bad_via".to_owned(),
            modules: Vec::new(),
            outline: None,
            copper: LayoutCopper {
                segments: Vec::new(),
                vias: vec![LayoutVia {
                    id: "via-bad".to_owned(),
                    net: "TYPO_NET".to_owned(),
                    x: 0.0,
                    y: 0.0,
                    drill: 0.4,
                    diameter: 0.8,
                }],
            },
            tracks: Vec::new(),
        };

        let err = render_kicad_pcb(&board, &layout, "FixtureLib", &BTreeMap::new()).unwrap_err();

        assert!(format!("{err}").contains("via via-bad references unknown net TYPO_NET"));
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
