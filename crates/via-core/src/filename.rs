use crate::{Error, Result};

pub fn validate_file_stem(stem: &str) -> Result<()> {
    let invalid_character = stem.chars().any(|ch| {
        ch.is_control() || matches!(ch, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|')
    });
    let trimmed = stem.trim();
    let reserved = trimmed
        .split('.')
        .next()
        .map(|base| {
            matches!(
                base.to_ascii_uppercase().as_str(),
                "CON"
                    | "PRN"
                    | "AUX"
                    | "NUL"
                    | "COM1"
                    | "COM2"
                    | "COM3"
                    | "COM4"
                    | "COM5"
                    | "COM6"
                    | "COM7"
                    | "COM8"
                    | "COM9"
                    | "LPT1"
                    | "LPT2"
                    | "LPT3"
                    | "LPT4"
                    | "LPT5"
                    | "LPT6"
                    | "LPT7"
                    | "LPT8"
                    | "LPT9"
            )
        })
        .unwrap_or(false);

    if trimmed.is_empty()
        || trimmed != stem
        || matches!(stem, "." | "..")
        || stem.ends_with('.')
        || invalid_character
        || reserved
    {
        return Err(Error::diagnostic(
            "export.invalid_file_stem",
            format!(
                "{stem:?} is not a safe output file stem; use a single file name without path separators or reserved characters"
            ),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_portable_file_stems() {
        for stem in ["demo", "demo_board", "赤道仪 控制板", "board-v2"] {
            validate_file_stem(stem).unwrap();
        }
    }

    #[test]
    fn rejects_paths_and_windows_reserved_names() {
        for stem in [
            "",
            "..",
            "../escape",
            "a/b",
            "a\\b",
            "C:drive",
            "name.",
            " name",
            "CON",
            "lpt1.txt",
        ] {
            let err = validate_file_stem(stem).unwrap_err();
            let Error::Diagnostic(diagnostic) = err else {
                panic!("expected a coded diagnostic for {stem:?}");
            };
            assert_eq!(diagnostic.code(), Some("export.invalid_file_stem"));
        }
    }
}
