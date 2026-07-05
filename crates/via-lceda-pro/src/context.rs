use std::collections::BTreeMap;

use via_core::{Board, FootprintPads};

pub(crate) struct ExportContext<'a> {
    board: &'a Board,
    footprints: BTreeMap<String, FootprintPads>,
}

impl<'a> ExportContext<'a> {
    pub(crate) fn new(board: &'a Board) -> Self {
        let mut footprints = board
            .footprints()
            .map(Clone::clone)
            .map(|footprint| (footprint.name().to_owned(), footprint))
            .collect::<BTreeMap<_, _>>();

        for module in board.modules() {
            if let Some(name) = module.footprint_name() {
                footprints
                    .entry(name.to_owned())
                    .or_insert_with(|| FootprintPads::new(name, module.modeled_pads()));
            }
        }

        Self { board, footprints }
    }

    pub(crate) fn board(&self) -> &'a Board {
        self.board
    }

    pub(crate) fn footprints(&self) -> impl Iterator<Item = &FootprintPads> {
        self.footprints.values()
    }

    pub(crate) fn footprint(&self, name: &str) -> Option<&FootprintPads> {
        self.footprints.get(name)
    }
}
