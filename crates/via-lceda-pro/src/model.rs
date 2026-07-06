use std::collections::{BTreeMap, BTreeSet};

use via_core::{Board, SymbolSide, model::Part};

use crate::ids::sanitize_id;

const PIN_SPACING: i32 = 16;
const PIN_X: i32 = 90;

#[derive(Debug)]
pub(crate) struct Placement {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) z_index: usize,
}

pub(crate) struct SymbolPinEntry {
    pub(crate) logical_name: String,
    pub(crate) pad_number: String,
    pub(crate) side: SymbolSide,
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) rotation: i32,
}

pub(crate) fn symbol_pin_entries(module: &Part) -> Vec<SymbolPinEntry> {
    let mut pins = match module.symbol_spec() {
        Some(_) => explicit_symbol_pin_entries(module),
        None => auto_symbol_pin_entries(module),
    };
    layout_symbol_pin_entries(&mut pins);
    pins
}

pub(crate) fn module_placements(board: &Board) -> BTreeMap<String, Placement> {
    let mut placements = BTreeMap::new();
    for (index, module) in board.modules().enumerate() {
        let column = index % 3;
        let row = index / 3;
        let x = 120 + column as i32 * 360;
        let y = -120 - row as i32 * 240;
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

pub(crate) fn symbol_height_from_entries(pins: &[SymbolPinEntry]) -> i32 {
    let side_rows = pins
        .iter()
        .filter(|pin| matches!(pin.side, SymbolSide::Left | SymbolSide::Right))
        .fold((0usize, 0usize), |(left, right), pin| match pin.side {
            SymbolSide::Left => (left + 1, right),
            _ => (left, right + 1),
        })
        .0
        .max(
            pins.iter()
                .filter(|pin| !matches!(pin.side, SymbolSide::Left))
                .count(),
        )
        .max(2);
    (side_rows as i32 * PIN_SPACING).max(64)
}

fn layout_symbol_pin_entries(pins: &mut [SymbolPinEntry]) {
    let left_count = pins
        .iter()
        .filter(|pin| pin.side == SymbolSide::Left)
        .count();
    let right_count = pins
        .iter()
        .filter(|pin| pin.side == SymbolSide::Right)
        .count();
    let top_count = pins
        .iter()
        .filter(|pin| pin.side == SymbolSide::Top)
        .count();
    let bottom_count = pins
        .iter()
        .filter(|pin| pin.side == SymbolSide::Bottom)
        .count();
    let side_rows = left_count.max(right_count).max(1);
    let top_y = (side_rows.saturating_sub(1) as i32 * PIN_SPACING) / 2;
    let body_top = top_y + PIN_SPACING;
    let body_bottom = -body_top;

    let mut left_idx = 0usize;
    let mut right_idx = 0usize;
    let mut top_idx = 0usize;
    let mut bottom_idx = 0usize;

    for pin in pins {
        match pin.side {
            SymbolSide::Left => {
                pin.x = -PIN_X;
                pin.y = top_y - left_idx as i32 * PIN_SPACING;
                pin.rotation = 0;
                left_idx += 1;
            }
            SymbolSide::Right => {
                pin.x = PIN_X;
                pin.y = top_y - right_idx as i32 * PIN_SPACING;
                pin.rotation = 180;
                right_idx += 1;
            }
            SymbolSide::Top => {
                pin.x = centered_axis_position(top_idx, top_count);
                pin.y = body_top;
                pin.rotation = 270;
                top_idx += 1;
            }
            SymbolSide::Bottom => {
                pin.x = centered_axis_position(bottom_idx, bottom_count);
                pin.y = body_bottom;
                pin.rotation = 90;
                bottom_idx += 1;
            }
        }
    }
}

fn centered_axis_position(index: usize, count: usize) -> i32 {
    let center2 = count.saturating_sub(1) as i32;
    (index as i32 * 2 - center2) * PIN_SPACING / 2
}

fn auto_symbol_pin_entries(module: &Part) -> Vec<SymbolPinEntry> {
    let mut pins = physical_symbol_pin_entries(module, SymbolSide::Left);
    let left_count = pins.len().div_ceil(2);

    for (idx, pin) in pins.iter_mut().enumerate() {
        pin.side = if idx < left_count {
            SymbolSide::Left
        } else {
            SymbolSide::Right
        };
    }

    pins
}

fn explicit_symbol_pin_entries(module: &Part) -> Vec<SymbolPinEntry> {
    let Some(spec) = module.symbol_spec() else {
        return auto_symbol_pin_entries(module);
    };
    let mut pins = Vec::new();
    let mut seen_numbers = BTreeSet::new();
    let mut specified_logical_pins = BTreeSet::new();

    for side in [
        SymbolSide::Left,
        SymbolSide::Right,
        SymbolSide::Top,
        SymbolSide::Bottom,
    ] {
        for logical_pin in spec.pins_on(side) {
            specified_logical_pins.insert(logical_pin.clone());
            push_physical_symbol_pin_entries(
                module,
                logical_pin,
                side,
                &mut seen_numbers,
                &mut pins,
            );
        }
    }

    for logical_pin in module.pins_iter() {
        if !specified_logical_pins.contains(logical_pin) {
            push_physical_symbol_pin_entries(
                module,
                logical_pin,
                SymbolSide::Right,
                &mut seen_numbers,
                &mut pins,
            );
        }
    }

    pins
}

fn physical_symbol_pin_entries(module: &Part, side: SymbolSide) -> Vec<SymbolPinEntry> {
    let mut pins = Vec::new();
    let mut seen_numbers = BTreeSet::new();

    for logical_pin in module.pins_iter() {
        push_physical_symbol_pin_entries(module, logical_pin, side, &mut seen_numbers, &mut pins);
    }

    pins
}

fn push_physical_symbol_pin_entries(
    module: &Part,
    logical_pin: &str,
    side: SymbolSide,
    seen_numbers: &mut BTreeSet<String>,
    pins: &mut Vec<SymbolPinEntry>,
) {
    for pad_number in module.pads_for_pin(logical_pin) {
        if seen_numbers.insert(pad_number.clone()) {
            pins.push(SymbolPinEntry {
                logical_name: logical_pin.to_owned(),
                pad_number,
                side,
                x: 0,
                y: 0,
                rotation: 0,
            });
        }
    }
}

pub(crate) fn symbol_name(module: &Part) -> String {
    format!("via_{}_{}", module.refdes(), sanitize_id(module.value()))
}

pub(crate) fn symbol_part_id(module: &Part) -> String {
    format!("{}.1", symbol_name(module))
}
