use std::io;
use std::path::Path;

use via_core::{Board, atomic_write};

use crate::archive::ZipArchive;
use crate::context::ExportContext;
use crate::device::{render_device_document, render_sheet_device_document};
use crate::epru::EpruWriter;
use crate::footprint::render_footprint_document;
use crate::pcb::render_pcb_document;
use crate::project::{render_config_document, render_font_document, render_project2_json};
use crate::schematic::{
    render_board_document, render_schematic_document, render_schematic_page_document,
};
use crate::symbol::{render_sheet_symbol_document, render_symbol_document};
use crate::validate::validate_lceda_export;

pub fn write_lceda_pro_project(board: &Board, path: impl AsRef<Path>) -> io::Result<()> {
    board
        .check()
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))?;

    let path = path.as_ref();
    let title = board.name();
    let ctx = ExportContext::new(board);
    validate_lceda_export(&ctx).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{}\n{}", error, error.diagnostics().join("\n")),
        )
    })?;

    let mut archive = ZipArchive::new();
    archive.add_file("IMAGE/".to_owned(), Vec::new());
    archive.add_file(
        "project2.json".to_owned(),
        render_project2_json(title).into_bytes(),
    );
    archive.add_file(
        format!("{title}.epru"),
        render_epru_with_context(&ctx).into_bytes(),
    );
    atomic_write(path, archive.finish()?).map_err(|err| io::Error::other(err.to_string()))
}

#[cfg(test)]
pub(crate) fn render_epru(board: &Board) -> String {
    let ctx = ExportContext::new(board);
    render_epru_with_context(&ctx)
}

fn render_epru_with_context(ctx: &ExportContext<'_>) -> String {
    let board = ctx.board();
    let mut writer = EpruWriter::new();
    for footprint in ctx.footprints() {
        render_footprint_document(&mut writer, footprint);
    }
    for module in board.modules() {
        render_symbol_document(&mut writer, module);
    }
    render_sheet_symbol_document(&mut writer, board);
    for module in board.modules() {
        render_device_document(&mut writer, module);
    }
    render_sheet_device_document(&mut writer, board);
    render_board_document(&mut writer, board);
    render_schematic_document(&mut writer, board);
    render_schematic_page_document(&mut writer, board);
    render_pcb_document(&mut writer, board);
    render_config_document(&mut writer, board.name());
    render_font_document(&mut writer);
    writer.finish()
}
