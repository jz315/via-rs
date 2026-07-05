use via_core::Part;

pub(crate) fn footprint_name(module: &Part) -> &str {
    module.footprint_name().unwrap_or("via_unassigned")
}

pub(crate) fn board_uuid(name: &str) -> String {
    stable_uuid(&format!("board:{name}"))
}

pub(crate) fn schematic_uuid(name: &str) -> String {
    stable_uuid(&format!("sch:{name}"))
}

pub(crate) fn schematic_page_uuid(name: &str) -> String {
    stable_uuid(&format!("sch-page:{name}"))
}

pub(crate) fn pcb_uuid(name: &str) -> String {
    stable_uuid(&format!("pcb:{name}"))
}

pub(crate) fn footprint_uuid(name: &str) -> String {
    stable_uuid(&format!("footprint:{name}"))
}

pub(crate) fn sheet_symbol_uuid(name: &str) -> String {
    format!("{}_sheet", schematic_page_uuid(name))
}

pub(crate) fn symbol_uuid(module: &Part) -> String {
    stable_uuid(&format!("symbol:{}:{}", module.refdes(), module.value()))
}

pub(crate) fn device_uuid(module: &Part) -> String {
    stable_uuid(&format!("device:{}:{}", module.refdes(), module.value()))
}

pub(crate) fn sheet_device_uuid(name: &str) -> String {
    stable_uuid(&format!("sheet-device:{name}"))
}

pub(crate) fn component_id(refdes: &str) -> String {
    format!("c_{}", sanitize_id(refdes))
}

pub(crate) fn pcb_component_id(refdes: &str) -> String {
    format!("pcb_{}", sanitize_id(refdes))
}

pub(crate) fn footprint_pad_id(pad_name: &str) -> String {
    format!("pad_{}", sanitize_id(pad_name))
}

pub(crate) fn sheet_part_id() -> &'static str {
    "pid8a0e77bacb214e"
}

pub(crate) fn sanitize_id(value: &str) -> String {
    value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect()
}

pub(crate) fn json_escape(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_control() => escaped.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => escaped.push(ch),
        }
    }
    escaped
}

pub(crate) fn stable_uuid(seed: &str) -> String {
    let a = fnv64(seed.as_bytes(), 0xcbf2_9ce4_8422_2325);
    let b = fnv64(seed.as_bytes(), 0x8422_2325_cbf2_9ce4);
    format!("{a:016x}{b:016x}")
}

fn fnv64(bytes: &[u8], offset: u64) -> u64 {
    let mut hash = offset;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    hash
}
