use crate::epru::EpruWriter;

pub(crate) fn render_standard_layers(writer: &mut EpruWriter) {
    for (id, layer_type, layer_name, active, inactive, alpha) in [
        (1, "TOP", "Top Layer", "#00c040", "#7f0000", 1.0),
        (2, "BOTTOM", "Bottom Layer", "#808000", "#00007f", 1.0),
        (
            3,
            "TOP_SILK",
            "Top Silkscreen Layer",
            "#00ffff",
            "#7f6600",
            1.0,
        ),
        (
            4,
            "BOT_SILK",
            "Bottom Silkscreen Layer",
            "#ff00ff",
            "#336619",
            1.0,
        ),
        (
            7,
            "TOP_PASTE_MASK",
            "Top Paste Mask Layer",
            "#008000",
            "#404040",
            1.0,
        ),
        (
            8,
            "BOT_PASTE_MASK",
            "Bottom Paste Mask Layer",
            "#000080",
            "#400000",
            1.0,
        ),
        (
            5,
            "TOP_SOLDER_MASK",
            "Top Solder Mask Layer",
            "#0000ff",
            "#400040",
            0.7,
        ),
        (
            6,
            "BOT_SOLDER_MASK",
            "Bottom Solder Mask Layer",
            "#0000ff",
            "#55007f",
            0.7,
        ),
        (13, "DOCUMENT", "Document Layer", "#ffffff", "#7f7f7f", 1.0),
        (
            11,
            "OUTLINE",
            "Board Outline Layer",
            "#ff0000",
            "#7f007f",
            1.0,
        ),
        (12, "MULTI", "Multi-Layer", "#0082bf", "#606060", 1.0),
        (47, "HOLE", "Hole Layer", "#222222", "#111111", 1.0),
        (57, "OTHER", "Ratline Layer", "#6464ff", "#32327f", 1.0),
    ] {
        writer.record_with_id(
            "LAYER",
            &format!("[\"LAYER\",{id}]"),
            &format!(
                concat!(
                    "{{\"layerId\":{},\"layerType\":\"{}\",\"layerName\":\"{}\",",
                    "\"use\":true,\"show\":true,\"locked\":false,",
                    "\"activeColor\":\"{}\",\"activateTransparency\":{},",
                    "\"inactiveColor\":\"{}\",\"inactiveTransparency\":0.5}}"
                ),
                id, layer_type, layer_name, active, alpha, inactive,
            ),
        );
    }
}

pub(crate) fn render_layer_phys(writer: &mut EpruWriter) {
    for (id, material, thickness, permittivity, loss_tangent, z_index) in [
        (3, "", 0.0, 0.0, 0.0, 1),
        (7, "", 0.0, 0.0, 0.0, 2),
        (5, "", 0.394, 3.3, 0.02, 3),
        (1, "", 1.378, 0.0, 0.0, 4),
        (361, "FR4", 59.449, 4.5, 0.0, 5),
        (2, "", 1.378, 0.0, 0.0, 6),
        (6, "", 0.394, 3.3, 0.02, 7),
        (8, "", 0.0, 0.0, 0.0, 8),
        (4, "", 0.0, 0.0, 0.0, 9),
    ] {
        writer.record_with_id(
            "LAYER_PHYS",
            &format!("[\"LAYER_PHYS\",{id}]"),
            &format!(
                concat!(
                    "{{\"material\":\"{}\",\"thickness\":{},",
                    "\"permittivity\":{},\"lossTangent\":{},",
                    "\"isKeepIsland\":true,\"zIndex\":{}}}"
                ),
                material, thickness, permittivity, loss_tangent, z_index,
            ),
        );
    }
}

pub(crate) fn lceda_layer_id(layer: &str) -> usize {
    match layer {
        "F.Cu" => 1,
        "B.Cu" => 2,
        "F.SilkS" => 3,
        "B.SilkS" => 4,
        "F.Mask" => 5,
        "B.Mask" => 6,
        "F.Paste" => 7,
        "B.Paste" => 8,
        "Edge.Cuts" => 11,
        "*.Cu" => 12,
        "F.Fab" | "B.Fab" | "Dwgs.User" => 13,
        _ => 13,
    }
}
