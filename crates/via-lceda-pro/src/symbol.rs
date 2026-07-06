use via_core::{Board, SymbolSide, model::Part};

use crate::epru::{EpruWriter, SymbolAttr};
use crate::ids::{footprint_name, json_escape, sheet_part_id, sheet_symbol_uuid};
use crate::model::{
    SymbolPinEntry, symbol_height_from_entries, symbol_name, symbol_part_id, symbol_pin_entries,
};

pub(crate) fn render_symbol_document(writer: &mut EpruWriter, module: &Part) {
    let pins = symbol_pin_entries(module);
    let height = symbol_height_from_entries(&pins);
    let part_id = symbol_part_id(module);

    writer.dochead("SYMBOL", &crate::ids::symbol_uuid(module));
    writer.record_with_id("CANVAS", "CANVAS", "{\"originX\":0,\"originY\":0}");
    writer.record_with_id(
        "PART",
        &part_id,
        &format!(
            "{{\"BBOX\":[-70,{},70,{}],\"title\":\"{}\"}}",
            -height / 2,
            height / 2,
            json_escape(&part_id),
        ),
    );
    writer.attr(SymbolAttr {
        id: "e1".to_owned(),
        part_id: Some(part_id.clone()),
        parent_id: "",
        key: "Symbol",
        value: &symbol_name(module),
        x: None,
        y: None,
        visible: false,
        z_index: 1,
    });
    writer.attr(SymbolAttr {
        id: "e2".to_owned(),
        part_id: Some(part_id.clone()),
        parent_id: "",
        key: "Designator",
        value: module.refdes(),
        x: None,
        y: None,
        visible: false,
        z_index: 2,
    });
    writer.attr(SymbolAttr {
        id: "e3".to_owned(),
        part_id: Some(part_id.clone()),
        parent_id: "",
        key: "Footprint",
        value: footprint_name(module),
        x: None,
        y: None,
        visible: false,
        z_index: 3,
    });
    writer.record_with_id(
        "RECT",
        "e4",
        &format!(
            concat!(
                "{{\"partId\":\"{}\",\"groupId\":\"\",\"locked\":false,\"zIndex\":4,",
                "\"dotX1\":-70,\"dotY1\":{},\"dotX2\":70,\"dotY2\":{},",
                "\"radiusX\":0,\"radiusY\":0,\"rotation\":0,\"strokeColor\":null,",
                "\"strokeStyle\":null,\"fillColor\":null,\"strokeWidth\":null,\"fillStyle\":null}}"
            ),
            json_escape(&part_id),
            -height / 2,
            height / 2,
        ),
    );
    writer.record_with_id(
        "ELE_PLACEHOLDER",
        "placeholder_pins",
        &format!("{{\"dataType\":\"PIN\",\"max\":{}}}", pins.len()),
    );

    for (index, pin) in pins.iter().enumerate() {
        let pin_id = format!("p{}", index + 1);
        let pin_name_x = pin_name_x(pin);
        let pin_number_x = pin_number_x(pin);
        writer.record_with_id(
            "PIN",
            &pin_id,
            &format!(
                concat!(
                    "{{\"partId\":\"{}\",\"groupId\":\"\",\"locked\":false,\"zIndex\":{},",
                    "\"display\":true,\"x\":{},\"y\":{},",
                    "\"length\":20,\"rotation\":{},\"color\":null,\"pinShape\":\"NONE\"}}"
                ),
                json_escape(&part_id),
                10 + index * 4,
                pin.x,
                pin.y,
                pin.rotation,
            ),
        );
        writer.attr(SymbolAttr {
            id: format!("{pin_id}_name"),
            part_id: Some(part_id.clone()),
            parent_id: &pin_id,
            key: "Pin Name",
            value: &pin.logical_name,
            x: Some(pin_name_x),
            y: Some(pin.y - 5),
            visible: true,
            z_index: 11 + index * 4,
        });
        writer.attr(SymbolAttr {
            id: format!("{pin_id}_num"),
            part_id: Some(part_id.clone()),
            parent_id: &pin_id,
            key: "Pin Number",
            value: &pin.pad_number,
            x: Some(pin_number_x),
            y: Some(pin.y - 5),
            visible: false,
            z_index: 12 + index * 4,
        });
        writer.attr(SymbolAttr {
            id: format!("{pin_id}_type"),
            part_id: Some(part_id.clone()),
            parent_id: &pin_id,
            key: "Pin Type",
            value: "IN",
            x: Some(pin.x),
            y: Some(pin.y),
            visible: false,
            z_index: 13 + index * 4,
        });
    }

    writer.record_with_id(
        "META",
        "META",
        &format!(
            concat!(
                "{{\"title\":\"{}\",\"description\":\"{}\",\"tags\":[],",
                "\"docType\":2,\"source\":\"\"}}"
            ),
            json_escape(&symbol_name(module)),
            json_escape(module.value()),
        ),
    );
}

fn pin_name_x(pin: &SymbolPinEntry) -> i32 {
    match pin.side {
        SymbolSide::Left => -64,
        SymbolSide::Right => 64,
        SymbolSide::Top | SymbolSide::Bottom => pin.x - 20,
    }
}

fn pin_number_x(pin: &SymbolPinEntry) -> i32 {
    match pin.side {
        SymbolSide::Left => -84,
        SymbolSide::Right => 84,
        SymbolSide::Top | SymbolSide::Bottom => pin.x - 8,
    }
}

pub(crate) fn render_sheet_symbol_document(writer: &mut EpruWriter, board: &Board) {
    writer.dochead("SYMBOL", &sheet_symbol_uuid(board.name()));
    writer.record_with_id("CANVAS", "CANVAS", "{\"originX\":0,\"originY\":0}");
    writer.record_with_id(
        "PART",
        sheet_part_id(),
        "{\"BBOX\":[0,825,1170,0],\"title\":\"\"}",
    );
    writer.record_with_id("GROUP", "1", "{\"parentId\":\"\",\"title\":\"border\"}");

    for (index, (key, value, x, y, align)) in [
        ("Symbol", "Sheet-Symbol_A4", 2506, 116, "CENTER_MIDDLE"),
        ("Company", "via-rs", 998, -30, "CENTER_MIDDLE"),
        ("Version", "V0", 718, -30, "CENTER_MIDDLE"),
        ("Page Size", "A4", 800, -30, "CENTER_MIDDLE"),
        ("@Project Name", board.name(), 920, -100, "CENTER_MIDDLE"),
        ("@Page Count", "1", 1102, -61, "CENTER_MIDDLE"),
        ("@Page No", "1", 985, -61, "CENTER_MIDDLE"),
        ("@Page Name", "P1", 730, -140, "CENTER_MIDDLE"),
        ("@Schematic Name", "原理图", 730, -170, "CENTER_MIDDLE"),
    ]
    .into_iter()
    .enumerate()
    {
        writer.record_with_id(
            "ATTR",
            &format!("sheet_attr_{index}"),
            &format!(
                concat!(
                    "{{\"partId\":\"{}\",\"groupId\":\"\",\"locked\":false,",
                    "\"zIndex\":{},\"parentId\":\"\",\"key\":\"{}\",",
                    "\"value\":\"{}\",\"keyVisible\":false,\"valueVisible\":true,",
                    "\"x\":{},\"y\":{},\"rotation\":0,\"color\":null,",
                    "\"fillColor\":null,\"fontFamily\":null,\"fontSize\":15,",
                    "\"strikeout\":null,\"underline\":false,\"italic\":false,",
                    "\"fontWeight\":false,\"align\":\"{}\",\"version\":\"2.0\"}}"
                ),
                sheet_part_id(),
                index + 1,
                json_escape(key),
                json_escape(value),
                x,
                y,
                align,
            ),
        );
    }

    writer.record_with_id(
        "RECT",
        "sheet_rect_outer",
        concat!(
            "{\"partId\":\"pid8a0e77bacb214e\",\"groupId\":\"1\",",
            "\"locked\":false,\"zIndex\":20,\"dotX1\":0,\"dotY1\":0,",
            "\"dotX2\":1170,\"dotY2\":-825,\"radiusX\":0,\"radiusY\":0,",
            "\"rotation\":0,\"strokeColor\":null,\"strokeStyle\":null,",
            "\"fillColor\":null,\"strokeWidth\":null,\"fillStyle\":null}"
        ),
    );
    writer.record_with_id(
        "RECT",
        "sheet_rect_title",
        concat!(
            "{\"partId\":\"pid8a0e77bacb214e\",\"groupId\":\"1\",",
            "\"locked\":false,\"zIndex\":21,\"dotX1\":460,\"dotY1\":-190,",
            "\"dotX2\":1160,\"dotY2\":0,\"radiusX\":0,\"radiusY\":0,",
            "\"rotation\":0,\"strokeColor\":null,\"strokeStyle\":null,",
            "\"fillColor\":null,\"strokeWidth\":null,\"fillStyle\":null}"
        ),
    );
    writer.record_with_id(
        "META",
        "META",
        "{\"title\":\"Sheet-Symbol_A4\",\"description\":\"\",\"tags\":[\"\",\"\"],\"docType\":21,\"source\":\"\"}",
    );
}
