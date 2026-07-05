const MM_TO_MIL: f64 = 39.370_078_740_157_48;

pub(crate) fn opt_i32(value: Option<i32>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_owned())
}

pub(crate) fn lceda_mm(value: f64) -> String {
    lceda_number(value * MM_TO_MIL)
}

pub(crate) fn lceda_number(value: f64) -> String {
    let value = if value.abs() < 0.000_000_1 {
        0.0
    } else {
        value
    };
    let text = format!("{value:.4}");
    text.trim_end_matches('0').trim_end_matches('.').to_owned()
}
