#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagnosticDefinition {
    pub code: &'static str,
    pub title: &'static str,
    pub explanation: &'static str,
    pub causes: &'static [&'static str],
    pub help: &'static [&'static str],
}

pub fn diagnostic_definition(code: &str) -> Option<&'static DiagnosticDefinition> {
    DIAGNOSTIC_DEFINITIONS
        .iter()
        .find(|definition| definition.code == code)
}

pub fn all_diagnostic_definitions() -> &'static [DiagnosticDefinition] {
    DIAGNOSTIC_DEFINITIONS
}

pub static DIAGNOSTIC_DEFINITIONS: &[DiagnosticDefinition] = &[
    DiagnosticDefinition {
        code: "diagnostic.unknown_code",
        title: "Unknown diagnostic code",
        explanation: "The requested diagnostic code is not known by this version of via.",
        causes: &[
            "The code was misspelled.",
            "The code belongs to a different via version.",
        ],
        help: &["Run `via explain --list` to see the diagnostic codes supported by this tool."],
    },
    DiagnosticDefinition {
        code: "part.duplicate_refdes",
        title: "Duplicate module reference designator",
        explanation: "Two modules in the same board use the same reference designator.",
        causes: &[
            "A part was added twice.",
            "A reusable module hard-coded a refdes already used by the board.",
        ],
        help: &["Give each module a unique refdes such as U1, U2, R1, or C1."],
    },
    DiagnosticDefinition {
        code: "part.no_footprint",
        title: "Part has no footprint",
        explanation: "The module can be checked schematically, but export cannot place it on a board without a footprint name.",
        causes: &[
            "The part definition omitted `.footprint(...)`.",
            "A reusable part helper did not attach a footprint.",
        ],
        help: &["Attach a footprint to the part, or use a part helper that includes one."],
    },
    DiagnosticDefinition {
        code: "footprint.asset_alias",
        title: "KiCad footprint asset alias is unsupported",
        explanation: "A local footprint name points at a different KiCad library footprint name. via currently requires the local name and KiCad asset name to match.",
        causes: &[
            "The footprint was renamed locally while still using an official KiCad asset.",
            "The KiCad asset metadata was copied from a different footprint.",
        ],
        help: &[
            "Use matching footprint names, or create a separate local footprint asset for the alias.",
        ],
    },
    DiagnosticDefinition {
        code: "pin_pad_map.missing_pad",
        title: "Pin maps to a missing footprint pad",
        explanation: "A logical pin is mapped to a pad number that is not present in the loaded footprint pad model.",
        causes: &[
            "The pin-to-pad map has a typo.",
            "The footprint changed but the part model was not updated.",
        ],
        help: &[
            "Fix the pad number in the part model or load the footprint that contains that pad.",
        ],
    },
    DiagnosticDefinition {
        code: "pin_pad_map.uncovered_footprint_pad",
        title: "Footprint pad is not covered by the part model",
        explanation: "The loaded footprint has electrical pads that no logical pin maps to, so via cannot reason about every copper connection.",
        causes: &[
            "The part model omitted a pin.",
            "A multi-pad thermal tab or mounting pad was not intentionally modeled.",
        ],
        help: &[
            "Map every electrical pad to a logical pin, or split mechanical-only pads out of the footprint model.",
        ],
    },
    DiagnosticDefinition {
        code: "part.unknown_footprint",
        title: "Part references an unknown footprint",
        explanation: "A module names a footprint that was not loaded into the board footprint catalog.",
        causes: &[
            "The footprint name is misspelled.",
            "The design did not add or import the required footprint pad model.",
        ],
        help: &[
            "Load the footprint pads for that footprint or change the part to use a known footprint name.",
        ],
    },
    DiagnosticDefinition {
        code: "part.unknown_mapped_pin",
        title: "Pin map references an unknown logical pin",
        explanation: "The part maps a pad for a logical pin that the part itself does not define.",
        causes: &[
            "The pin name in `.map_pin(...)` is misspelled.",
            "The part pin list was changed without updating the pin map.",
        ],
        help: &["Define the logical pin or update the pin map to use an existing pin."],
    },
    DiagnosticDefinition {
        code: "part.unknown_classified_pin",
        title: "Electrical class references an unknown logical pin",
        explanation: "The part assigns an electrical class to a pin that the part itself does not define.",
        causes: &[
            "A `.power_pin(...)`, `.ground_pin(...)`, or similar call uses the wrong pin name.",
            "The part pin list was changed without updating classifications.",
        ],
        help: &["Define the logical pin or update the classification to use an existing pin."],
    },
    DiagnosticDefinition {
        code: "symbol.unknown_pin",
        title: "Symbol references an unknown logical pin",
        explanation: "The schematic symbol layout contains a pin name that is not part of the module model.",
        causes: &[
            "The symbol pin list has a typo.",
            "The part pin names were changed after the symbol was written.",
        ],
        help: &["Update the symbol pins or add the missing logical pin to the part model."],
    },
    DiagnosticDefinition {
        code: "symbol.duplicate_pin",
        title: "Symbol places a logical pin more than once",
        explanation: "The same logical pin appears multiple times in a generated schematic symbol.",
        causes: &[
            "The pin was listed on both sides of the symbol.",
            "A helper expanded the same pin name twice.",
        ],
        help: &["Remove the duplicate symbol pin entry."],
    },
    DiagnosticDefinition {
        code: "net.too_few_connections",
        title: "Net has too few connections",
        explanation: "A net with fewer than two connections usually cannot represent a meaningful electrical relationship.",
        causes: &[
            "Only one pin was connected.",
            "A second connection was accidentally omitted.",
        ],
        help: &["Connect at least two pins, or remove the unused net until it is needed."],
    },
    DiagnosticDefinition {
        code: "net.unknown_pin",
        title: "Net references an unknown pin",
        explanation: "The net connects to a pin name that does not exist on the target module.",
        causes: &[
            "The pin name is misspelled.",
            "The module helper exposes a different logical pin name than expected.",
        ],
        help: &[
            "Define the pin on the part, fix the pin name, or connect the net to an existing pin.",
        ],
    },
    DiagnosticDefinition {
        code: "net.unknown_module",
        title: "Net references an unknown module",
        explanation: "The net connects to a module refdes that is not present in the board.",
        causes: &[
            "The module was never added to the design.",
            "A stored ModuleId or refdes came from another design.",
        ],
        help: &[
            "Add the module before connecting it, or update the connection to use an existing module.",
        ],
    },
    DiagnosticDefinition {
        code: "net.electrical_class_mismatch",
        title: "Net electrical class does not match a connected pin",
        explanation: "The net's declared electrical class is incompatible with one of the connected pin classes.",
        causes: &[
            "A power domain name is wrong.",
            "A logic signal was connected to a power or motor-phase pin.",
            "A pin was classified too strictly.",
        ],
        help: &[
            "Check the net class and the connected pin class, then make the electrical intent match.",
        ],
    },
    DiagnosticDefinition {
        code: "net.physical_pad_short",
        title: "Physical pad is connected to multiple nets",
        explanation: "Two or more logical pins mapped to the same footprint pad are connected to different nets, which would short those nets on the PCB.",
        causes: &[
            "The pin-to-pad map reuses a pad accidentally.",
            "Equivalent pins sharing a pad were connected to different nets.",
        ],
        help: &["Map the pins to distinct pads or connect equivalent pins to the same net."],
    },
    DiagnosticDefinition {
        code: "production.unverified_footprint",
        title: "Footprint still requires physical verification",
        explanation: "The module is marked as needing footprint verification before production checks can pass.",
        causes: &[
            "The footprint came from a drawing, marketplace module, or hand-entered dimensions.",
            "The exact purchased part has not been measured yet.",
        ],
        help: &[
            "Measure the real part or module, then remove the verification marker only after the footprint matches.",
        ],
    },
    DiagnosticDefinition {
        code: "production.missing_source",
        title: "Production source is missing",
        explanation: "The module has no manufacturer part number or supplier part number for production.",
        causes: &[
            "The part is still a placeholder.",
            "The BOM source was not recorded in the part helper.",
        ],
        help: &["Add an MPN or supplier part number such as an LCSC code."],
    },
    DiagnosticDefinition {
        code: "project.unsupported_schema",
        title: "Unsupported project configuration schema",
        explanation: "The via.toml file declares a schema version this VIA release does not understand.",
        causes: &[
            "The project was created by a newer VIA release.",
            "The schema value was entered incorrectly.",
        ],
        help: &["Use `schema = 1` for this release, or upgrade VIA before opening the project."],
    },
    DiagnosticDefinition {
        code: "project.invalid_footprint_config",
        title: "Invalid KiCad footprint configuration",
        explanation: "The [kicad-footprints] table contains an empty or contradictory source setting.",
        causes: &[
            "Both legacy source and url were configured.",
            "A required version or URL value is empty.",
        ],
        help: &[
            "Use version plus an optional url. The default release source does not need a source field.",
        ],
    },
    DiagnosticDefinition {
        code: "project.missing_manifest",
        title: "No via project manifest was found",
        explanation: "via needs a via.toml file to know which design provider to run.",
        causes: &[
            "The command was run outside a via project.",
            "The project path points at the wrong directory.",
        ],
        help: &["Run `via init` to create a project, or pass `--project <FILE_OR_DIR>`."],
    },
    DiagnosticDefinition {
        code: "project.invalid_manifest",
        title: "via.toml could not be parsed",
        explanation: "The project manifest is not valid TOML or does not match via's expected project shape.",
        causes: &[
            "The TOML syntax is invalid.",
            "A field has the wrong type or provider shape.",
        ],
        help: &[
            "Fix via.toml, or run `via init` in a scratch directory to compare against a fresh manifest.",
        ],
    },
    DiagnosticDefinition {
        code: "project.unknown_design",
        title: "Unknown design name",
        explanation: "The requested design is not declared under [designs] in via.toml.",
        causes: &[
            "The design argument is misspelled.",
            "The project manifest does not declare that design.",
        ],
        help: &["Run `via designs` to list available designs, then pass one of those names."],
    },
    DiagnosticDefinition {
        code: "project.invalid_default_design",
        title: "Default design is not declared",
        explanation: "The project's default design points to a name that is not present under [designs].",
        causes: &[
            "The default-design field is stale.",
            "The design was renamed without updating [project].",
        ],
        help: &["Update [project].default-design or add the missing [designs.<name>] entry."],
    },
    DiagnosticDefinition {
        code: "project.no_designs",
        title: "Project declares no designs",
        explanation: "The manifest loaded successfully, but it has no design entries for via to build.",
        causes: &[
            "The [designs] table is missing.",
            "A scaffold was edited before a design provider was added.",
        ],
        help: &["Add a [designs.<name>] entry or regenerate a starter project with `via init`."],
    },
    DiagnosticDefinition {
        code: "project.ambiguous_design",
        title: "Multiple designs require an explicit choice",
        explanation: "The project defines more than one design and no default design was selected.",
        causes: &[
            "No design argument was passed.",
            "[project].default-design is not set.",
        ],
        help: &["Pass a design name or set [project].default-design in via.toml."],
    },
    DiagnosticDefinition {
        code: "provider.invalid_board_ir",
        title: "Design provider did not emit valid Board IR JSON",
        explanation: "via ran the design provider, but stdout could not be parsed as Board IR JSON.",
        causes: &[
            "The provider printed logs to stdout.",
            "The provider crashed after writing partial JSON.",
            "The provider uses an incompatible Board IR shape.",
        ],
        help: &["Print logs to stderr, not stdout; keep stdout for Board IR JSON only."],
    },
    DiagnosticDefinition {
        code: "provider.command_not_found",
        title: "Design provider command could not be started",
        explanation: "via could not spawn the provider command declared in via.toml.",
        causes: &[
            "The program is missing from PATH.",
            "The command path is wrong.",
            "The current platform cannot run the command.",
        ],
        help: &["Install the provider command or update via.toml to point at the correct program."],
    },
    DiagnosticDefinition {
        code: "provider.command_failed",
        title: "Design provider command failed",
        explanation: "The provider process exited with a failing status before via could parse Board IR.",
        causes: &[
            "The Rust design did not compile.",
            "The provider panicked or returned an error.",
            "The provider wrote diagnostics and exited non-zero.",
        ],
        help: &[
            "Fix the provider error shown in stderr. Provider stdout must contain only Board IR JSON on success.",
        ],
    },
    DiagnosticDefinition {
        code: "provider.command_timed_out",
        title: "Design provider timed out",
        explanation: "The provider exceeded its configured execution timeout and was terminated.",
        causes: &[
            "The provider is waiting for interactive input.",
            "A build or subprocess is stuck.",
            "The configured timeout is too short for the project.",
        ],
        help: &[
            "Remove interactive waits or set timeout-seconds on the provider to a suitable positive value.",
        ],
    },
    DiagnosticDefinition {
        code: "provider.termination_failed",
        title: "Timed-out provider could not be terminated",
        explanation: "The provider exceeded its timeout, but the operating system rejected the termination request.",
        causes: &[
            "The process exited during timeout handling.",
            "Operating-system permissions or process state prevented termination.",
        ],
        help: &["Check for a remaining provider process and terminate it before retrying."],
    },
    DiagnosticDefinition {
        code: "provider.output_too_large",
        title: "Design provider output exceeded its limit",
        explanation: "Provider output is bounded so a noisy process cannot exhaust via's memory.",
        causes: &[
            "The Board IR document is unusually large.",
            "The provider printed logs or generated data to stdout.",
        ],
        help: &["Keep stdout limited to Board IR JSON, or raise max-output-bytes on the provider."],
    },
    DiagnosticDefinition {
        code: "provider.invalid_limits",
        title: "Design provider resource limits are invalid",
        explanation: "Provider timeout and output limits must be positive values.",
        causes: &["timeout-seconds is zero.", "max-output-bytes is zero."],
        help: &["Set positive timeout-seconds and max-output-bytes values in via.toml."],
    },
    DiagnosticDefinition {
        code: "provider.invalid_stdout_utf8",
        title: "Design provider stdout is not UTF-8",
        explanation: "Provider stdout must be UTF-8 JSON, but the process emitted invalid bytes.",
        causes: &[
            "The provider wrote binary data to stdout.",
            "A subprocess wrote non-UTF-8 text to stdout.",
        ],
        help: &["Write Board IR JSON to stdout and send binary artifacts or logs elsewhere."],
    },
    DiagnosticDefinition {
        code: "export.kicad.missing_output_dir",
        title: "KiCad export output directory is missing",
        explanation: "KiCad export needs a destination directory.",
        causes: &[
            "No `--out` argument was passed.",
            "[outputs.kicad].dir is missing from via.toml.",
        ],
        help: &["Pass `via export kicad --out <DIR>` or set [outputs.kicad].dir."],
    },
    DiagnosticDefinition {
        code: "export.invalid_file_stem",
        title: "Output file stem is unsafe",
        explanation: "Generated project names must be portable file names and cannot contain path components.",
        causes: &[
            "The board or project name contains a slash, backslash, or reserved character.",
            "The name is a reserved Windows device name such as CON or LPT1.",
        ],
        help: &["Use a simple project name such as controller_v2 without path separators."],
    },
    DiagnosticDefinition {
        code: "export.kicad.unresolved_footprint_library_uri",
        title: "Footprint library URI cannot be mapped to an output directory",
        explanation: "via can derive local output paths from relative paths, absolute paths, and ${KIPRJMOD} URIs, but not from arbitrary variables or remote URIs.",
        causes: &[
            "The footprint URI uses an environment variable other than ${KIPRJMOD}.",
            "The footprint URI uses a non-file scheme.",
        ],
        help: &[
            "Set footprint-output-dir explicitly while keeping the desired footprint-library-path URI.",
        ],
    },
    DiagnosticDefinition {
        code: "export.kicad.unsafe_footprint_library_path",
        title: "Derived footprint output path escapes the KiCad project",
        explanation: "A relative footprint library URI cannot be used to derive an output directory outside the configured KiCad project directory.",
        causes: &[
            "The URI contains one or more parent-directory components.",
            "A ${KIPRJMOD} URI points above the project directory.",
        ],
        help: &[
            "Keep the library below the KiCad project directory, or set footprint-output-dir explicitly for an intentional external location.",
        ],
    },
    DiagnosticDefinition {
        code: "export.lceda_pro.missing_output",
        title: "LCEDA Pro export output file is missing",
        explanation: "LCEDA Pro export needs a destination file.",
        causes: &[
            "No `--out` argument was passed.",
            "[outputs.lceda-pro].file is missing from via.toml.",
        ],
        help: &["Pass `via export lceda-pro --out <FILE>` or set [outputs.lceda-pro].file."],
    },
    DiagnosticDefinition {
        code: "export.pcb.missing_layout",
        title: "PCB export layout file is missing",
        explanation: "Experimental PCB export needs a layout JSON file.",
        causes: &["No `--layout` argument was passed."],
        help: &["Pass `via export pcb --layout <FILE>`."],
    },
    DiagnosticDefinition {
        code: "export.pcb.missing_output",
        title: "PCB export output file is missing",
        explanation: "Experimental PCB export needs a KiCad PCB output file path.",
        causes: &["No `--out` argument was passed."],
        help: &["Pass `via export pcb --out <FILE>`."],
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn diagnostic_codes_are_unique() {
        let mut seen = BTreeSet::new();
        for definition in DIAGNOSTIC_DEFINITIONS {
            assert!(
                seen.insert(definition.code),
                "duplicate diagnostic code {}",
                definition.code
            );
        }
    }
}
