use std::collections::BTreeMap;

use serde::Serialize;
use via_core::{Board, Diagnostic, ObjectRef};

const SNAPSHOT_VERSION: u8 = 3;

#[derive(Debug, Serialize)]
struct CheckSummary {
    board: String,
    ok: bool,
    footprints_loaded: usize,
    diagnostics: Vec<DiagnosticJson>,
}

#[derive(Debug, Serialize)]
struct BoardSnapshot {
    version: u8,
    board: String,
    source_hash: String,
    source_signatures: SnapshotSignatures,
    footprints_loaded: usize,
    footprints: Vec<FootprintJson>,
    rules: RulesJson,
    modules: Vec<ModuleJson>,
    nets: Vec<NetJson>,
    diagnostics: Vec<DiagnosticJson>,
    production_diagnostics: Vec<DiagnosticJson>,
}

#[derive(Debug, Serialize)]
struct SnapshotSignatures {
    rules: String,
    modules: BTreeMap<String, String>,
    nets: BTreeMap<String, String>,
    footprints: BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
struct RulesJson {
    #[serde(rename = "gridMm")]
    grid_mm: f64,
    #[serde(rename = "defaultTrackWidthMm")]
    default_track_width_mm: f64,
    #[serde(rename = "netClassTrackWidthMm")]
    net_class_track_width_mm: BTreeMap<String, f64>,
    #[serde(rename = "clearanceMm")]
    clearance_mm: f64,
    #[serde(rename = "viaDrillMm")]
    via_drill_mm: f64,
    #[serde(rename = "viaDiameterMm")]
    via_diameter_mm: f64,
}

#[derive(Debug, Serialize)]
struct FootprintJson {
    name: String,
    pads: Vec<FootprintPadJson>,
    lines: Vec<FootprintLineJson>,
}

#[derive(Debug, Serialize)]
struct FootprintPadJson {
    number: String,
    kind: String,
    shape: String,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    drill: Option<f64>,
    #[serde(rename = "drillShape")]
    drill_shape: String,
    #[serde(rename = "drillW")]
    drill_w: Option<f64>,
    #[serde(rename = "drillH")]
    drill_h: Option<f64>,
    layers: Vec<String>,
}

#[derive(Debug, Serialize)]
struct FootprintLineJson {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    layer: String,
    width: f64,
}

#[derive(Debug, Serialize)]
struct ModuleJson {
    refdes: String,
    value: String,
    footprint: Option<String>,
    requires_verification: bool,
    mpn: Option<String>,
    supplier_parts: Vec<SupplierPartJson>,
    production_notes: Vec<String>,
    pins: Vec<PinJson>,
}

#[derive(Debug, Serialize)]
struct SupplierPartJson {
    supplier: String,
    part_number: String,
}

#[derive(Debug, Serialize)]
struct PinJson {
    name: String,
    class: Option<String>,
    pads: Vec<String>,
}

#[derive(Debug, Serialize)]
struct NetJson {
    name: String,
    class: Option<String>,
    connections: Vec<ConnectionJson>,
}

#[derive(Debug, Serialize)]
struct ConnectionJson {
    module: String,
    pin: String,
}

#[derive(Debug, Serialize)]
struct DiagnosticJson {
    severity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<String>,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    object: Option<ObjectRefJson>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    related: Vec<ObjectRefJson>,
}

#[derive(Debug, Serialize)]
struct ObjectRefJson {
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    refdes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pad: Option<String>,
}

pub fn check_summary(
    board: &str,
    ok: bool,
    footprints_loaded: usize,
    diagnostics: &[Diagnostic],
) -> String {
    to_pretty_json(&CheckSummary {
        board: board.to_owned(),
        ok,
        footprints_loaded,
        diagnostics: diagnostics_json(diagnostics),
    })
}

pub fn escape_json(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out
}

pub fn board_snapshot(
    board: &Board,
    footprints_loaded: usize,
    diagnostics: &[Diagnostic],
    production_diagnostics: &[Diagnostic],
) -> String {
    let footprints = footprint_geometries_json();
    let signatures = source_signatures(board, &footprints);
    let source_hash = source_hash(&signatures);
    to_pretty_json(&BoardSnapshot {
        version: SNAPSHOT_VERSION,
        board: board.name().to_owned(),
        source_hash,
        source_signatures: signatures,
        footprints_loaded,
        footprints,
        rules: rules_json(board),
        modules: modules_json(board),
        nets: nets_json(board),
        diagnostics: diagnostics_json(diagnostics),
        production_diagnostics: diagnostics_json(production_diagnostics),
    })
}

fn source_signatures(board: &Board, footprints: &[FootprintJson]) -> SnapshotSignatures {
    SnapshotSignatures {
        rules: stable_hash(&rules_signature(board)),
        modules: board
            .modules()
            .map(|module| {
                let pins = module
                    .pins_iter()
                    .map(|pin| {
                        format!(
                            "{}:{}:{}",
                            pin,
                            module
                                .class_for_pin(pin)
                                .map(ToString::to_string)
                                .unwrap_or_default(),
                            module
                                .pads_for_pin(pin)
                                .into_iter()
                                .collect::<Vec<_>>()
                                .join("+")
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("|");
                let text = format!(
                    "{};{};{};{};{}",
                    module.refdes(),
                    module.value(),
                    module.footprint_name().unwrap_or(""),
                    module.requires_verification(),
                    pins
                );
                (module.refdes().to_owned(), stable_hash(&text))
            })
            .collect(),
        nets: board
            .nets()
            .map(|net| {
                let mut connections = net
                    .connections()
                    .iter()
                    .map(|pin| format!("{}.{}", pin.module, pin.pin))
                    .collect::<Vec<_>>();
                connections.sort();
                let text = format!(
                    "{};{};{}",
                    net.name(),
                    net.electrical_class()
                        .map(ToString::to_string)
                        .unwrap_or_default(),
                    connections.join("|")
                );
                (net.name().to_owned(), stable_hash(&text))
            })
            .collect(),
        footprints: footprints
            .iter()
            .map(|footprint| {
                (
                    footprint.name.clone(),
                    stable_hash(&footprint_signature(footprint)),
                )
            })
            .collect(),
    }
}

fn source_hash(signatures: &SnapshotSignatures) -> String {
    let mut parts = vec![format!("rules={}", signatures.rules)];
    parts.extend(
        signatures
            .modules
            .iter()
            .map(|(name, hash)| format!("module:{name}={hash}")),
    );
    parts.extend(
        signatures
            .nets
            .iter()
            .map(|(name, hash)| format!("net:{name}={hash}")),
    );
    parts.extend(
        signatures
            .footprints
            .iter()
            .map(|(name, hash)| format!("footprint:{name}={hash}")),
    );
    stable_hash(&parts.join("\n"))
}

fn rules_signature(board: &Board) -> String {
    let rules = board.rules();
    let classes = rules
        .net_class_track_widths_mm()
        .map(|(class, width)| format!("{class}:{width}"))
        .collect::<Vec<_>>()
        .join("|");
    format!(
        "grid={};track={};clearance={};via={}/{};classes={}",
        rules.grid_mm(),
        rules.default_track_width_mm(),
        rules.clearance_mm(),
        rules.via_diameter_mm(),
        rules.via_drill_mm(),
        classes
    )
}

fn footprint_signature(footprint: &FootprintJson) -> String {
    let pads = footprint
        .pads
        .iter()
        .map(|pad| {
            format!(
                "{}:{}:{}:{},{},{},{}:{:?}:{:?}:{}",
                pad.number,
                pad.kind,
                pad.shape,
                pad.x,
                pad.y,
                pad.w,
                pad.h,
                pad.drill_w,
                pad.drill_h,
                pad.layers.join("+")
            )
        })
        .collect::<Vec<_>>()
        .join("|");
    let lines = footprint
        .lines
        .iter()
        .map(|line| {
            format!(
                "{},{},{},{}:{}:{}",
                line.x1, line.y1, line.x2, line.y2, line.layer, line.width
            )
        })
        .collect::<Vec<_>>()
        .join("|");
    format!("{};pads={pads};lines={lines}", footprint.name)
}

fn stable_hash(text: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn to_pretty_json<T: Serialize>(value: &T) -> String {
    let mut text =
        serde_json::to_string_pretty(value).expect("serializing VIA JSON should not fail");
    text.push('\n');
    text
}

fn rules_json(board: &Board) -> RulesJson {
    let rules = board.rules();
    RulesJson {
        grid_mm: rules.grid_mm(),
        default_track_width_mm: rules.default_track_width_mm(),
        net_class_track_width_mm: rules
            .net_class_track_widths_mm()
            .map(|(class, width)| (class.clone(), *width))
            .collect(),
        clearance_mm: rules.clearance_mm(),
        via_drill_mm: rules.via_drill_mm(),
        via_diameter_mm: rules.via_diameter_mm(),
    }
}

fn footprint_geometries_json() -> Vec<FootprintJson> {
    via_parts_harmonic::generated_footprints()
        .into_iter()
        .map(|footprint| {
            let ir = footprint.into_ir();
            FootprintJson {
                name: ir.name().to_owned(),
                pads: ir
                    .pads()
                    .iter()
                    .map(|pad| FootprintPadJson {
                        number: pad.number.clone(),
                        kind: format!("{:?}", pad.kind),
                        shape: format!("{:?}", pad.shape),
                        x: pad.at.x,
                        y: pad.at.y,
                        w: pad.size.x,
                        h: pad.size.y,
                        drill: pad.drill.map(|drill| drill.x.min(drill.y)),
                        drill_shape: pad
                            .drill
                            .map(|drill| if drill.is_round() { "Round" } else { "Oval" })
                            .unwrap_or("None")
                            .to_owned(),
                        drill_w: pad.drill.map(|drill| drill.x),
                        drill_h: pad.drill.map(|drill| drill.y),
                        layers: pad.layers.clone(),
                    })
                    .collect(),
                lines: ir
                    .lines()
                    .iter()
                    .map(|line| FootprintLineJson {
                        x1: line.start.x,
                        y1: line.start.y,
                        x2: line.end.x,
                        y2: line.end.y,
                        layer: line.layer.clone(),
                        width: line.width,
                    })
                    .collect(),
            }
        })
        .collect()
}

fn modules_json(board: &Board) -> Vec<ModuleJson> {
    board
        .modules()
        .map(|module| ModuleJson {
            refdes: module.refdes().to_owned(),
            value: module.value().to_owned(),
            footprint: module.footprint_name().map(str::to_owned),
            requires_verification: module.requires_verification(),
            mpn: module.manufacturer_part_number().map(str::to_owned),
            supplier_parts: module
                .supplier_parts()
                .map(|(supplier, part_number)| SupplierPartJson {
                    supplier: supplier.clone(),
                    part_number: part_number.clone(),
                })
                .collect(),
            production_notes: module.production_notes().to_vec(),
            pins: module
                .pins_iter()
                .map(|pin| PinJson {
                    name: pin.clone(),
                    class: module.class_for_pin(pin).map(ToString::to_string),
                    pads: module.pads_for_pin(pin).into_iter().collect(),
                })
                .collect(),
        })
        .collect()
}

fn nets_json(board: &Board) -> Vec<NetJson> {
    board
        .nets()
        .map(|net| NetJson {
            name: net.name().to_owned(),
            class: net.electrical_class().map(ToString::to_string),
            connections: net
                .connections()
                .iter()
                .map(|pin| ConnectionJson {
                    module: pin.module.clone(),
                    pin: pin.pin.clone(),
                })
                .collect(),
        })
        .collect()
}

fn diagnostics_json(diagnostics: &[Diagnostic]) -> Vec<DiagnosticJson> {
    diagnostics
        .iter()
        .map(|diagnostic| DiagnosticJson {
            severity: diagnostic.severity().to_string(),
            code: diagnostic.code.clone(),
            message: diagnostic.message().to_owned(),
            object: diagnostic.object().map(object_ref_json),
            related: diagnostic.related().iter().map(object_ref_json).collect(),
        })
        .collect()
}

fn object_ref_json(object: &ObjectRef) -> ObjectRefJson {
    match object {
        ObjectRef::Board { name } => ObjectRefJson {
            kind: object.kind().to_owned(),
            name: Some(name.clone()),
            refdes: None,
            pin: None,
            pad: None,
        },
        ObjectRef::Module { refdes } => ObjectRefJson {
            kind: object.kind().to_owned(),
            name: None,
            refdes: Some(refdes.clone()),
            pin: None,
            pad: None,
        },
        ObjectRef::Pin { refdes, pin } => ObjectRefJson {
            kind: object.kind().to_owned(),
            name: None,
            refdes: Some(refdes.clone()),
            pin: Some(pin.clone()),
            pad: None,
        },
        ObjectRef::Net { name } => ObjectRefJson {
            kind: object.kind().to_owned(),
            name: Some(name.clone()),
            refdes: None,
            pin: None,
            pad: None,
        },
        ObjectRef::Footprint { name } => ObjectRefJson {
            kind: object.kind().to_owned(),
            name: Some(name.clone()),
            refdes: None,
            pin: None,
            pad: None,
        },
        ObjectRef::Pad { refdes, pad } => ObjectRefJson {
            kind: object.kind().to_owned(),
            name: None,
            refdes: Some(refdes.clone()),
            pin: None,
            pad: Some(pad.clone()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use via_core::{BoardSpec, ObjectRef, Part};

    #[test]
    fn escapes_json_strings() {
        assert_eq!(escape_json("a\"b\\c\n"), "a\\\"b\\\\c\\n");
    }

    #[test]
    fn renders_check_summary() {
        let json = check_summary(
            "demo",
            false,
            2,
            &[
                Diagnostic::coded("net.physical_pad_short", "pin U1.1 is on NET_A and NET_B")
                    .at(ObjectRef::pin("U1", "1"))
                    .relates_to(ObjectRef::net("NET_A")),
            ],
        );

        assert!(json.contains("\"board\": \"demo\""));
        assert!(json.contains("\"ok\": false"));
        assert!(json.contains("\"footprints_loaded\": 2"));
        assert!(json.contains("\"severity\": \"error\""));
        assert!(json.contains("\"code\": \"net.physical_pad_short\""));
        assert!(json.contains("\"kind\": \"pin\""));
        assert!(json.contains("\"refdes\": \"U1\""));
        assert!(json.contains("\"pin\": \"1\""));
        assert!(json.contains("\"related\""));
        assert!(json.contains("\"name\": \"NET_A\""));
        assert!(json.contains("pin U1.1 is on NET_A and NET_B"));
    }

    #[test]
    fn renders_board_snapshot() {
        let mut spec = BoardSpec::new("demo");
        let module = spec
            .add(
                Part::new("R1", "1k")
                    .footprint("R_0603")
                    .pins(["1", "2"])
                    .lcsc("C21190"),
            )
            .unwrap();
        spec.net("N")
            .connect_all([module.pin("1"), module.pin("2")]);
        let board = spec.build().unwrap();

        let json = board_snapshot(&board, 0, &[], &[]);

        assert!(json.contains("\"version\": 3"));
        assert!(json.contains("\"modules\""));
        assert!(json.contains("\"rules\""));
        assert!(json.contains("\"defaultTrackWidthMm\": 0.3"));
        assert!(json.contains("\"logic:3V3\": 0.25"));
        assert!(json.contains("\"refdes\": \"R1\""));
        assert!(json.contains("\"supplier\": \"LCSC\""));
        assert!(json.contains("\"nets\""));
    }

    #[test]
    fn renders_polar_adjuster_snapshot_contract() {
        let board = via_examples::polar_adjuster::polar_adjuster_v0_board().unwrap();
        let json = board_snapshot(&board, board.footprints().count(), &[], &[]);

        assert!(json.contains("\"version\": 3"));
        assert!(json.contains("\"board\": \"polar_adjuster_v0\""));
        assert!(json.contains("\"footprints_loaded\": 14"));
        assert!(json.contains("\"name\": \"ESP32-S3-N16R8_DevBoard_2x22_P2.54_Row25.40\""));
        assert!(
            json.contains("\"name\": \"SilentStepStick_TMC2209_v20_CarrierSocket_2x8_Row12p70\"")
        );
        assert!(json.contains("\"name\": \"DC005_5p5x2p1_RightAngle_THT_Drawing_2_3_4_VERIFY\""));
        assert!(json.contains("\"name\": \"12V_IN\""));
        assert!(json.contains("\"class\": \"power:12V\""));
        assert!(json.contains("\"power:12V\": 0.8"));
        assert!(json.contains("\"motor-phase\": 0.5"));
    }
}
