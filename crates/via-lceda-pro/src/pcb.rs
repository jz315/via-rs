use std::collections::BTreeMap;

use via_core::{Board, model::Part};

use crate::epru::EpruWriter;
use crate::ids::{
    board_uuid, device_uuid, footprint_pad_id, footprint_uuid, json_escape, pcb_component_id,
    pcb_uuid, stable_uuid,
};
use crate::model::pcb_component_placement;
use crate::units::opt_i32;

pub(crate) fn render_pcb_document(writer: &mut EpruWriter, board: &Board) {
    writer.dochead("PCB", &pcb_uuid(board.name()));
    writer.record_with_id(
        "CANVAS",
        "CANVAS",
        concat!(
            "{\"originX\":0,\"originY\":0,\"unit\":\"mil\",",
            "\"gridXSize\":2.5,\"gridYSize\":2.5,",
            "\"snapXSize\":2.5,\"snapYSize\":2.5,",
            "\"altSnapXSize\":1,\"altSnapYSize\":1,",
            "\"gridType\":\"OUTLETS\",\"multiGridType\":\"NONE\",",
            "\"multiGridRatio\":5,\"highlightValue\":0.5}"
        ),
    );
    for (id, layer_type, layer_name, color) in [
        (1, "TOP", "Top Layer", "#ff0000"),
        (2, "BOTTOM", "Bottom Layer", "#0000ff"),
        (3, "TOP_SILK", "Top Silkscreen Layer", "#ffcc00"),
        (4, "BOT_SILK", "Bottom Silkscreen Layer", "#66cc33"),
        (5, "TOP_SOLDER_MASK", "Top Solder Mask Layer", "#800080"),
        (6, "BOT_SOLDER_MASK", "Bottom Solder Mask Layer", "#aa00ff"),
        (7, "TOP_PASTE_MASK", "Top Paste Mask Layer", "#808080"),
        (8, "BOT_PASTE_MASK", "Bottom Paste Mask Layer", "#800000"),
        (11, "OUTLINE", "Board Outline Layer", "#ff00ff"),
        (12, "MULTI", "Multi-Layer", "#c0c0c0"),
        (13, "DOCUMENT", "Document Layer", "#ffffff"),
        (14, "MECHANICAL", "Mechanical Layer", "#f022f0"),
        (47, "HOLE", "Hole Layer", "#222222"),
        (57, "OTHER", "Ratline Layer", "#6464ff"),
    ] {
        writer.record_with_id(
            "LAYER",
            &format!("[\"LAYER\",{id}]"),
            &format!(
                concat!(
                    "{{\"layerType\":\"{}\",\"layerName\":\"{}\",",
                    "\"use\":true,\"show\":true,\"locked\":false,",
                    "\"activeColor\":\"{}\",\"activateTransparency\":1,",
                    "\"inactiveColor\":\"#7f7f7f\",\"inactiveTransparency\":0.5}}"
                ),
                layer_type, layer_name, color,
            ),
        );
    }
    writer.record_with_id("ACTIVE_LAYER", "ACTIVE_LAYER", "{\"layerId\":1}");
    writer.record_with_id(
        "NET",
        "[\"NET\",\"\"]",
        "{\"netType\":null,\"specialColor\":null,\"retLine\":true,\"differentialName\":null,\"isPositiveNet\":false,\"equalLengthGroupName\":null}",
    );
    for net in board.nets() {
        writer.record_with_id(
            "NET",
            &format!("[\"NET\",\"{}\"]", json_escape(net.name())),
            "{\"netType\":null,\"specialColor\":null,\"retLine\":true,\"differentialName\":null,\"isPositiveNet\":false,\"equalLengthGroupName\":null}",
        );
    }
    writer.record_with_id(
        "ELE_PLACEHOLDER",
        "placeholder_pcb_components",
        &format!(
            "{{\"dataType\":\"COMPONENT\",\"max\":{}}}",
            board.modules().count()
        ),
    );
    render_pcb_components(writer, board);
    writer.record_with_id(
        "META",
        "META",
        &format!(
            "{{\"title\":\"PCB\",\"parent\":\"\",\"board\":\"{}\",\"zIndex\":null}}",
            board_uuid(board.name()),
        ),
    );
    writer.record_with_id(
        "PREFERENCE",
        "PREFERENCE",
        concat!(
            "{\"startTrackWidthFollowLast\":false,\"lastTrackWidth\":10,",
            "\"startViaSizeFollowLast\":false,\"lastViaInnerDiameter\":12,",
            "\"lastViaDiameter\":24,\"snap\":true,\"routingMode\":\"OBSTRUCT\",",
            "\"routingCorner\":\"L90\",\"removeLoop\":false,",
            "\"rotatingObject\":false,\"trackFollow\":false,",
            "\"stretchTrackMinCorner\":1,\"preferenceConfig\":null,",
            "\"realTimeUpdateUnusedLayers\":false,\"unusedPadRange\":\"VIA\",",
            "\"pushVia\":\"OPTIMIZA_OPEN\",",
            "\"pathOptimization4BePushed\":\"SINGLE\",",
            "\"currentPathOptimization4BePushed\":\"OPTIMIZA_WEAK\",",
            "\"removeCircuitsContainingVias\":true,\"removeAntenna\":true}"
        ),
    );
    writer.record_with_id(
        "PANELIZE",
        "PANELIZE",
        concat!(
            "{\"on\":false,\"row\":1,\"column\":1,",
            "\"rowSpacing\":0,\"columnSpacing\":0,\"onlyOutline\":true,",
            "\"horizontalStamp\":{\"on\":false},",
            "\"verticalStamp\":{\"on\":false},",
            "\"horizontalSize\":{\"on\":false},",
            "\"verticalSize\":{\"on\":false}}"
        ),
    );
}

fn render_pcb_components(writer: &mut EpruWriter, board: &Board) {
    for (index, module) in board.modules().enumerate() {
        let component_id = pcb_component_id(module.refdes());
        let placement = pcb_component_placement(index);
        writer.record_with_id(
            "COMPONENT",
            &component_id,
            &format!(
                concat!(
                    "{{\"partitionId\":\"\",\"groupId\":0,\"layerId\":1,",
                    "\"x\":{},\"y\":{},\"angle\":0,",
                    "\"attrs\":{{\"Unique ID\":\"{}\",\"Reuse Block\":\"\",",
                    "\"Group ID\":\"\",\"Channel ID\":\"{}\"}},",
                    "\"locked\":false,\"zIndex\":{}}}"
                ),
                placement.x,
                placement.y,
                json_escape(&stable_uuid(&format!("pcb-component:{}", module.refdes()))),
                json_escape(&format!("$1{}", component_id)),
                index + 1,
            ),
        );

        let pad_nets = module_pad_nets(board, module);
        for (pad, net_name) in pad_nets {
            writer.record_with_id(
                "PAD_NET",
                &format!(
                    "[\"PAD_NET\",\"{}\",\"{}\",\"{}\"]",
                    json_escape(&component_id),
                    json_escape(&pad),
                    json_escape(&footprint_pad_id(&pad))
                ),
                &format!(
                    "{{\"partitionId\":\"\",\"padNet\":\"{}\",\"padLen\":0}}",
                    json_escape(&net_name),
                ),
            );
        }

        if let Some(footprint_name) = module.footprint_name() {
            pcb_attr(
                writer,
                PcbAttribute {
                    id: &format!("{component_id}_footprint"),
                    parent_id: &component_id,
                    layer_id: 3,
                    key: "Footprint",
                    value: &footprint_uuid(footprint_name),
                    x: None,
                    y: None,
                    visible: false,
                },
            );
        }
        pcb_attr(
            writer,
            PcbAttribute {
                id: &format!("{component_id}_designator"),
                parent_id: &component_id,
                layer_id: 3,
                key: "Designator",
                value: module.refdes(),
                x: Some(placement.x - 55),
                y: Some(placement.y - 80),
                visible: true,
            },
        );
        pcb_attr(
            writer,
            PcbAttribute {
                id: &format!("{component_id}_device"),
                parent_id: &component_id,
                layer_id: 3,
                key: "Device",
                value: &device_uuid(module),
                x: None,
                y: None,
                visible: false,
            },
        );
    }
}

fn module_pad_nets(board: &Board, module: &Part) -> BTreeMap<String, String> {
    let mut pad_nets = BTreeMap::new();
    for net in board.nets() {
        for pin_ref in net.connections() {
            if pin_ref.module != module.refdes() {
                continue;
            }
            for pad in module.pads_for_pin(&pin_ref.pin) {
                pad_nets.insert(pad, net.name().to_owned());
            }
        }
    }
    pad_nets
}

struct PcbAttribute<'a> {
    id: &'a str,
    parent_id: &'a str,
    layer_id: usize,
    key: &'a str,
    value: &'a str,
    x: Option<i32>,
    y: Option<i32>,
    visible: bool,
}

fn pcb_attr(writer: &mut EpruWriter, attribute: PcbAttribute<'_>) {
    let PcbAttribute {
        id,
        parent_id,
        layer_id,
        key,
        value,
        x,
        y,
        visible,
    } = attribute;
    writer.record_with_id(
        "ATTR",
        id,
        &format!(
            concat!(
                "{{\"partitionId\":\"\",\"groupId\":0,\"parentId\":\"{}\",",
                "\"layerId\":{},\"x\":{},\"y\":{},\"key\":\"{}\",",
                "\"value\":\"{}\",\"keyVisible\":false,\"valueVisible\":{},",
                "\"fontFamily\":\"default\",\"fontSize\":45,\"strokeWidth\":6,",
                "\"bold\":false,\"italic\":false,\"origin\":\"LEFT_BOTTOM\",",
                "\"angle\":0,\"reverse\":false,\"expansion\":0,\"mirror\":false,",
                "\"locked\":false,\"zIndex\":null}}"
            ),
            json_escape(parent_id),
            layer_id,
            opt_i32(x),
            opt_i32(y),
            json_escape(key),
            json_escape(value),
            visible,
        ),
    );
}
