use via_core::{Board, SymbolSide};

use crate::epru::{EpruWriter, SymbolAttr};
use crate::ids::{
    board_uuid, component_id, device_uuid, footprint_uuid, json_escape, schematic_page_uuid,
    schematic_uuid, sheet_device_uuid, sheet_part_id, sheet_symbol_uuid, stable_uuid, symbol_uuid,
};
use crate::model::{
    module_placements, symbol_height_from_entries, symbol_part_id, symbol_pin_entries,
};

pub(crate) fn render_board_document(writer: &mut EpruWriter, board: &Board) {
    writer.dochead("BOARD", &board_uuid(board.name()));
    writer.record_with_id(
        "META",
        "META",
        &format!(
            "{{\"title\":\"{}\",\"zIndex\":1}}",
            json_escape(board.name())
        ),
    );
}

pub(crate) fn render_schematic_document(writer: &mut EpruWriter, board: &Board) {
    writer.dochead("SCH", &schematic_uuid(board.name()));
    writer.record_with_id(
        "META",
        "META",
        &format!(
            concat!("{{\"title\":\"原理图\",\"board\":\"{}\",\"zIndex\":null}}"),
            board_uuid(board.name()),
        ),
    );
}

pub(crate) fn render_schematic_page_document(writer: &mut EpruWriter, board: &Board) {
    let placements = module_placements(board);

    writer.dochead("SCH_PAGE", &schematic_page_uuid(board.name()));
    writer.record_with_id("CANVAS", "CANVAS", "{\"originX\":0,\"originY\":0}");
    render_sheet_component(writer, board);
    writer.record_with_id(
        "ELE_PLACEHOLDER",
        "placeholder_components",
        &format!(
            "{{\"dataType\":\"COMPONENT\",\"max\":{}}}",
            board.modules().count() + 1
        ),
    );

    for module in board.modules() {
        let placement = placements
            .get(module.refdes())
            .expect("missing module placement");
        let component_id = component_id(module.refdes());
        writer.record_with_id(
            "COMPONENT",
            &component_id,
            &format!(
                concat!(
                    "{{\"partId\":\"{}\",\"groupId\":\"\",\"locked\":false,\"zIndex\":{},",
                    "\"x\":{},\"y\":{},\"rotation\":0,",
                    "\"isMirror\":false,\"attrs\":{{}}}}"
                ),
                json_escape(&symbol_part_id(module)),
                placement.z_index,
                placement.x,
                placement.y,
            ),
        );
        writer.attr(SymbolAttr {
            id: format!("{component_id}_des"),
            part_id: None,
            parent_id: &component_id,
            key: "Designator",
            value: module.refdes(),
            x: Some(placement.x - 40),
            y: Some(placement.y - symbol_height_from_entries(&symbol_pin_entries(module)) / 2 - 30),
            visible: true,
            z_index: placement.z_index + 1,
        });
        writer.attr(SymbolAttr {
            id: format!("{component_id}_name"),
            part_id: None,
            parent_id: &component_id,
            key: "Name",
            value: module.value(),
            x: Some(placement.x - 40),
            y: Some(placement.y + symbol_height_from_entries(&symbol_pin_entries(module)) / 2 + 15),
            visible: true,
            z_index: placement.z_index + 2,
        });
        let symbol_uuid = symbol_uuid(module);
        writer.attr(SymbolAttr {
            id: format!("{component_id}_symbol"),
            part_id: None,
            parent_id: &component_id,
            key: "Symbol",
            value: &symbol_uuid,
            x: None,
            y: None,
            visible: false,
            z_index: placement.z_index + 3,
        });
        let device_uuid = device_uuid(module);
        writer.attr(SymbolAttr {
            id: format!("{component_id}_device"),
            part_id: None,
            parent_id: &component_id,
            key: "Device",
            value: &device_uuid,
            x: None,
            y: None,
            visible: false,
            z_index: placement.z_index + 4,
        });
        let footprint_uuid = module
            .footprint_name()
            .map(footprint_uuid)
            .unwrap_or_default();
        writer.attr(SymbolAttr {
            id: format!("{component_id}_footprint"),
            part_id: None,
            parent_id: &component_id,
            key: "Footprint",
            value: &footprint_uuid,
            x: None,
            y: None,
            visible: false,
            z_index: placement.z_index + 5,
        });
        let unique_id = stable_uuid(&format!("sch-component:{}", module.refdes()));
        writer.attr(SymbolAttr {
            id: format!("{component_id}_uid"),
            part_id: None,
            parent_id: &component_id,
            key: "Unique ID",
            value: &unique_id,
            x: None,
            y: None,
            visible: false,
            z_index: placement.z_index + 6,
        });
    }

    let mut wire_index = 0;
    for (net_index, net) in board.nets().enumerate() {
        for pin_ref in net.connections() {
            let Some(module) = board.module(&pin_ref.module) else {
                continue;
            };
            let Some(placement) = placements.get(module.refdes()) else {
                continue;
            };
            let symbol_pins = symbol_pin_entries(module);
            for symbol_pin in symbol_pins
                .iter()
                .filter(|symbol_pin| symbol_pin.logical_name == pin_ref.pin)
            {
                let pin_x = placement.x + symbol_pin.x;
                let pin_y = placement.y + symbol_pin.y;
                let (stub_x, stub_y, label_x, label_y, label_rotation, align) =
                    net_stub_geometry(pin_x, pin_y, symbol_pin.side);

                writer.wire(
                    &format!("w{net_index}_{wire_index}"),
                    5000 + wire_index,
                    net.name(),
                    &[(pin_x, pin_y), (stub_x, stub_y)],
                );
                writer.record_with_id(
                    "TEXT",
                    &format!("w{net_index}_{wire_index}_label"),
                    &format!(
                        concat!(
                            "{{\"partId\":\"\",\"groupId\":\"\",\"locked\":false,\"zIndex\":{},",
                            "\"x\":{},\"y\":{},\"rotation\":{},\"value\":\"{}\",",
                            "\"color\":\"#666666\",\"fillColor\":null,\"fontFamily\":null,",
                            "\"fontSize\":12,\"fontWeight\":null,\"italic\":null,",
                            "\"underline\":null,\"strikeout\":null,\"align\":\"{}\",",
                            "\"version\":\"2.0\"}}"
                        ),
                        7000 + wire_index,
                        label_x,
                        label_y,
                        label_rotation,
                        json_escape(net.name()),
                        align,
                    ),
                );
                wire_index += 1;
            }
        }
    }

    writer.record_with_id(
        "META",
        "META",
        &format!(
            "{{\"title\":\"P1\",\"schematic\":\"{}\",\"zIndex\":1}}",
            schematic_uuid(board.name()),
        ),
    );
}

fn net_stub_geometry(
    pin_x: i32,
    pin_y: i32,
    side: SymbolSide,
) -> (i32, i32, i32, i32, i32, &'static str) {
    match side {
        SymbolSide::Left => (pin_x - 45, pin_y, pin_x - 165, pin_y - 6, 0, "LEFT_BOTTOM"),
        SymbolSide::Right => (pin_x + 45, pin_y, pin_x + 53, pin_y - 6, 0, "LEFT_BOTTOM"),
        SymbolSide::Top => (pin_x, pin_y - 45, pin_x + 8, pin_y - 53, 90, "LEFT_BOTTOM"),
        SymbolSide::Bottom => (pin_x, pin_y + 45, pin_x + 8, pin_y + 53, 90, "LEFT_BOTTOM"),
    }
}

fn render_sheet_component(writer: &mut EpruWriter, board: &Board) {
    writer.record_with_id(
        "COMPONENT",
        "e1",
        &format!(
            concat!(
                "{{\"partId\":\"{}\",\"groupId\":\"\",\"locked\":false,",
                "\"zIndex\":1,\"x\":0,\"y\":0,\"rotation\":0,",
                "\"isMirror\":false,\"attrs\":{{}}}}"
            ),
            sheet_part_id(),
        ),
    );
    for (index, (key, value, x, y, visible)) in [
        ("Description", "", None, None, false),
        (
            "Symbol",
            &sheet_symbol_uuid(board.name()),
            None,
            None,
            false,
        ),
        ("Footprint", "", None, None, false),
        ("Company", "via-rs", None, None, true),
        ("Version", "V0", None, None, true),
        ("Page Size", "A4", None, None, true),
        ("@Project Name", board.name(), None, None, true),
        ("@Page Count", "1", None, None, true),
        ("@Page No", "1", None, None, true),
        ("@Page Name", "P1", None, None, true),
        ("@Schematic Name", "原理图", None, None, true),
        (
            "Device",
            &sheet_device_uuid(board.name()),
            None,
            None,
            false,
        ),
        ("@Board Name", board.name(), None, None, false),
    ]
    .into_iter()
    .enumerate()
    {
        writer.attr(SymbolAttr {
            id: format!("sheet_component_attr_{index}"),
            part_id: None,
            parent_id: "e1",
            key,
            value,
            x,
            y,
            visible,
            z_index: 2 + index,
        });
    }
}
