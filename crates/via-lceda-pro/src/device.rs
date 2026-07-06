use via_core::{Board, model::Part};

use crate::epru::EpruWriter;
use crate::ids::{
    device_uuid, footprint_uuid, json_escape, sheet_device_uuid, sheet_symbol_uuid, symbol_uuid,
};

pub(crate) fn render_device_document(writer: &mut EpruWriter, module: &Part) {
    writer.dochead("DEVICE", &device_uuid(module));
    let footprint_uuid = module
        .footprint_name()
        .map(footprint_uuid)
        .unwrap_or_default();
    writer.record_with_id(
        "META",
        "META",
        &format!(
            concat!(
                "{{\"title\":\"{}\",\"tags\":[],\"source\":\"\",",
                "\"images\":[],\"attributes\":{{",
                "\"Symbol\":\"{}\",\"Footprint\":\"{}\",\"Name\":\"{}\",",
                "\"Designator\":\"{}\",\"Add into BOM\":\"yes\",",
                "\"Convert to PCB\":\"yes\",\"Description\":\"{}\"}}}}"
            ),
            json_escape(module.value()),
            json_escape(&symbol_uuid(module)),
            json_escape(&footprint_uuid),
            json_escape(module.value()),
            json_escape(module.refdes()),
            if module.requires_verification() {
                "VERIFY footprint against purchased module before fabrication"
            } else {
                ""
            },
        ),
    );
}

pub(crate) fn render_sheet_device_document(writer: &mut EpruWriter, board: &Board) {
    writer.dochead("DEVICE", &sheet_device_uuid(board.name()));
    writer.record_with_id(
        "META",
        "META",
        &format!(
            concat!(
                "{{\"title\":\"Sheet-Symbol_A4\",\"tags\":[\"\",\"\"],",
                "\"source\":\"\",\"images\":[\"\"],\"attributes\":{{",
                "\"Symbol\":\"{}\",\"@Page Name\":\"P1\",",
                "\"@Page Count\":\"1\",\"@Page No\":\"1\",",
                "\"@Project Name\":\"{}\",\"@Schematic Name\":\"原理图\",",
                "\"Company\":\"via-rs\",\"Page Size\":\"A4\",",
                "\"Version\":\"V0\",\"Footprint\":\"\",\"Description\":\"\"}}}}"
            ),
            json_escape(&sheet_symbol_uuid(board.name())),
            json_escape(board.name()),
        ),
    );
}
