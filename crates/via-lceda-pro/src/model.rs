use std::collections::BTreeMap;

use via_core::{Board, Part};

use crate::ids::sanitize_id;

#[derive(Debug)]
pub(crate) struct Placement {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) z_index: usize,
}

pub(crate) struct SymbolPinEntry {
    pub(crate) logical_name: String,
    pub(crate) pad_number: String,
}

pub(crate) fn symbol_pin_entries(module: &Part) -> Vec<SymbolPinEntry> {
    module
        .pins_iter()
        .flat_map(|pin| {
            module
                .pads_for_pin(pin)
                .into_iter()
                .map(move |pad| SymbolPinEntry {
                    logical_name: pin.clone(),
                    pad_number: pad,
                })
        })
        .collect()
}

pub(crate) fn module_placements(board: &Board) -> BTreeMap<String, Placement> {
    let preferred = [
        ("J1", (115, -235)),
        ("U4", (375, -235)),
        ("U1", (625, -420)),
        ("R1", (860, -250)),
        ("R2", (860, -620)),
        ("C1", (280, -300)),
        ("C2", (280, -185)),
        ("C3", (870, -130)),
        ("C4", (870, -380)),
        ("C5", (870, -500)),
        ("C6", (870, -750)),
        ("C7", (380, -100)),
        ("C8", (560, -100)),
        ("C9", (650, -220)),
        ("C10", (530, -340)),
        ("U2", (1000, -250)),
        ("U3", (1000, -620)),
        ("J2", (1160, -250)),
        ("J3", (1160, -620)),
        ("J6", (160, -665)),
    ];
    let mut placements = BTreeMap::new();
    for (index, module) in board.modules().enumerate() {
        let (x, y) = preferred
            .iter()
            .find(|(refdes, _)| *refdes == module.refdes())
            .map(|(_, position)| *position)
            .unwrap_or_else(|| {
                let column = index % 3;
                let row = index / 3;
                (120 + column as i32 * 360, -120 - row as i32 * 240)
            });
        placements.insert(
            module.refdes().to_owned(),
            Placement {
                x,
                y,
                z_index: 100 + index * 10,
            },
        );
    }
    placements
}

pub(crate) fn pcb_component_placement(index: usize) -> Placement {
    let column = index % 5;
    let row = index / 5;
    Placement {
        x: 300 + column as i32 * 450,
        y: 300 + row as i32 * 350,
        z_index: index + 1,
    }
}

pub(crate) fn symbol_height(pin_count: usize) -> i32 {
    (pin_count.max(2) as i32 * 16).max(64)
}

pub(crate) fn symbol_pin_y(index: usize, pin_count: usize) -> i32 {
    let top = ((pin_count.max(2) as i32 - 1) * 16) / 2;
    top - index as i32 * 16
}

pub(crate) fn symbol_name(module: &Part) -> String {
    format!("via_{}_{}", module.refdes(), sanitize_id(module.value()))
}

pub(crate) fn symbol_part_id(module: &Part) -> String {
    format!("{}.1", symbol_name(module))
}
