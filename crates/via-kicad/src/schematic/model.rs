use std::collections::{BTreeMap, BTreeSet};

use via_core::{Board, SymbolSide, model::Part};

use super::util::{natural_pin_key, sanitize_symbol};
use super::{PIN_SPACING, PIN_X};

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SymbolPin {
    pub(super) logical_pin: String,
    pub(super) number: String,
    pub(super) name: String,
    pub(super) side: SymbolSide,
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
    let mut pins = match module.symbol_spec() {
        Some(_) => explicit_symbol_pins(module),
        None => auto_symbol_pins(module),
    };
    layout_symbol_pins(&mut pins);

    let rows = pins
        .iter()
        .filter(|pin| matches!(pin.side, SymbolSide::Left | SymbolSide::Right))
        .fold((0usize, 0usize), |(left, right), pin| match pin.side {
            SymbolSide::Left => (left + 1, right),
            SymbolSide::Right => (left, right + 1),
            _ => (left, right),
        });
    let half_height = rows.0.max(rows.1).max(1) as f64 * PIN_SPACING / 2.0 + PIN_SPACING / 2.0;

    SymbolTemplate {
        name: name.to_owned(),
        pins,
        half_height,
    }
}

fn layout_symbol_pins(pins: &mut [SymbolPin]) {
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
    let top_y = (side_rows.saturating_sub(1)) as f64 * PIN_SPACING / 2.0;
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
                pin.y = top_y - left_idx as f64 * PIN_SPACING;
                pin.rotation = 0;
                left_idx += 1;
            }
            SymbolSide::Right => {
                pin.x = PIN_X;
                pin.y = top_y - right_idx as f64 * PIN_SPACING;
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

fn centered_axis_position(index: usize, count: usize) -> f64 {
    let center = (count.saturating_sub(1)) as f64 / 2.0;
    (index as f64 - center) * PIN_SPACING
}

fn auto_symbol_pins(module: &Part) -> Vec<SymbolPin> {
    let mut pins = physical_symbol_pins(module, SymbolSide::Left);
    let left_count = pins.len().div_ceil(2);

    for (idx, pin) in pins.iter_mut().enumerate() {
        if idx < left_count {
            pin.side = SymbolSide::Left;
        } else {
            pin.side = SymbolSide::Right;
        }
    }

    pins
}

fn explicit_symbol_pins(module: &Part) -> Vec<SymbolPin> {
    let Some(spec) = module.symbol_spec() else {
        return auto_symbol_pins(module);
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
            push_physical_symbol_pins(module, logical_pin, side, &mut seen_numbers, &mut pins);
        }
    }

    for logical_pin in module.pins_iter() {
        if !specified_logical_pins.contains(logical_pin) {
            push_physical_symbol_pins(
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

fn physical_symbol_pins(module: &Part, side: SymbolSide) -> Vec<SymbolPin> {
    let mut pins = Vec::new();
    let mut seen_numbers = BTreeSet::new();

    for logical_pin in module.pins_iter() {
        push_physical_symbol_pins(module, logical_pin, side, &mut seen_numbers, &mut pins);
    }

    pins.sort_by_key(|pin| natural_pin_key(&pin.number));
    pins
}

fn push_physical_symbol_pins(
    module: &Part,
    logical_pin: &str,
    side: SymbolSide,
    seen_numbers: &mut BTreeSet<String>,
    pins: &mut Vec<SymbolPin>,
) {
    let mut numbers = module
        .pads_for_pin(logical_pin)
        .into_iter()
        .collect::<Vec<_>>();
    numbers.sort_by_key(|number| natural_pin_key(number));
    for number in numbers {
        if seen_numbers.insert(number.clone()) {
            pins.push(SymbolPin {
                logical_pin: logical_pin.to_owned(),
                name: logical_pin.to_owned(),
                number,
                side,
                x: 0.0,
                y: 0.0,
                rotation: 0,
            });
        }
    }
}

pub(super) fn symbol_name(module: &Part) -> String {
    let base = module
        .footprint_name()
        .map(sanitize_symbol)
        .unwrap_or_else(|| sanitize_symbol(module.value()));
    format!("{}_{}", base, sanitize_symbol(module.refdes()))
}
