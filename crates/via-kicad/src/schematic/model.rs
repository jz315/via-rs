use std::collections::{BTreeMap, BTreeSet};

use via_core::{Board, Part};

use super::util::{natural_pin_key, sanitize_symbol};
use super::{PIN_SPACING, PIN_X};

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SymbolPin {
    pub(super) logical_pin: String,
    pub(super) number: String,
    pub(super) name: String,
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) rotation: i32,
}

#[derive(Debug, Clone)]
pub(super) struct SymbolTemplate {
    pub(super) name: String,
    pub(super) pins: Vec<SymbolPin>,
    pub(super) half_height: f64,
}

#[derive(Debug, Clone)]
pub(super) struct PlacedModule {
    pub(super) refdes: String,
    pub(super) value: String,
    pub(super) footprint: Option<String>,
    pub(super) symbol_name: String,
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) pins: Vec<SymbolPin>,
    pub(super) half_height: f64,
}

pub(super) fn symbol_templates(board: &Board) -> BTreeMap<String, SymbolTemplate> {
    let mut templates = BTreeMap::new();

    for module in board.modules() {
        let name = symbol_name(module);
        templates
            .entry(name.clone())
            .or_insert_with(|| build_symbol_template(&name, module));
    }

    templates
}

pub(super) fn place_modules(
    board: &Board,
    templates: &BTreeMap<String, SymbolTemplate>,
) -> Vec<PlacedModule> {
    let mut placed = Vec::new();
    let start_x = grid(55.0);
    let column_step = grid(67.0);
    let row_gap = grid(16.0);
    let mut cursor_x = start_x;
    let mut cursor_y = grid(55.0);
    let mut row_height: f64 = 0.0;
    let max_x = 250.0;

    for module in board.modules() {
        let symbol_name = symbol_name(module);
        let template = templates
            .get(&symbol_name)
            .expect("template exists for every module");
        let height = grid_mm(template.half_height * 2.0 + 25.4);

        if cursor_x > max_x {
            cursor_x = start_x;
            cursor_y = grid_mm(cursor_y + row_height + row_gap);
            row_height = 0.0;
        }

        placed.push(PlacedModule {
            refdes: module.refdes().to_owned(),
            value: module.value().to_owned(),
            footprint: module.footprint_name().map(str::to_owned),
            symbol_name,
            x: cursor_x,
            y: cursor_y,
            pins: template.pins.clone(),
            half_height: template.half_height,
        });

        cursor_x = grid_mm(cursor_x + column_step);
        row_height = row_height.max(height);
    }

    placed
}

fn grid(units: f64) -> f64 {
    units * 1.27
}

fn grid_mm(value: f64) -> f64 {
    (value / 1.27).round() * 1.27
}

pub(super) fn pin_net_map(board: &Board) -> BTreeMap<(String, String), String> {
    let mut pin_nets = BTreeMap::new();
    for net in board.nets() {
        for pin_ref in net.connections() {
            pin_nets.insert(
                (pin_ref.module.clone(), pin_ref.pin.clone()),
                net.name().to_owned(),
            );
        }
    }
    pin_nets
}

fn build_symbol_template(name: &str, module: &Part) -> SymbolTemplate {
    let mut pins = physical_symbol_pins(module);
    let left_count = pins.len().div_ceil(2);
    let right_count = pins.len() - left_count;
    let rows = left_count.max(right_count).max(1);
    let top_y = (rows.saturating_sub(1)) as f64 * PIN_SPACING / 2.0;

    for (idx, pin) in pins.iter_mut().enumerate() {
        if idx < left_count {
            pin.x = -PIN_X;
            pin.y = top_y - idx as f64 * PIN_SPACING;
            pin.rotation = 0;
        } else {
            pin.x = PIN_X;
            pin.y = top_y - (idx - left_count) as f64 * PIN_SPACING;
            pin.rotation = 180;
        }
    }

    SymbolTemplate {
        name: name.to_owned(),
        pins,
        half_height: top_y + PIN_SPACING,
    }
}

fn physical_symbol_pins(module: &Part) -> Vec<SymbolPin> {
    let mut pins = Vec::new();
    let mut seen_numbers = BTreeSet::new();

    for logical_pin in module.pins_iter() {
        for number in module.pads_for_pin(logical_pin) {
            if seen_numbers.insert(number.clone()) {
                pins.push(SymbolPin {
                    logical_pin: logical_pin.clone(),
                    name: logical_pin.clone(),
                    number,
                    x: 0.0,
                    y: 0.0,
                    rotation: 0,
                });
            }
        }
    }

    pins.sort_by_key(|pin| natural_pin_key(&pin.number));
    pins
}

pub(super) fn symbol_name(module: &Part) -> String {
    module
        .footprint_name()
        .map(sanitize_symbol)
        .unwrap_or_else(|| sanitize_symbol(module.value()))
}
