use via_footprint_ir::{GraphicLine, GraphicText, Pad, PadKind, PadShape, TextJustify};

use crate::epru::EpruWriter;
use crate::ids::{footprint_pad_id, json_escape};
use crate::layers::lceda_layer_id;
use crate::units::{lceda_mm, lceda_number};

pub(crate) struct FootprintPadRecord {
    id: String,
    number: String,
    layer_id: usize,
    center_x: String,
    center_y: String,
    hole: String,
    default_pad: String,
    plated: bool,
    z_index: usize,
    include_unused_inner_layers: bool,
}

impl FootprintPadRecord {
    pub(crate) fn from_ir(pad: &Pad, z_index: usize) -> Self {
        Self {
            id: footprint_pad_id(&pad.number),
            number: pad.number.clone(),
            layer_id: pad_layer_id(pad),
            center_x: lceda_mm(pad.at.x),
            center_y: lceda_mm(pad.at.y),
            hole: lceda_pad_hole(pad),
            default_pad: lceda_default_pad(pad),
            plated: !matches!(pad.kind, PadKind::NpThruHole),
            z_index,
            include_unused_inner_layers: true,
        }
    }

    pub(crate) fn placeholder(
        pad_name: &str,
        center_x_mil: f64,
        center_y_mil: f64,
        z_index: usize,
    ) -> Self {
        Self {
            id: footprint_pad_id(pad_name),
            number: pad_name.to_owned(),
            layer_id: 1,
            center_x: lceda_number(center_x_mil),
            center_y: lceda_number(center_y_mil),
            hole: "null".to_owned(),
            default_pad: "{\"padType\":\"RECT\",\"width\":55,\"height\":55,\"radius\":0}"
                .to_owned(),
            plated: true,
            z_index,
            include_unused_inner_layers: false,
        }
    }

    pub(crate) fn write(&self, writer: &mut EpruWriter) {
        let unused_inner_layers = if self.include_unused_inner_layers {
            "\"unusedInnerLayers\":[],"
        } else {
            ""
        };
        writer.record_with_id(
            "PAD",
            &self.id,
            &format!(
                concat!(
                    "{{\"groupId\":0,\"netName\":\"\",\"layerId\":{},",
                    "\"num\":\"{}\",\"centerX\":{},\"centerY\":{},",
                    "\"padAngle\":0,\"hole\":{},\"defaultPad\":{},",
                    "\"specialPad\":[],\"padOffsetX\":0,\"padOffsetY\":0,",
                    "\"relativeAngle\":0,\"plated\":{},\"padType\":\"NORMAL\",",
                    "\"topSolderExpansion\":null,\"bottomSolderExpansion\":null,",
                    "\"topPasteExpansion\":null,\"bottomPasteExpansion\":null,",
                    "\"locked\":false,\"zIndex\":{},\"connectMode\":null,",
                    "\"spokeSpace\":null,\"spokeWidth\":null,\"spokeAngle\":null,",
                    "{}\"padLen\":0}}"
                ),
                self.layer_id,
                json_escape(&self.number),
                self.center_x,
                self.center_y,
                self.hole,
                self.default_pad,
                self.plated,
                self.z_index,
                unused_inner_layers,
            ),
        );
    }
}

pub(crate) struct FootprintLineRecord {
    id: String,
    layer_id: usize,
    start_x: String,
    start_y: String,
    end_x: String,
    end_y: String,
    width: String,
    z_index: usize,
}

impl FootprintLineRecord {
    pub(crate) fn from_ir(line: &GraphicLine, z_index: usize) -> Self {
        Self {
            id: format!("line_{}", z_index - 300),
            layer_id: lceda_layer_id(&line.layer),
            start_x: lceda_mm(line.start.x),
            start_y: lceda_mm(line.start.y),
            end_x: lceda_mm(line.end.x),
            end_y: lceda_mm(line.end.y),
            width: lceda_mm(line.width),
            z_index,
        }
    }

    pub(crate) fn write(&self, writer: &mut EpruWriter) {
        writer.record_with_id(
            "LINE",
            &self.id,
            &format!(
                concat!(
                    "{{\"partitionId\":\"\",\"groupId\":0,\"locked\":false,",
                    "\"zIndex\":{},\"netName\":\"\",\"layerId\":{},",
                    "\"startX\":{},\"startY\":{},\"endX\":{},\"endY\":{},",
                    "\"width\":{}}}"
                ),
                self.z_index,
                self.layer_id,
                self.start_x,
                self.start_y,
                self.end_x,
                self.end_y,
                self.width,
            ),
        );
    }
}

pub(crate) struct FootprintTextRecord {
    id: String,
    layer_id: usize,
    x: String,
    y: String,
    text: String,
    font_size: String,
    stroke_width: String,
    origin: &'static str,
    angle: String,
    z_index: usize,
}

impl FootprintTextRecord {
    pub(crate) fn from_ir(text: &GraphicText, z_index: usize) -> Self {
        Self {
            id: format!("text_{}", z_index - 500),
            layer_id: lceda_layer_id(&text.layer),
            x: lceda_mm(text.at.x),
            y: lceda_mm(text.at.y),
            text: text.text.clone(),
            font_size: lceda_mm(text.size.x),
            stroke_width: lceda_mm(text.thickness),
            origin: match text.justify {
                Some(TextJustify::Left) => "LEFT_BOTTOM",
                Some(TextJustify::Right) => "RIGHT_BOTTOM",
                Some(TextJustify::Center) | None => "CENTER_MIDDLE",
            },
            angle: lceda_number(text.rotation),
            z_index,
        }
    }

    pub(crate) fn write(&self, writer: &mut EpruWriter) {
        writer.record_with_id(
            "STRING",
            &self.id,
            &format!(
                concat!(
                    "{{\"partitionId\":\"\",\"groupId\":0,\"layerId\":{},",
                    "\"x\":{},\"y\":{},\"text\":\"{}\",\"fontFamily\":\"default\",",
                    "\"fontSize\":{},\"strokeWidth\":{},\"bold\":false,\"italic\":false,",
                    "\"origin\":\"{}\",\"angle\":{},\"reverse\":false,\"expansion\":0,",
                    "\"mirror\":false,\"locked\":false,\"zIndex\":{},\"specialColor\":null}}"
                ),
                self.layer_id,
                self.x,
                self.y,
                json_escape(&self.text),
                self.font_size,
                self.stroke_width,
                self.origin,
                self.angle,
                self.z_index,
            ),
        );
    }
}

fn pad_layer_id(pad: &Pad) -> usize {
    if matches!(pad.kind, PadKind::ThruHole | PadKind::NpThruHole)
        || pad.layers.iter().any(|layer| layer == "*.Cu")
    {
        12
    } else if pad.layers.iter().any(|layer| layer == "B.Cu")
        && !pad.layers.iter().any(|layer| layer == "F.Cu")
    {
        2
    } else {
        1
    }
}

fn lceda_pad_hole(pad: &Pad) -> String {
    pad.drill
        .map(|drill| {
            let hole_type = if drill.is_round() { "ROUND" } else { "SLOT" };
            format!(
                "{{\"holeType\":\"{}\",\"width\":{},\"height\":{}}}",
                hole_type,
                lceda_mm(drill.x),
                lceda_mm(drill.y),
            )
        })
        .unwrap_or_else(|| "null".to_owned())
}

fn lceda_default_pad(pad: &Pad) -> String {
    match pad.shape {
        PadShape::Circle => format!(
            "{{\"padType\":\"ELLIPSE\",\"width\":{},\"height\":{}}}",
            lceda_mm(pad.size.x),
            lceda_mm(pad.size.y),
        ),
        PadShape::Oval => format!(
            "{{\"padType\":\"OVAL\",\"width\":{},\"height\":{}}}",
            lceda_mm(pad.size.x),
            lceda_mm(pad.size.y),
        ),
        PadShape::Rect | PadShape::Trapezoid => format!(
            "{{\"padType\":\"RECT\",\"width\":{},\"height\":{},\"radius\":0}}",
            lceda_mm(pad.size.x),
            lceda_mm(pad.size.y),
        ),
        PadShape::RoundRect => {
            let radius = pad.size.x.min(pad.size.y) * 0.25;
            format!(
                "{{\"padType\":\"RECT\",\"width\":{},\"height\":{},\"radius\":{}}}",
                lceda_mm(pad.size.x),
                lceda_mm(pad.size.y),
                lceda_mm(radius),
            )
        }
    }
}
