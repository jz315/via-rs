use std::path::Path;
use std::{error, fmt};

pub mod generators;

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
        esp32_s3_n16r8_devboard_socket, mp1584_4wire_adapter, silentstepstick_tmc2209_v20_socket,
        terminal_block_1x, tht_header_1x, tht_header_2x, xh_vertical_1x,
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
            (terminal_block_1x("TerminalBlock_1x05_P5.08", 5).build(), 5),
            (
                xh_vertical_1x("XH2p54_1x04_Vertical_THT_VERIFY", 4).build(),
                4,
            ),
            (mp1584_4wire_adapter(), 4),
            (silentstepstick_tmc2209_v20_socket(), 16),
            (esp32_s3_n16r8_devboard_socket(), 44),
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
}
