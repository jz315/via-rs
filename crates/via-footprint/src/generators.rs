mod common;
mod connectors;
mod headers;
mod mechanical;
mod passives;
mod semiconductors;

use crate::GeneratedFootprint;

pub use connectors::{
    TerminalBlock1x, XhVertical1x, jst_ph_1x02_p2p00_vertical_verify,
    jst_ph_1x03_p2p00_vertical_verify, jst_ph_1x04_p2p00_vertical_verify,
    jst_xh_1x02_p2p54_vertical_verify, jst_xh_1x03_p2p54_vertical_verify,
    jst_xh_1x04_p2p54_vertical_verify, jst_xh_1x05_p2p54_vertical_verify,
    jst_xh_1x06_p2p54_vertical_verify, terminal_block_1x, terminal_block_1x02_p5p08,
    terminal_block_1x03_p5p08, terminal_block_1x04_p5p08, terminal_block_1x05_p5p08,
    terminal_block_1x06_p5p08, xh_vertical_1x,
};
pub use headers::{
    RightRowOrder, ThtHeader1x, ThtHeader2x, pin_header_1x02_p2p54, pin_header_1x03_p2p54,
    pin_header_1x04_p2p54, pin_header_1x05_p2p54, pin_header_1x06_p2p54, pin_header_1x08_p2p54,
    pin_header_1x10_p2p54, pin_header_1x20_p2p54, pin_header_2x03_p2p54, pin_header_2x05_p2p54,
    pin_header_2x10_p2p54, pin_header_2x20_p2p54, pin_socket_2x08_p2p54_row12p70, tht_header_1x,
    tht_header_2x,
};
pub use mechanical::{
    fiducial_1p0, fiducial_round, mounting_hole_m2_np, mounting_hole_m2_pth, mounting_hole_m3_np,
    mounting_hole_m3_pth, mounting_hole_m25_np, mounting_hole_m25_pth, mounting_hole_np,
    mounting_hole_plated, testpad_1p0, testpad_1p5, testpad_2p0, testpad_round,
};
pub use passives::{
    capacitor_0402, capacitor_0603, capacitor_0805, capacitor_1206,
    polarized_capacitor_radial_d5p0_p2p00_verify, polarized_capacitor_radial_d6p3_p2p50_verify,
    polarized_capacitor_radial_d8p0_p3p50_verify, polarized_capacitor_radial_d10p0_p5p00_verify,
    resistor_0402, resistor_0603, resistor_0805, resistor_1206,
};
pub use semiconductors::{
    led_0603, led_0805, sod123, sod323, soic8, soic14, soic16, sot23_3, sot23_5, sot23_6, sot223,
    tssop16, tssop20,
};

pub fn common_footprints() -> Vec<GeneratedFootprint> {
    vec![
        crate::fp::r0402(),
        crate::fp::r0603(),
        crate::fp::r0805(),
        crate::fp::r1206(),
        crate::fp::c0402(),
        crate::fp::c0603(),
        crate::fp::c0805(),
        crate::fp::c1206(),
        crate::fp::cp_d5_p2(),
        crate::fp::cp_d63_p25(),
        crate::fp::cp_d8_p35(),
        crate::fp::cp_d10_p5(),
        crate::fp::pin_1x02(),
        crate::fp::pin_1x03(),
        crate::fp::pin_1x04(),
        crate::fp::pin_1x05(),
        crate::fp::pin_1x06(),
        crate::fp::pin_1x08(),
        crate::fp::pin_1x10(),
        crate::fp::pin_1x20(),
        crate::fp::pin_2x03(),
        crate::fp::pin_2x05(),
        crate::fp::pin_2x10(),
        crate::fp::pin_2x20(),
        crate::fp::socket_2x08_r12p7(),
        crate::fp::tb_2p(),
        crate::fp::tb_3p(),
        crate::fp::tb_4p(),
        crate::fp::tb_5p(),
        crate::fp::tb_6p(),
        crate::fp::xh_2p(),
        crate::fp::xh_3p(),
        crate::fp::xh_4p(),
        crate::fp::xh_5p(),
        crate::fp::xh_6p(),
        crate::fp::ph_2p(),
        crate::fp::ph_3p(),
        crate::fp::ph_4p(),
        crate::fp::testpad_1p0(),
        crate::fp::testpad_1p5(),
        crate::fp::testpad_2p0(),
        crate::fp::fiducial_1p0(),
        crate::fp::mh_m2_np(),
        crate::fp::mh_m25_np(),
        crate::fp::mh_m3_np(),
        crate::fp::mh_m2_pth(),
        crate::fp::mh_m25_pth(),
        crate::fp::mh_m3_pth(),
        crate::fp::led_0603(),
        crate::fp::led_0805(),
        crate::fp::sod123(),
        crate::fp::sod323(),
        crate::fp::sot23_3(),
        crate::fp::sot23_5(),
        crate::fp::sot23_6(),
        crate::fp::sot223(),
        crate::fp::soic8(),
        crate::fp::soic14(),
        crate::fp::soic16(),
        crate::fp::tssop16(),
        crate::fp::tssop20(),
    ]
}
