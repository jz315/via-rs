use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;
use via_core::{Board, BoardIr, Error, Result};

#[derive(Debug, Clone)]
pub struct Project {
    root: PathBuf,
    manifest_path: PathBuf,
    config: ProjectConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProjectConfig {
    pub project: ProjectMeta,
    #[serde(default)]
    pub designs: BTreeMap<String, DesignSpec>,
    #[serde(default)]
    pub outputs: Outputs,
    #[serde(default, rename = "kicad-footprints")]
    pub kicad_footprints: Option<KicadFootprintsConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProjectMeta {
    pub name: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(rename = "default-design")]
    pub default_design: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DesignSpec {
    #[serde(flatten)]
    pub provider: ProviderSpec,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "provider", rename_all = "kebab-case")]
pub enum ProviderSpec {
    Cargo {
        package: String,
        #[serde(default)]
        bin: Option<String>,
        #[serde(default)]
        command: Option<String>,
        #[serde(default)]
        args: Vec<String>,
    },
    Command {
        program: String,
        #[serde(default)]
        args: Vec<String>,
    },
    File {
        path: PathBuf,
    },
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Outputs {
    #[serde(default)]
    pub kicad: Option<KicadOutput>,
    #[serde(default)]
    pub lceda: Option<LcedaOutput>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KicadOutput {
    pub dir: PathBuf,
    #[serde(default)]
    pub project: Option<String>,
    #[serde(default, rename = "footprint-library-name")]
    pub footprint_library_name: Option<String>,
    #[serde(default, rename = "footprint-library-path")]
    pub footprint_library_path: Option<String>,
    #[serde(default, rename = "footprint-output-dir")]
    pub footprint_output_dir: Option<PathBuf>,
    #[serde(default)]
    pub emit: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KicadFootprintExport {
    pub library_name: String,
    pub library_path: String,
    pub output_dir: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KicadFootprintsConfig {
    pub version: String,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LcedaOutput {
    pub file: PathBuf,
}

impl Project {
    pub fn discover(explicit: Option<PathBuf>) -> Result<Self> {
        let manifest_path = match explicit {
            Some(path) if path.is_dir() => path.join("via.toml"),
            Some(path) => path,
            None => discover_manifest(&std::env::current_dir()?).ok_or_else(|| {
                Error::Io("could not find via.toml in this directory or its parents".to_owned())
            })?,
        };
        Self::load(manifest_path)
    }

    pub fn load(manifest_path: impl Into<PathBuf>) -> Result<Self> {
        let manifest_path = manifest_path.into();
        let text = std::fs::read_to_string(&manifest_path)?;
        let config: ProjectConfig = toml::from_str(&text).map_err(|err| {
            Error::Io(format!(
                "failed to parse {}: {err}",
                manifest_path.display()
            ))
        })?;
        let root = manifest_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));

        Ok(Self {
            root,
            manifest_path,
            config,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn manifest_path(&self) -> &Path {
        &self.manifest_path
    }

    pub fn config(&self) -> &ProjectConfig {
        &self.config
    }

    pub fn kicad_footprints_version(&self) -> Option<&str> {
        self.config
            .kicad_footprints
            .as_ref()
            .map(|config| config.version.as_str())
    }

    pub fn kicad_footprints_url(&self) -> Option<&str> {
        self.config
            .kicad_footprints
            .as_ref()
            .and_then(|config| config.url.as_deref())
    }

    pub fn design_names(&self) -> impl Iterator<Item = &String> {
        self.config.designs.keys()
    }

    pub fn resolve_design_name<'a>(&'a self, requested: Option<&'a str>) -> Result<&'a str> {
        if let Some(name) = requested {
            if self.config.designs.contains_key(name) {
                return Ok(name);
            }
            return Err(Error::Io(format!("unknown design {name}")));
        }

        if let Some(name) = self.config.project.default_design.as_deref() {
            if self.config.designs.contains_key(name) {
                return Ok(name);
            }
            return Err(Error::Io(format!(
                "default design {name} is not listed under [designs]"
            )));
        }

        match self.config.designs.len() {
            0 => Err(Error::Io("via.toml does not define any designs".to_owned())),
            1 => Ok(self.config.designs.keys().next().unwrap()),
            _ => Err(Error::Io(
                "multiple designs are defined; pass a design name".to_owned(),
            )),
        }
    }

    pub fn build_design(&self, requested: Option<&str>) -> Result<(String, Board)> {
        let name = self.resolve_design_name(requested)?.to_owned();
        let spec = self
            .config
            .designs
            .get(&name)
            .expect("resolved design exists");
        let json = spec.provider.emit_ir(self.root())?;
        let ir: BoardIr = serde_json::from_str(&json).map_err(|err| {
            Error::Io(format!(
                "provider for {name} did not emit BoardIr JSON: {err}"
            ))
        })?;
        let board = Board::from_ir(ir)?;
        Ok((name, board))
    }

    pub fn kicad_output_dir(&self, override_dir: Option<PathBuf>) -> Result<PathBuf> {
        let path = override_dir
            .or_else(|| {
                self.config
                    .outputs
                    .kicad
                    .as_ref()
                    .map(|out| out.dir.clone())
            })
            .ok_or_else(|| {
                Error::Io("export kicad requires --out or [outputs.kicad].dir".to_owned())
            })?;
        Ok(self.resolve_path(path))
    }

    pub fn kicad_project_name(&self, board: &Board) -> String {
        self.config
            .outputs
            .kicad
            .as_ref()
            .and_then(|out| out.project.clone())
            .unwrap_or_else(|| board.name().to_owned())
    }

    pub fn kicad_footprint_library_name(&self, override_name: Option<String>) -> Result<String> {
        override_name
            .or_else(|| {
                self.config
                    .outputs
                    .kicad
                    .as_ref()
                    .and_then(|out| out.footprint_library_name.clone())
            })
            .ok_or_else(|| {
                Error::Io(
                    "PCB export requires --footprint-library-name or [outputs.kicad].footprint-library-name"
                        .to_owned(),
                )
            })
    }

    pub fn kicad_footprint_export(
        &self,
        library_name: Option<String>,
        library_path: Option<String>,
        output_dir: Option<PathBuf>,
    ) -> Result<KicadFootprintExport> {
        let output = self.config.outputs.kicad.as_ref();
        let library_name = library_name
            .or_else(|| output.and_then(|out| out.footprint_library_name.clone()))
            .ok_or_else(|| {
                Error::Io(
                    "export kicad requires --footprint-library-name or [outputs.kicad].footprint-library-name"
                        .to_owned(),
                )
            })?;
        let library_path = library_path
            .or_else(|| output.and_then(|out| out.footprint_library_path.clone()))
            .ok_or_else(|| {
                Error::Io(
                    "export kicad requires --footprint-library-path or [outputs.kicad].footprint-library-path"
                        .to_owned(),
                )
            })?;
        let output_dir = output_dir
            .or_else(|| output.and_then(|out| out.footprint_output_dir.clone()))
            .ok_or_else(|| {
                Error::Io(
                    "export kicad requires --footprint-output-dir or [outputs.kicad].footprint-output-dir"
                        .to_owned(),
                )
            })?;

        Ok(KicadFootprintExport {
            library_name,
            library_path,
            output_dir: self.resolve_path(output_dir),
        })
    }

    pub fn lceda_output_file(&self, override_file: Option<PathBuf>) -> Result<PathBuf> {
        let path = override_file
            .or_else(|| {
                self.config
                    .outputs
                    .lceda
                    .as_ref()
                    .map(|out| out.file.clone())
            })
            .ok_or_else(|| {
                Error::Io("export lceda-pro requires --out or [outputs.lceda].file".to_owned())
            })?;
        Ok(self.resolve_path(path))
    }

    pub fn resolve_path(&self, path: PathBuf) -> PathBuf {
        if path.is_absolute() {
            path
        } else {
            self.root.join(path)
        }
    }
}

impl ProviderSpec {
    pub fn emit_ir(&self, cwd: &Path) -> Result<String> {
        match self {
            ProviderSpec::Cargo {
                package,
                bin,
                command,
                args,
            } => {
                let mut cargo_args = vec![
                    "run".to_owned(),
                    "-q".to_owned(),
                    "-p".to_owned(),
                    package.clone(),
                ];
                if let Some(bin) = bin {
                    cargo_args.push("--bin".to_owned());
                    cargo_args.push(bin.clone());
                }
                cargo_args.push("--".to_owned());
                if let Some(command) = command {
                    cargo_args.push(command.clone());
                }
                cargo_args.extend(args.iter().cloned());
                run_capture(cwd, "cargo", &cargo_args)
            }
            ProviderSpec::Command { program, args } => run_capture(cwd, program, args),
            ProviderSpec::File { path } => {
                std::fs::read_to_string(cwd.join(path)).map_err(Into::into)
            }
        }
    }
}

pub fn emit_ir(board: &Board) -> Result<()> {
    let stdout = io::stdout();
    let mut lock = stdout.lock();
    serde_json::to_writer_pretty(&mut lock, &board.to_ir())
        .map_err(|err| Error::Io(format!("failed to write BoardIr JSON: {err}")))?;
    Ok(())
}

pub fn board_ir_json(board: &Board) -> Result<String> {
    serde_json::to_string_pretty(&board.to_ir())
        .map_err(|err| Error::Io(format!("failed to serialize BoardIr JSON: {err}")))
}

fn discover_manifest(start: &Path) -> Option<PathBuf> {
    for dir in start.ancestors() {
        let candidate = dir.join("via.toml");
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

fn run_capture(cwd: &Path, program: &str, args: &[String]) -> Result<String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|err| {
            Error::Io(format!(
                "failed to run provider command `{}`: {err}",
                command_line(program, args)
            ))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(Error::Io(format!(
            "provider command `{}` failed with status {}\nstdout:\n{}\nstderr:\n{}",
            command_line(program, args),
            output.status,
            stdout.trim(),
            stderr.trim()
        )));
    }

    String::from_utf8(output.stdout).map_err(|err| Error::Io(err.to_string()))
}

fn command_line(program: &str, args: &[String]) -> String {
    std::iter::once(program.to_owned())
        .chain(args.iter().cloned())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn parses_inline_provider_designs() {
        let text = r#"
            [project]
            name = "demo"
            version = "0.1.0"
            default-design = "main"

            [designs.main]
            provider = "file"
            path = "main.board.json"

            [outputs.kicad]
            dir = "generated/kicad"
            project = "demo_board"
        "#;

        let config: ProjectConfig = toml::from_str(text).unwrap();

        assert_eq!(config.project.name, "demo");
        assert!(matches!(
            config.designs["main"].provider,
            ProviderSpec::File { .. }
        ));
        assert_eq!(
            config.outputs.kicad.unwrap().dir,
            PathBuf::from("generated/kicad")
        );
    }

    #[test]
    fn file_provider_builds_board_from_board_ir() {
        let root = std::env::temp_dir().join(format!(
            "via_project_file_provider_{}_{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();

        let mut design = via_core::Design::new("file_provider_demo");
        let module = design
            .add(
                via_core::part("J1", "Header")
                    .footprint("Header_1x02")
                    .pin(via_core::pin("1").pad("1"))
                    .pin(via_core::pin("2").pad("2")),
            )
            .unwrap();
        design.add_footprint_pads(via_core::FootprintPads::new("Header_1x02", ["1", "2"]));
        design
            .net("N")
            .connect_all(&mut design, [module.pin("1"), module.pin("2")]);
        let board = design.build().unwrap();

        std::fs::write(
            root.join("board.json"),
            serde_json::to_string_pretty(&board.to_ir()).unwrap(),
        )
        .unwrap();
        std::fs::write(
            root.join("via.toml"),
            r#"
                [project]
                name = "file-provider"
                default-design = "main"

                [designs.main]
                provider = "file"
                path = "board.json"
            "#,
        )
        .unwrap();

        let project = Project::load(root.join("via.toml")).unwrap();
        let (design_name, loaded) = project.build_design(None).unwrap();

        assert_eq!(design_name, "main");
        assert_eq!(loaded.name(), "file_provider_demo");
        assert_eq!(loaded.modules().count(), 1);

        std::fs::remove_dir_all(root).unwrap();
    }
}
