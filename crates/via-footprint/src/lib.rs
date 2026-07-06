use std::path::Path;
use std::{error, fmt};

pub mod fp;
pub mod generators;

use via_core::{Footprint, FootprintPads};
pub use via_footprint_ir::{FootprintValidationError, FootprintWriteError};

#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedFootprint {
    ir: via_footprint_ir::FootprintIr,
    metadata: FootprintMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FootprintMetadata {
    generator_name: String,
    source_kind: FootprintSourceKind,
    verification_status: FootprintVerificationStatus,
    notes: String,
}

impl FootprintMetadata {
    pub fn generated(generator_name: impl Into<String>) -> Self {
        Self {
            generator_name: generator_name.into(),
            source_kind: FootprintSourceKind::Generated,
            verification_status: FootprintVerificationStatus::VerifyRequired,
            notes:
                "Generated dimensions; verify against the exact purchased part before fabrication"
                    .to_owned(),
        }
    }

    pub fn custom_ir() -> Self {
        Self {
            generator_name: "custom-ir".to_owned(),
            source_kind: FootprintSourceKind::CustomIr,
            verification_status: FootprintVerificationStatus::VerifyRequired,
            notes: "Constructed from low-level FootprintIr; verify before fabrication".to_owned(),
        }
    }

    pub fn with_verification_status(mut self, status: FootprintVerificationStatus) -> Self {
        self.verification_status = status;
        self
    }

    pub fn notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = notes.into();
        self
    }

    pub fn generator_name(&self) -> &str {
        &self.generator_name
    }

    pub fn source_kind(&self) -> FootprintSourceKind {
        self.source_kind
    }

    pub fn verification_status(&self) -> FootprintVerificationStatus {
        self.verification_status
    }

    pub fn notes_text(&self) -> &str {
        &self.notes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FootprintSourceKind {
    Generated,
    MeasuredManual,
    ExternalReference,
    CustomIr,
}

impl FootprintSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            FootprintSourceKind::Generated => "generated",
            FootprintSourceKind::MeasuredManual => "measured-manual",
            FootprintSourceKind::ExternalReference => "external-reference",
            FootprintSourceKind::CustomIr => "custom-ir",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FootprintVerificationStatus {
    Verified,
    VerifyRequired,
    Unverified,
}

impl FootprintVerificationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            FootprintVerificationStatus::Verified => "verified",
            FootprintVerificationStatus::VerifyRequired => "verify-required",
            FootprintVerificationStatus::Unverified => "unverified",
        }
    }

    pub fn requires_verify(self) -> bool {
        matches!(
            self,
            FootprintVerificationStatus::VerifyRequired | FootprintVerificationStatus::Unverified
        )
    }
}

impl GeneratedFootprint {
    pub fn new(ir: via_footprint_ir::FootprintIr, metadata: FootprintMetadata) -> Self {
        let mut footprint = Self { ir, metadata };
        footprint.apply_metadata_properties();
        footprint
    }

    pub fn name(&self) -> &str {
        self.ir.name()
    }

    pub fn metadata(&self) -> &FootprintMetadata {
        &self.metadata
    }

    pub fn write_kicad_mod(&self, path: impl AsRef<Path>) -> Result<(), FootprintWriteError> {
        self.try_write_kicad_mod(path)
    }

    pub fn try_write_kicad_mod(&self, path: impl AsRef<Path>) -> Result<(), FootprintWriteError> {
        self.ir.write_kicad_mod(path)
    }

    pub fn validate(&self) -> Result<(), FootprintValidationError> {
        self.ir.validate()
    }

    pub fn into_ir(self) -> via_footprint_ir::FootprintIr {
        self.ir
    }

    pub fn into_pads(self) -> FootprintPads {
        FootprintPads::from_ir(self.into_ir())
    }

    pub fn into_footprint(self) -> Footprint {
        Footprint::pads(self.into_pads())
    }

    fn apply_metadata_properties(&mut self) {
        self.ir
            .set_property("VIA_GENERATOR", self.metadata.generator_name())
            .set_property("VIA_SOURCE", self.metadata.source_kind().as_str())
            .set_property(
                "VIA_VERIFY",
                if self.metadata.verification_status().requires_verify() {
                    "true"
                } else {
                    "false"
                },
            )
            .set_property(
                "VIA_VERIFICATION_STATUS",
                self.metadata.verification_status().as_str(),
            )
            .set_property("VIA_NOTES", self.metadata.notes_text());
    }
}

impl From<via_footprint_ir::FootprintIr> for GeneratedFootprint {
    fn from(ir: via_footprint_ir::FootprintIr) -> Self {
        Self::new(ir, FootprintMetadata::custom_ir())
    }
}

impl From<GeneratedFootprint> for Footprint {
    fn from(footprint: GeneratedFootprint) -> Self {
        footprint.into_footprint()
    }
}

#[derive(Debug, Default, Clone)]
pub struct FootprintCatalog {
    names: std::collections::BTreeSet<String>,
    footprints: Vec<GeneratedFootprint>,
}

impl FootprintCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(
        &mut self,
        footprint: GeneratedFootprint,
    ) -> Result<&mut Self, FootprintCatalogError> {
        let name = footprint.name().to_owned();
        if !self.names.insert(name.clone()) {
            return Err(FootprintCatalogError::duplicate(name));
        }
        self.footprints.push(footprint);
        Ok(self)
    }

    pub fn extend<I>(&mut self, footprints: I) -> Result<&mut Self, FootprintCatalogError>
    where
        I: IntoIterator<Item = GeneratedFootprint>,
    {
        for footprint in footprints {
            self.add(footprint)?;
        }
        Ok(self)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.names.contains(name)
    }

    pub fn len(&self) -> usize {
        self.footprints.len()
    }

    pub fn is_empty(&self) -> bool {
        self.footprints.is_empty()
    }

    pub fn into_vec(self) -> Vec<GeneratedFootprint> {
        self.footprints
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FootprintCatalogError {
    message: String,
}

impl FootprintCatalogError {
    fn duplicate(name: impl Into<String>) -> Self {
        Self {
            message: format!("duplicate footprint {}", name.into()),
        }
    }
}

impl fmt::Display for FootprintCatalogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl error::Error for FootprintCatalogError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FootprintBuildError {
    message: String,
}

impl FootprintBuildError {
    pub fn invalid_parameter(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for FootprintBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl error::Error for FootprintBuildError {}

#[cfg(test)]
mod tests {
    use super::*;
    use generators::{
        common_footprints, fiducial_1p0, led_0805, mounting_hole_m3_np,
        polarized_capacitor_radial_d5p0_p2p00_verify, polarized_capacitor_radial_d8p0_p3p50_verify,
        polarized_capacitor_radial_d10p0_p5p00_verify, resistor_1206, soic8, terminal_block_1x,
        tht_header_1x, tht_header_2x, xh_vertical_1x,
    };

    #[test]
    fn generates_tht_header_pad_numbers() {
        let footprint = tht_header_1x("PinHeader_1x04_P2.54_Via", 4)
            .pitch(2.54)
            .drill(1.0)
            .pad_diameter(1.7)
            .build();

        let ir = footprint.clone().into_ir();
        assert_eq!(ir.pads().len(), 4);
        assert_eq!(ir.pads()[0].number, "1");
        assert_eq!(ir.pads()[3].at.y, 7.62);
        assert_eq!(footprint.metadata().generator_name(), "tht_header_1x");
        footprint.validate().unwrap();
    }

    #[test]
    fn try_build_reports_invalid_pin_counts() {
        let err = tht_header_1x("Bad", 0).try_build().unwrap_err();
        assert!(err.to_string().contains("at least one pin"));
    }

    #[test]
    fn metadata_is_written_to_kicad_properties() {
        let footprint = tht_header_1x("PinHeader_1x02_P2.54", 2).build();
        let text =
            via_footprint_ir::kicad::try_render_kicad_mod(&footprint.clone().into_ir()).unwrap();
        assert!(text.contains("(property \"VIA_GENERATOR\" \"tht_header_1x\""));
        assert!(text.contains("(property \"VIA_SOURCE\" \"generated\""));
        assert!(text.contains("(property \"VIA_VERIFY\" \"true\""));
    }

    #[test]
    fn public_api_does_not_reexport_low_level_ir_types() {
        let api =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs")).unwrap();
        for type_name in ["Pad", "Point", "Size", "GraphicText", "FootprintIr"] {
            assert!(
                !api.contains(&format!("pub use via_footprint_ir::{type_name}")),
                "{type_name} should stay in via-footprint-ir"
            );
        }
    }

    #[test]
    fn generates_common_production_footprints() {
        let footprints = [
            (
                tht_header_2x("Socket_2x08", 8).row_spacing(12.7).build(),
                16,
            ),
            (terminal_block_1x("TB_1x05_P5.08", 5).build(), 5),
            (xh_vertical_1x("XH_1x04_P2.54_VERIFY", 4).build(), 4),
            (fiducial_1p0(), 1),
            (mounting_hole_m3_np(), 1),
            (led_0805(), 2),
            (soic8(), 8),
        ];

        for (footprint, pad_count) in footprints {
            footprint.validate().unwrap();
            let ir = footprint.clone().into_ir();
            assert_eq!(ir.pads().len(), pad_count, "{}", footprint.name());
            assert!(
                ir.lines().iter().any(|line| line.layer == "F.CrtYd"),
                "{} should include courtyard geometry",
                footprint.name()
            );
            assert!(
                ir.texts()
                    .iter()
                    .any(|text| matches!(text.kind, via_footprint_ir::TextKind::Reference)),
                "{} should include a reference text",
                footprint.name()
            );
            assert!(
                ir.texts()
                    .iter()
                    .any(|text| matches!(text.kind, via_footprint_ir::TextKind::Value)),
                "{} should include a value text",
                footprint.name()
            );
        }
    }

    #[test]
    fn common_footprint_pack_has_stable_names_and_pad_counts() {
        let footprints = common_footprints();
        let mut names = std::collections::BTreeSet::new();

        for footprint in &footprints {
            assert!(
                names.insert(footprint.name().to_owned()),
                "{}",
                footprint.name()
            );
            footprint.validate().unwrap();
        }

        for expected in [
            ("R_0402", 2),
            ("R_0603", 2),
            ("R_0805", 2),
            ("R_1206", 2),
            ("C_0402", 2),
            ("C_0603", 2),
            ("C_0805", 2),
            ("C_1206", 2),
            ("CP_D5.0_P2.0_VERIFY", 2),
            ("CP_D6.3_P2.5_VERIFY", 2),
            ("CP_D8.0_P3.5_VERIFY", 2),
            ("CP_D10.0_P5.0_VERIFY", 2),
            ("Pin_1x02_P2.54", 2),
            ("Pin_1x03_P2.54", 3),
            ("Pin_1x04_P2.54", 4),
            ("Pin_1x05_P2.54", 5),
            ("Pin_1x06_P2.54", 6),
            ("Pin_1x08_P2.54", 8),
            ("Pin_1x10_P2.54", 10),
            ("Pin_1x20_P2.54", 20),
            ("Pin_2x03_P2.54", 6),
            ("Pin_2x05_P2.54", 10),
            ("Pin_2x10_P2.54", 20),
            ("Pin_2x20_P2.54", 40),
            ("Socket_2x08_R12.7", 16),
            ("TB_1x02_P5.08", 2),
            ("TB_1x03_P5.08", 3),
            ("TB_1x04_P5.08", 4),
            ("TB_1x05_P5.08", 5),
            ("TB_1x06_P5.08", 6),
            ("XH_1x02_P2.54_VERIFY", 2),
            ("XH_1x03_P2.54_VERIFY", 3),
            ("XH_1x04_P2.54_VERIFY", 4),
            ("XH_1x05_P2.54_VERIFY", 5),
            ("XH_1x06_P2.54_VERIFY", 6),
            ("PH_1x02_P2.00_VERIFY", 2),
            ("PH_1x03_P2.00_VERIFY", 3),
            ("PH_1x04_P2.00_VERIFY", 4),
            ("TestPad_D1.0", 1),
            ("TestPad_D1.5", 1),
            ("TestPad_D2.0", 1),
            ("Fiducial_D1.0", 1),
            ("MH_M2_NPTH_D2.2", 1),
            ("MH_M2.5_NPTH_D2.7", 1),
            ("MH_M3_NPTH_D3.2", 1),
            ("MH_M2_PTH_D2.2_P4.2", 1),
            ("MH_M2.5_PTH_D2.7_P4.8", 1),
            ("MH_M3_PTH_D3.2_P5.5", 1),
            ("LED_0603", 2),
            ("LED_0805", 2),
            ("SOD-123", 2),
            ("SOD-323", 2),
            ("SOT-23-3", 3),
            ("SOT-23-5", 5),
            ("SOT-23-6", 6),
            ("SOT-223", 4),
            ("SOIC-8", 8),
            ("SOIC-14", 14),
            ("SOIC-16", 16),
            ("TSSOP-16", 16),
            ("TSSOP-20", 20),
        ] {
            let footprint = footprints
                .iter()
                .find(|footprint| footprint.name() == expected.0)
                .unwrap_or_else(|| panic!("missing {}", expected.0));
            assert_eq!(
                footprint.clone().into_ir().pads().len(),
                expected.1,
                "{}",
                expected.0
            );
        }
    }

    #[test]
    fn common_passives_encode_expected_geometry() {
        let r1206 = resistor_1206("R_1206_3216Metric").into_ir();
        assert_eq!(r1206.pads()[0].at.x, -1.5);
        assert_eq!(r1206.pads()[1].at.x, 1.5);
        assert_eq!(r1206.pads()[0].size.x, 1.25);
        assert_eq!(r1206.pads()[0].size.y, 1.75);

        for (footprint, pitch) in [
            (polarized_capacitor_radial_d5p0_p2p00_verify("CP_D5"), 2.0),
            (polarized_capacitor_radial_d8p0_p3p50_verify("CP_D8"), 3.5),
            (polarized_capacitor_radial_d10p0_p5p00_verify("CP_D10"), 5.0),
        ] {
            let ir = footprint.into_ir();
            assert_eq!(ir.pads()[0].at.x, -pitch / 2.0);
            assert_eq!(ir.pads()[1].at.x, pitch / 2.0);
            assert!(ir.pads()[0].drill.unwrap().is_round());
        }
    }

    #[test]
    fn gullwing_and_sot_pads_use_horizontal_land_pattern() {
        for footprint in [soic8(), generators::tssop20(), generators::sot23_3()] {
            let ir = footprint.into_ir();
            for pad in ir.pads() {
                assert!(
                    pad.size.x > pad.size.y,
                    "{} pad {} should be longer on X than Y",
                    ir.name(),
                    pad.number
                );
            }
        }

        let tssop20 = generators::tssop20().into_ir();
        let pad1 = tssop20.pads().iter().find(|pad| pad.number == "1").unwrap();
        let pad2 = tssop20.pads().iter().find(|pad| pad.number == "2").unwrap();
        let pitch = (pad2.at.y - pad1.at.y).abs();
        assert!(
            pad1.size.y < pitch,
            "TSSOP-20 pad height must be smaller than pin pitch"
        );
    }

    #[test]
    fn footprint_catalog_rejects_duplicate_names() {
        let mut catalog = FootprintCatalog::new();
        catalog
            .add(tht_header_1x("Pin_1x02_P2.54", 2).build())
            .unwrap();

        let err = catalog
            .add(tht_header_1x("Pin_1x02_P2.54", 2).build())
            .unwrap_err();

        assert!(err.to_string().contains("duplicate footprint"));
        assert!(catalog.contains("Pin_1x02_P2.54"));
        assert_eq!(catalog.len(), 1);
    }
}
