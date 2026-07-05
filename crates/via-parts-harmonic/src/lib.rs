mod connectors;
mod drivers;
mod esp32;
mod footprints;
mod passives;
mod power;

pub use connectors::{
    Header2, TerminalBlock5, Xh2p54Motor4, pin_header_1x02, terminal_block_1x05, xh2p54_motor4,
};
pub use drivers::{SilentStepStickTmc2209V20, silentstepstick_tmc2209_v20};
pub use esp32::{Esp32S3N16R8, esp32_s3_n16r8};
pub use footprints::{generated_footprint_pads, generated_footprints, write_generated_footprints};
pub use passives::{
    Capacitor2, Resistor2, capacitor_0603, capacitor_0805, polarized_capacitor_radial_verify,
    resistor_0603, resistor_0805,
};
pub use power::{Dc005BarrelJack, Mp1584BuckAdapter, dc005_barrel_jack, mp1584_buck_adapter};

#[cfg(test)]
mod tests {
    use super::*;
    use via_core::BoardSpec;

    #[test]
    fn typed_parts_construct_and_expose_pins() {
        let mut board = BoardSpec::new("parts_smoke");
        let esp32 = board.add(esp32_s3_n16r8("U1")).unwrap();
        let tmc = board.add(silentstepstick_tmc2209_v20("U2")).unwrap();

        board
            .net("STEP")
            .logic("3V3")
            .connect_all([esp32.gpio4(), tmc.step()]);

        board.build().unwrap();
    }
}
