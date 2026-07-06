use crate::GeneratedFootprint;
use crate::generators::{
    capacitor_0402, capacitor_0603, capacitor_0805, capacitor_1206,
    fiducial_1p0 as generator_fiducial_1p0, jst_ph_1x02_p2p00_vertical_verify,
    jst_ph_1x03_p2p00_vertical_verify, jst_ph_1x04_p2p00_vertical_verify,
    jst_xh_1x02_p2p54_vertical_verify, jst_xh_1x03_p2p54_vertical_verify,
    jst_xh_1x04_p2p54_vertical_verify, jst_xh_1x05_p2p54_vertical_verify,
    jst_xh_1x06_p2p54_vertical_verify, led_0603 as generator_led_0603,
    led_0805 as generator_led_0805, mounting_hole_m2_np, mounting_hole_m2_pth, mounting_hole_m3_np,
    mounting_hole_m3_pth, mounting_hole_m25_np, mounting_hole_m25_pth, pin_header_1x02_p2p54,
    pin_header_1x03_p2p54, pin_header_1x04_p2p54, pin_header_1x05_p2p54, pin_header_1x06_p2p54,
    pin_header_1x08_p2p54, pin_header_1x10_p2p54, pin_header_1x20_p2p54, pin_header_2x03_p2p54,
    pin_header_2x05_p2p54, pin_header_2x10_p2p54, pin_header_2x20_p2p54,
    pin_socket_2x08_p2p54_row12p70, polarized_capacitor_radial_d5p0_p2p00_verify,
    polarized_capacitor_radial_d6p3_p2p50_verify, polarized_capacitor_radial_d8p0_p3p50_verify,
    polarized_capacitor_radial_d10p0_p5p00_verify, resistor_0402, resistor_0603, resistor_0805,
    resistor_1206, sod123 as generator_sod123, sod323 as generator_sod323,
    soic8 as generator_soic8, soic14 as generator_soic14, soic16 as generator_soic16,
    sot23_3 as generator_sot23_3, sot23_5 as generator_sot23_5, sot23_6 as generator_sot23_6,
    sot223 as generator_sot223, terminal_block_1x02_p5p08, terminal_block_1x03_p5p08,
    terminal_block_1x04_p5p08, terminal_block_1x05_p5p08, terminal_block_1x06_p5p08,
    testpad_1p0 as generator_testpad_1p0, testpad_1p5 as generator_testpad_1p5,
    testpad_2p0 as generator_testpad_2p0, tssop16 as generator_tssop16,
    tssop20 as generator_tssop20,
};

pub fn r0402() -> GeneratedFootprint {
    resistor_0402("R_0402")
}

pub fn r0603() -> GeneratedFootprint {
    resistor_0603("R_0603")
}

pub fn r0805() -> GeneratedFootprint {
    resistor_0805("R_0805")
}

pub fn r1206() -> GeneratedFootprint {
    resistor_1206("R_1206")
}

pub fn c0402() -> GeneratedFootprint {
    capacitor_0402("C_0402")
}

pub fn c0603() -> GeneratedFootprint {
    capacitor_0603("C_0603")
}

pub fn c0805() -> GeneratedFootprint {
    capacitor_0805("C_0805")
}

pub fn c1206() -> GeneratedFootprint {
    capacitor_1206("C_1206")
}

pub fn cp_d5_p2() -> GeneratedFootprint {
    polarized_capacitor_radial_d5p0_p2p00_verify("CP_D5.0_P2.0_VERIFY")
}

pub fn cp_d63_p25() -> GeneratedFootprint {
    polarized_capacitor_radial_d6p3_p2p50_verify("CP_D6.3_P2.5_VERIFY")
}

pub fn cp_d8_p35() -> GeneratedFootprint {
    polarized_capacitor_radial_d8p0_p3p50_verify("CP_D8.0_P3.5_VERIFY")
}

pub fn cp_d10_p5() -> GeneratedFootprint {
    polarized_capacitor_radial_d10p0_p5p00_verify("CP_D10.0_P5.0_VERIFY")
}

pub fn pin_1x02() -> GeneratedFootprint {
    pin_header_1x02_p2p54()
}

pub fn pin_1x03() -> GeneratedFootprint {
    pin_header_1x03_p2p54()
}

pub fn pin_1x04() -> GeneratedFootprint {
    pin_header_1x04_p2p54()
}

pub fn pin_1x05() -> GeneratedFootprint {
    pin_header_1x05_p2p54()
}

pub fn pin_1x06() -> GeneratedFootprint {
    pin_header_1x06_p2p54()
}

pub fn pin_1x08() -> GeneratedFootprint {
    pin_header_1x08_p2p54()
}

pub fn pin_1x10() -> GeneratedFootprint {
    pin_header_1x10_p2p54()
}

pub fn pin_1x20() -> GeneratedFootprint {
    pin_header_1x20_p2p54()
}

pub fn pin_2x03() -> GeneratedFootprint {
    pin_header_2x03_p2p54()
}

pub fn pin_2x05() -> GeneratedFootprint {
    pin_header_2x05_p2p54()
}

pub fn pin_2x10() -> GeneratedFootprint {
    pin_header_2x10_p2p54()
}

pub fn pin_2x20() -> GeneratedFootprint {
    pin_header_2x20_p2p54()
}

pub fn socket_2x08_r12p7() -> GeneratedFootprint {
    pin_socket_2x08_p2p54_row12p70()
}

pub fn tb_2p() -> GeneratedFootprint {
    terminal_block_1x02_p5p08()
}

pub fn tb_3p() -> GeneratedFootprint {
    terminal_block_1x03_p5p08()
}

pub fn tb_4p() -> GeneratedFootprint {
    terminal_block_1x04_p5p08()
}

pub fn tb_5p() -> GeneratedFootprint {
    terminal_block_1x05_p5p08()
}

pub fn tb_6p() -> GeneratedFootprint {
    terminal_block_1x06_p5p08()
}

pub fn xh_2p() -> GeneratedFootprint {
    jst_xh_1x02_p2p54_vertical_verify()
}

pub fn xh_3p() -> GeneratedFootprint {
    jst_xh_1x03_p2p54_vertical_verify()
}

pub fn xh_4p() -> GeneratedFootprint {
    jst_xh_1x04_p2p54_vertical_verify()
}

pub fn xh_5p() -> GeneratedFootprint {
    jst_xh_1x05_p2p54_vertical_verify()
}

pub fn xh_6p() -> GeneratedFootprint {
    jst_xh_1x06_p2p54_vertical_verify()
}

pub fn ph_2p() -> GeneratedFootprint {
    jst_ph_1x02_p2p00_vertical_verify()
}

pub fn ph_3p() -> GeneratedFootprint {
    jst_ph_1x03_p2p00_vertical_verify()
}

pub fn ph_4p() -> GeneratedFootprint {
    jst_ph_1x04_p2p00_vertical_verify()
}

pub fn testpad_1p0() -> GeneratedFootprint {
    generator_testpad_1p0()
}

pub fn testpad_1p5() -> GeneratedFootprint {
    generator_testpad_1p5()
}

pub fn testpad_2p0() -> GeneratedFootprint {
    generator_testpad_2p0()
}

pub fn fiducial_1p0() -> GeneratedFootprint {
    generator_fiducial_1p0()
}

pub fn mh_m2_np() -> GeneratedFootprint {
    mounting_hole_m2_np()
}

pub fn mh_m25_np() -> GeneratedFootprint {
    mounting_hole_m25_np()
}

pub fn mh_m3_np() -> GeneratedFootprint {
    mounting_hole_m3_np()
}

pub fn mh_m2_pth() -> GeneratedFootprint {
    mounting_hole_m2_pth()
}

pub fn mh_m25_pth() -> GeneratedFootprint {
    mounting_hole_m25_pth()
}

pub fn mh_m3_pth() -> GeneratedFootprint {
    mounting_hole_m3_pth()
}

pub fn led_0603() -> GeneratedFootprint {
    generator_led_0603()
}

pub fn led_0805() -> GeneratedFootprint {
    generator_led_0805()
}

pub fn sod123() -> GeneratedFootprint {
    generator_sod123()
}

pub fn sod323() -> GeneratedFootprint {
    generator_sod323()
}

pub fn sot23_3() -> GeneratedFootprint {
    generator_sot23_3()
}

pub fn sot23_5() -> GeneratedFootprint {
    generator_sot23_5()
}

pub fn sot23_6() -> GeneratedFootprint {
    generator_sot23_6()
}

pub fn sot223() -> GeneratedFootprint {
    generator_sot223()
}

pub fn soic8() -> GeneratedFootprint {
    generator_soic8()
}

pub fn soic14() -> GeneratedFootprint {
    generator_soic14()
}

pub fn soic16() -> GeneratedFootprint {
    generator_soic16()
}

pub fn tssop16() -> GeneratedFootprint {
    generator_tssop16()
}

pub fn tssop20() -> GeneratedFootprint {
    generator_tssop20()
}
