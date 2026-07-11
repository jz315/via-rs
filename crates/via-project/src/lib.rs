use std::collections::BTreeMap;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use serde::Deserialize;
use via_core::{Board, BoardIr, Error, Result};

#[derive(Debug, Clone)]
pub struct Project {
    root: PathBuf,
    manifest_path: PathBuf,
    config: ProjectConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfig {
    /// Project configuration schema. Omitted manifests are treated as legacy
    /// schema zero during the 0.3 migration window.
    #[serde(default)]
    pub schema: Option<u32>,
    pub project: ProjectMeta,
    #[serde(default)]
    pub designs: BTreeMap<String, DesignSpec>,
    #[serde(default)]
    pub outputs: Outputs,
    #[serde(default, rename = "kicad-footprints")]
    pub kicad_footprints: Option<KicadFootprintsConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
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
        #[serde(default, rename = "timeout-seconds")]
        timeout_seconds: Option<u64>,
        #[serde(default, rename = "max-output-bytes")]
        max_output_bytes: Option<usize>,
    },
    Command {
        program: String,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default, rename = "timeout-seconds")]
        timeout_seconds: Option<u64>,
        #[serde(default, rename = "max-output-bytes")]
        max_output_bytes: Option<usize>,
    },
    File {
        path: PathBuf,
    },
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outputs {
    #[serde(default)]
    pub kicad: Option<KicadOutput>,
    #[serde(default, rename = "lceda-pro", alias = "lceda")]
    pub lceda_pro: Option<LcedaOutput>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KicadOutput {
    #[serde(default)]
    pub dir: Option<PathBuf>,
    #[serde(default, rename = "project-name", alias = "project")]
    pub project_name: Option<String>,
    #[serde(default, rename = "footprint-library-name")]
    pub footprint_library_name: Option<String>,
    #[serde(default, rename = "footprint-library-path")]
    pub footprint_library_path: Option<String>,
    #[serde(default, rename = "footprint-output-dir")]
    pub footprint_output_dir: Option<PathBuf>,
    #[serde(default, rename = "emit")]
    _legacy_emit: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KicadFootprintExport {
    pub library_name: String,
    pub library_path: String,
    pub output_dir: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KicadFootprintsConfig {
    pub version: String,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LcedaOutput {
    pub file: PathBuf,
}

impl Project {
    pub fn discover(explicit: Option<PathBuf>) -> Result<Self> {
        let manifest_path = match explicit {
            Some(path) if path.is_dir() => {
                let manifest = path.join("via.toml");
                if !manifest.exists() {
                    return Err(Error::diagnostic(
                        "project.missing_manifest",
                        format!("could not find via.toml in {}", path.display()),
                    ));
                }
                manifest
            }
            Some(path) => {
                if !path.exists() {
                    return Err(Error::diagnostic(
                        "project.missing_manifest",
                        format!("project path {} does not exist", path.display()),
                    ));
                }
                path
            }
            None => discover_manifest(&std::env::current_dir()?).ok_or_else(|| {
                Error::diagnostic(
                    "project.missing_manifest",
                    "could not find via.toml in this directory or its parents",
                )
            })?,
        };
        Self::load(manifest_path)
    }

    pub fn load(manifest_path: impl Into<PathBuf>) -> Result<Self> {
        let manifest_path = manifest_path.into();
        let text = std::fs::read_to_string(&manifest_path)?;
        let config: ProjectConfig = toml::from_str(&text).map_err(|err| {
            Error::diagnostic(
                "project.invalid_manifest",
                format!("failed to parse {}: {err}", manifest_path.display()),
            )
        })?;
        validate_config(&config)?;
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

    /// Returns a legacy footprint source value, if a manifest still uses one.
    /// New manifests should use the default VIA release or the explicit `url`
    /// field instead.
    pub fn kicad_footprints_legacy_source(&self) -> Option<&str> {
        self.config
            .kicad_footprints
            .as_ref()
            .and_then(|config| config.source.as_deref())
    }

    pub fn design_names(&self) -> impl Iterator<Item = &String> {
        self.config.designs.keys()
    }

    pub fn resolve_design_name<'a>(&'a self, requested: Option<&'a str>) -> Result<&'a str> {
        if let Some(name) = requested {
            if self.config.designs.contains_key(name) {
                return Ok(name);
            }
            return Err(Error::diagnostic(
                "project.unknown_design",
                format!(
                    "unknown design {name}; available designs: {}",
                    design_list(&self.config.designs)
                ),
            ));
        }

        if let Some(name) = self.config.project.default_design.as_deref() {
            if self.config.designs.contains_key(name) {
                return Ok(name);
            }
            return Err(Error::diagnostic(
                "project.invalid_default_design",
                format!(
                    "default design {name} is not listed under [designs]; available designs: {}",
                    design_list(&self.config.designs)
                ),
            ));
        }

        match self.config.designs.len() {
            0 => Err(Error::diagnostic(
                "project.no_designs",
                "via.toml does not define any designs",
            )),
            1 => Ok(self.config.designs.keys().next().unwrap()),
            _ => Err(Error::diagnostic(
                "project.ambiguous_design",
                format!(
                    "multiple designs are defined; pass a design name; available designs: {}",
                    design_list(&self.config.designs)
                ),
            )),
        }
    }

    pub fn build_design(&self, requested: Option<&str>) -> Result<(String, Board)> {
        let name = self.resolve_design_name(requested)?.to_owned();
        let json = self.emit_design_ir_json(&name)?;
        let ir = parse_board_ir_json(&name, &json)?;
        let board = Board::from_ir(ir)?;
        Ok((name, board))
    }

    pub fn emit_design_ir_json(&self, design_name: &str) -> Result<String> {
        let spec = self.config.designs.get(design_name).ok_or_else(|| {
            Error::diagnostic(
                "project.unknown_design",
                format!(
                    "unknown design {design_name}; available designs: {}",
                    design_list(&self.config.designs)
                ),
            )
        })?;
        spec.provider.emit_ir(self.root())
    }

    pub fn kicad_output_dir(&self, override_dir: Option<PathBuf>) -> Result<PathBuf> {
        let path = override_dir
            .or_else(|| {
                self.config
                    .outputs
                    .kicad
                    .as_ref()
                    .and_then(|out| out.dir.clone())
            })
            .ok_or_else(|| {
                Error::diagnostic(
                    "export.kicad.missing_output_dir",
                    "export kicad requires --out or [outputs.kicad].dir",
                )
            })?;
        Ok(self.resolve_path(path))
    }

    pub fn kicad_project_name(&self, board: &Board) -> Result<String> {
        let name = self
            .config
            .outputs
            .kicad
            .as_ref()
            .and_then(|out| out.project_name.clone())
            .unwrap_or_else(|| board.name().to_owned());
        via_core::validate_file_stem(&name)?;
        Ok(name)
    }

    pub fn kicad_footprint_library_name(
        &self,
        override_name: Option<String>,
        project_name: &str,
    ) -> Result<String> {
        let name = override_name
            .or_else(|| {
                self.config
                    .outputs
                    .kicad
                    .as_ref()
                    .and_then(|out| out.footprint_library_name.clone())
            })
            .unwrap_or_else(|| project_name.to_owned());
        via_core::validate_file_stem(&name)?;
        Ok(name)
    }

    pub fn kicad_footprint_export(
        &self,
        kicad_output_dir: &Path,
        project_name: &str,
        library_name: Option<String>,
        library_path: Option<String>,
        output_dir: Option<PathBuf>,
    ) -> Result<KicadFootprintExport> {
        let output = self.config.outputs.kicad.as_ref();
        let library_name = library_name
            .or_else(|| output.and_then(|out| out.footprint_library_name.clone()))
            .unwrap_or_else(|| project_name.to_owned());
        via_core::validate_file_stem(&library_name)?;
        let library_path = library_path
            .or_else(|| output.and_then(|out| out.footprint_library_path.clone()))
            .unwrap_or_else(|| format!("${{KIPRJMOD}}/{library_name}.pretty"));
        let output_dir = if let Some(output_dir) =
            output_dir.or_else(|| output.and_then(|out| out.footprint_output_dir.clone()))
        {
            self.resolve_path(output_dir)
        } else {
            default_footprint_output_dir(kicad_output_dir, &library_path)?
        };

        Ok(KicadFootprintExport {
            library_name,
            library_path,
            output_dir,
        })
    }

    pub fn lceda_output_file(&self, override_file: Option<PathBuf>) -> Result<PathBuf> {
        let path = override_file
            .or_else(|| {
                self.config
                    .outputs
                    .lceda_pro
                    .as_ref()
                    .map(|out| out.file.clone())
            })
            .ok_or_else(|| {
                Error::diagnostic(
                    "export.lceda_pro.missing_output",
                    "export lceda-pro requires --out or [outputs.lceda-pro].file",
                )
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

fn validate_config(config: &ProjectConfig) -> Result<()> {
    if let Some(schema) = config.schema
        && schema != 1
    {
        return Err(Error::diagnostic(
            "project.unsupported_schema",
            format!("unsupported via.toml schema {schema}; expected schema = 1"),
        ));
    }

    if let Some(footprints) = &config.kicad_footprints {
        if footprints.version.trim().is_empty() {
            return Err(Error::diagnostic(
                "project.invalid_footprint_config",
                "[kicad-footprints].version must not be empty",
            ));
        }
        if footprints
            .url
            .as_deref()
            .is_some_and(|url| url.trim().is_empty())
        {
            return Err(Error::diagnostic(
                "project.invalid_footprint_config",
                "[kicad-footprints].url must not be empty when set",
            ));
        }
        if footprints.url.is_some() && footprints.source.is_some() {
            return Err(Error::diagnostic(
                "project.invalid_footprint_config",
                "use either [kicad-footprints].url or the legacy source field, not both",
            ));
        }
        if let Some(source) = footprints.source.as_deref()
            && source != "github-release"
            && !(source.starts_with("https://") || source.starts_with("http://"))
        {
            return Err(Error::diagnostic(
                "project.invalid_footprint_config",
                "legacy [kicad-footprints].source must be github-release or an HTTP(S) bundle URL; use url in schema = 1 manifests",
            ));
        }
    }

    Ok(())
}

impl ProviderSpec {
    pub fn emit_ir(&self, cwd: &Path) -> Result<String> {
        match self {
            ProviderSpec::Cargo {
                package,
                bin,
                command,
                args,
                timeout_seconds,
                max_output_bytes,
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
                run_capture(
                    cwd,
                    "cargo",
                    &cargo_args,
                    ProviderLimits::new(*timeout_seconds, *max_output_bytes),
                )
            }
            ProviderSpec::Command {
                program,
                args,
                timeout_seconds,
                max_output_bytes,
            } => run_capture(
                cwd,
                program,
                args,
                ProviderLimits::new(*timeout_seconds, *max_output_bytes),
            ),
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

pub fn parse_board_ir_json(design_name: &str, json: &str) -> Result<BoardIr> {
    serde_json::from_str(json).map_err(|err| {
        Error::diagnostic(
            "provider.invalid_board_ir",
            format!(
                "provider stdout is not valid Board IR JSON for {design_name}: {err}\nstdout preview:\n{}\nhint: print logs to stderr, not stdout",
                stdout_preview(json)
            ),
        )
    })
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

fn design_list(designs: &BTreeMap<String, DesignSpec>) -> String {
    if designs.is_empty() {
        return "none".to_owned();
    }
    designs.keys().cloned().collect::<Vec<_>>().join(", ")
}

fn default_footprint_output_dir(kicad_output_dir: &Path, library_path: &str) -> Result<PathBuf> {
    const KIPRJMOD: &str = "${KIPRJMOD}";
    if let Some(relative) = library_path.strip_prefix(KIPRJMOD) {
        let relative = relative.trim_start_matches(['/', '\\']);
        return join_safe_relative_library_path(kicad_output_dir, relative, library_path);
    }
    if library_path.contains("${") || library_path.contains("://") {
        return Err(Error::diagnostic(
            "export.kicad.unresolved_footprint_library_uri",
            format!(
                "cannot derive a local footprint output directory from KiCad URI {library_path:?}; set footprint-output-dir explicitly"
            ),
        ));
    }
    let path = PathBuf::from(library_path);
    if path.is_absolute() {
        Ok(path)
    } else {
        join_safe_relative_library_path(kicad_output_dir, &path, library_path)
    }
}

fn join_safe_relative_library_path(
    root: &Path,
    relative: impl AsRef<Path>,
    original: &str,
) -> Result<PathBuf> {
    let relative = relative.as_ref();
    if relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(Error::diagnostic(
            "export.kicad.unsafe_footprint_library_path",
            format!(
                "cannot derive a footprint output directory outside the KiCad project from {original:?}; set footprint-output-dir explicitly"
            ),
        ));
    }
    Ok(root.join(relative))
}

const DEFAULT_PROVIDER_TIMEOUT: Duration = Duration::from_secs(120);
const DEFAULT_PROVIDER_MAX_OUTPUT_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, Copy)]
struct ProviderLimits {
    timeout: Duration,
    max_output_bytes: usize,
}

impl ProviderLimits {
    fn new(timeout_seconds: Option<u64>, max_output_bytes: Option<usize>) -> Self {
        Self {
            timeout: timeout_seconds
                .map(Duration::from_secs)
                .unwrap_or(DEFAULT_PROVIDER_TIMEOUT),
            max_output_bytes: max_output_bytes.unwrap_or(DEFAULT_PROVIDER_MAX_OUTPUT_BYTES),
        }
    }
}

struct CapturedOutput {
    bytes: Vec<u8>,
    truncated: bool,
}

struct ManagedChild {
    child: Child,
    tree: ProcessTree,
}

impl ManagedChild {
    fn spawn(command: &mut Command) -> io::Result<Self> {
        configure_process_group(command);
        let mut child = command.spawn()?;
        let tree = match ProcessTree::attach(&child) {
            Ok(tree) => tree,
            Err(err) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(err);
            }
        };
        Ok(Self { child, tree })
    }

    fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        self.child.try_wait()
    }

    fn terminate(&mut self) -> io::Result<ExitStatus> {
        let tree_error = self.tree.terminate().err();
        let child_error = self.child.kill().err();
        if let (Some(tree_error), Some(child_error)) = (tree_error, child_error) {
            return Err(io::Error::other(format!(
                "failed to terminate provider process tree: {tree_error}; direct child termination also failed: {child_error}"
            )));
        }
        self.child.wait()
    }

    fn terminate_descendants(&self) {
        let _ = self.tree.terminate();
    }
}

#[cfg(unix)]
fn configure_process_group(command: &mut Command) {
    use std::os::unix::process::CommandExt;
    command.process_group(0);
}

#[cfg(not(unix))]
fn configure_process_group(_command: &mut Command) {}

#[cfg(unix)]
struct ProcessTree {
    process_group: i32,
}

#[cfg(unix)]
impl ProcessTree {
    fn attach(child: &Child) -> io::Result<Self> {
        let process_group = i32::try_from(child.id())
            .map_err(|_| io::Error::other("provider process id exceeds i32"))?;
        Ok(Self { process_group })
    }

    fn terminate(&self) -> io::Result<()> {
        // The child was placed in a fresh process group before spawn. A negative
        // pid asks kill(2) to signal every process in that group.
        let result = unsafe { libc::kill(-self.process_group, libc::SIGKILL) };
        if result == 0 {
            return Ok(());
        }
        let err = io::Error::last_os_error();
        if err.raw_os_error() == Some(libc::ESRCH) {
            Ok(())
        } else {
            Err(err)
        }
    }
}

#[cfg(windows)]
struct ProcessTree {
    job: windows_sys::Win32::Foundation::HANDLE,
    root_process_id: u32,
}

#[cfg(windows)]
impl ProcessTree {
    fn attach(child: &Child) -> io::Result<Self> {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::System::JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
            JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
            SetInformationJobObject,
        };

        // SAFETY: null security attributes and name request an unnamed job with
        // default security. The returned handle is owned by ProcessTree.
        let job = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        if job.is_null() {
            return Err(io::Error::last_os_error());
        }
        let tree = Self {
            job,
            root_process_id: child.id(),
        };
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        // SAFETY: limits points to the structure required by the selected
        // information class and remains alive for the duration of the call.
        let configured = unsafe {
            SetInformationJobObject(
                tree.job,
                JobObjectExtendedLimitInformation,
                std::ptr::from_ref(&limits).cast(),
                std::mem::size_of_val(&limits) as u32,
            )
        };
        if configured == 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: both handles are valid for this call; the process handle is
        // borrowed from Child and remains owned by Child.
        let assigned = unsafe { AssignProcessToJobObject(tree.job, child.as_raw_handle().cast()) };
        if assigned == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(tree)
    }

    fn terminate(&self) -> io::Result<()> {
        use windows_sys::Win32::System::JobObjects::TerminateJobObject;
        let descendant_result = terminate_windows_descendants(self.root_process_id);
        // SAFETY: self.job is a live job handle owned by this object.
        let job_result = if unsafe { TerminateJobObject(self.job, 1) } == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        };
        match (descendant_result, job_result) {
            (Ok(()), _) | (_, Ok(())) => Ok(()),
            (Err(descendant_error), Err(job_error)) => Err(io::Error::other(format!(
                "failed to terminate provider descendants: {descendant_error}; job termination also failed: {job_error}"
            ))),
        }
    }
}

#[cfg(windows)]
fn terminate_windows_descendants(root_process_id: u32) -> io::Result<()> {
    use std::collections::BTreeSet;
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
        TH32CS_SNAPPROCESS,
    };
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_TERMINATE, TerminateProcess};

    // SAFETY: the process snapshot handle is checked and closed below.
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    let mut entries = Vec::new();
    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    // SAFETY: entry has the required size and snapshot is a live process snapshot.
    let mut has_entry = unsafe { Process32FirstW(snapshot, &mut entry) } != 0;
    while has_entry {
        entries.push((entry.th32ProcessID, entry.th32ParentProcessID));
        // SAFETY: arguments remain valid for the lifetime of the snapshot.
        has_entry = unsafe { Process32NextW(snapshot, &mut entry) } != 0;
    }
    // SAFETY: snapshot is owned by this function and closed exactly once.
    unsafe {
        CloseHandle(snapshot);
    }

    let mut family = BTreeSet::from([root_process_id]);
    let mut descendants = Vec::new();
    loop {
        let mut added = false;
        for &(process_id, parent_process_id) in &entries {
            if !family.contains(&process_id) && family.contains(&parent_process_id) {
                family.insert(process_id);
                descendants.push(process_id);
                added = true;
            }
        }
        if !added {
            break;
        }
    }

    let mut first_error = None;
    for process_id in descendants.into_iter().rev() {
        // SAFETY: OpenProcess returns an owned handle or null on failure.
        let process = unsafe { OpenProcess(PROCESS_TERMINATE, 0, process_id) };
        if process.is_null() {
            continue;
        }
        // SAFETY: process is a live owned process handle.
        if unsafe { TerminateProcess(process, 1) } == 0 && first_error.is_none() {
            first_error = Some(io::Error::last_os_error());
        }
        // SAFETY: process is owned by this loop iteration and closed once.
        unsafe {
            CloseHandle(process);
        }
    }
    first_error.map_or(Ok(()), Err)
}

#[cfg(windows)]
impl Drop for ProcessTree {
    fn drop(&mut self) {
        use windows_sys::Win32::Foundation::CloseHandle;
        // SAFETY: self.job is owned by this object and is closed exactly once.
        unsafe {
            CloseHandle(self.job);
        }
    }
}

#[cfg(not(any(unix, windows)))]
struct ProcessTree;

#[cfg(not(any(unix, windows)))]
impl ProcessTree {
    fn attach(_child: &Child) -> io::Result<Self> {
        Ok(Self)
    }

    fn terminate(&self) -> io::Result<()> {
        Ok(())
    }
}

fn run_capture(
    cwd: &Path,
    program: &str,
    args: &[String],
    limits: ProviderLimits,
) -> Result<String> {
    if limits.timeout.is_zero() || limits.max_output_bytes == 0 {
        return Err(Error::diagnostic(
            "provider.invalid_limits",
            "provider timeout-seconds and max-output-bytes must both be greater than zero",
        ));
    }

    let mut command = Command::new(program);
    command
        .args(args)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = ManagedChild::spawn(&mut command).map_err(|err| {
        Error::diagnostic(
            "provider.command_not_found",
            format!(
                "failed to run provider command `{}`: {err}",
                command_line(program, args)
            ),
        )
    })?;

    let stdout = child
        .child
        .stdout
        .take()
        .ok_or_else(|| Error::Io("failed to capture provider stdout".to_owned()))?;
    let stderr = child
        .child
        .stderr
        .take()
        .ok_or_else(|| Error::Io("failed to capture provider stderr".to_owned()))?;
    let stdout_limit = limits.max_output_bytes;
    let stderr_limit = limits.max_output_bytes;
    let stdout_reader = std::thread::spawn(move || read_bounded(stdout, stdout_limit));
    let stderr_reader = std::thread::spawn(move || read_bounded(stderr, stderr_limit));

    let started = Instant::now();
    let mut timed_out = false;
    let status = loop {
        if let Some(status) = child.try_wait().map_err(Error::from)? {
            child.terminate_descendants();
            break status;
        }
        if started.elapsed() >= limits.timeout {
            timed_out = true;
            match child.terminate() {
                Ok(status) => break status,
                Err(kill_error) => {
                    return Err(Error::diagnostic(
                        "provider.termination_failed",
                        format!(
                            "provider command `{}` timed out, but the process tree could not be terminated: {kill_error}",
                            command_line(program, args)
                        ),
                    ));
                }
            }
        }
        std::thread::sleep(Duration::from_millis(10));
    };

    let stdout = join_capture(stdout_reader, "stdout")?;
    let stderr = join_capture(stderr_reader, "stderr")?;
    if timed_out {
        return Err(Error::diagnostic(
            "provider.command_timed_out",
            format!(
                "provider command `{}` exceeded its {} second timeout\nstdout:\n{}\nstderr:\n{}",
                command_line(program, args),
                limits.timeout.as_secs(),
                captured_preview(&stdout),
                captured_preview(&stderr)
            ),
        ));
    }

    if !status.success() {
        return Err(Error::diagnostic(
            "provider.command_failed",
            format!(
                "provider command `{}` failed with status {}\nstdout:\n{}\nstderr:\n{}",
                command_line(program, args),
                status,
                captured_preview(&stdout),
                captured_preview(&stderr)
            ),
        ));
    }

    if stdout.truncated {
        return Err(Error::diagnostic(
            "provider.output_too_large",
            format!(
                "provider stdout exceeded the {} byte limit; increase max-output-bytes or emit a smaller Board IR document",
                limits.max_output_bytes
            ),
        ));
    }

    String::from_utf8(stdout.bytes).map_err(|err| {
        Error::diagnostic(
            "provider.invalid_stdout_utf8",
            format!("provider stdout is not valid UTF-8: {err}"),
        )
    })
}

fn read_bounded(mut reader: impl Read, limit: usize) -> io::Result<CapturedOutput> {
    let mut bytes = Vec::with_capacity(limit.min(64 * 1024));
    let mut truncated = false;
    let mut buffer = [0u8; 8192];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        let remaining = limit.saturating_sub(bytes.len());
        let keep = remaining.min(read);
        bytes.extend_from_slice(&buffer[..keep]);
        truncated |= keep < read;
    }
    Ok(CapturedOutput { bytes, truncated })
}

fn join_capture(
    handle: std::thread::JoinHandle<io::Result<CapturedOutput>>,
    stream: &str,
) -> Result<CapturedOutput> {
    handle
        .join()
        .map_err(|_| Error::Io(format!("provider {stream} reader thread panicked")))?
        .map_err(Error::from)
}

fn captured_preview(output: &CapturedOutput) -> String {
    let text = String::from_utf8_lossy(&output.bytes);
    let mut preview = trim_provider_output(&text);
    if output.truncated {
        preview.push_str("\n... output byte limit reached ...");
    }
    preview
}

fn command_line(program: &str, args: &[String]) -> String {
    std::iter::once(program.to_owned())
        .chain(args.iter().cloned())
        .collect::<Vec<_>>()
        .join(" ")
}

fn trim_provider_output(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return "<empty>".to_owned();
    }

    const MAX_CHARS: usize = 1200;
    let mut out = String::new();
    for ch in trimmed.chars().take(MAX_CHARS) {
        out.push(ch);
    }
    if trimmed.chars().count() > MAX_CHARS {
        out.push_str("\n... output truncated ...");
    }
    out
}

fn stdout_preview(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return "<empty>".to_owned();
    }

    const MAX_CHARS: usize = 400;
    let mut out = String::new();
    for ch in trimmed.chars().take(MAX_CHARS) {
        out.push(ch);
    }
    if trimmed.chars().count() > MAX_CHARS {
        out.push_str("\n... stdout preview truncated ...");
    }
    out
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
            Some(PathBuf::from("generated/kicad"))
        );
    }

    #[test]
    fn schema_one_uses_new_output_names_and_rejects_unknown_fields() {
        let config: ProjectConfig = toml::from_str(
            r#"
                schema = 1

                [project]
                name = "demo"

                [outputs.kicad]
                dir = "generated/kicad"
                project-name = "demo_board"

                [outputs.lceda-pro]
                file = "generated/demo.epro"

                [kicad-footprints]
                version = "10.0.4"
            "#,
        )
        .unwrap();
        validate_config(&config).unwrap();
        assert_eq!(
            config.outputs.kicad.unwrap().project_name.as_deref(),
            Some("demo_board")
        );
        assert!(config.outputs.lceda_pro.is_some());

        let err = toml::from_str::<ProjectConfig>(
            r#"
                [project]
                name = "demo"
                typo = true
            "#,
        )
        .unwrap_err();
        assert!(err.to_string().contains("unknown field"));
    }

    #[test]
    fn footprint_config_rejects_conflicting_legacy_source_and_url() {
        let config: ProjectConfig = toml::from_str(
            r#"
                [project]
                name = "demo"

                [kicad-footprints]
                version = "10.0.4"
                source = "github-release"
                url = "https://example.invalid/footprints.tar.zst"
            "#,
        )
        .unwrap();
        let err = validate_config(&config).unwrap_err();
        let Error::Diagnostic(diagnostic) = err else {
            panic!("expected diagnostic error");
        };
        assert_eq!(diagnostic.code(), Some("project.invalid_footprint_config"));
    }

    #[test]
    fn kicad_export_derives_mechanical_footprint_defaults() {
        let project = project_from_toml(
            r#"
                [project]
                name = "demo"
                default-design = "main"

                [designs.main]
                provider = "file"
                path = "board.json"

                [outputs.kicad]
                dir = "generated/kicad"
                project = "demo_board"
            "#,
        );
        let board = via_core::Design::new("board_name").into_unchecked_board();

        let out = project.kicad_output_dir(None).unwrap();
        let project_name = project.kicad_project_name(&board).unwrap();
        let footprint_export = project
            .kicad_footprint_export(&out, &project_name, None, None, None)
            .unwrap();

        assert_eq!(out, project.root.join("generated/kicad"));
        assert_eq!(project_name, "demo_board");
        assert_eq!(footprint_export.library_name, "demo_board");
        assert_eq!(
            footprint_export.library_path,
            "${KIPRJMOD}/demo_board.pretty"
        );
        assert_eq!(
            footprint_export.output_dir,
            project
                .root
                .join("generated/kicad")
                .join("demo_board.pretty")
        );
    }

    #[test]
    fn kicad_export_preserves_explicit_legacy_footprint_fields() {
        let project = project_from_toml(
            r#"
                [project]
                name = "demo"
                default-design = "main"

                [designs.main]
                provider = "file"
                path = "board.json"

                [outputs.kicad]
                dir = "generated/kicad"
                project = "demo_board"
                footprint-library-name = "legacy_lib"
                footprint-library-path = "libs/legacy.pretty"
                footprint-output-dir = "custom/legacy.pretty"
            "#,
        );
        let out = project.kicad_output_dir(None).unwrap();
        let footprint_export = project
            .kicad_footprint_export(&out, "demo_board", None, None, None)
            .unwrap();

        assert_eq!(footprint_export.library_name, "legacy_lib");
        assert_eq!(footprint_export.library_path, "libs/legacy.pretty");
        assert_eq!(
            footprint_export.output_dir,
            project.root.join("custom/legacy.pretty")
        );
    }

    #[test]
    fn kicad_export_maps_kiprjmod_uri_to_the_project_output_directory() {
        let project = project_from_toml(
            r#"
                [project]
                name = "demo"

                [outputs.kicad]
                dir = "generated/kicad"
                footprint-library-path = "${KIPRJMOD}/libs/demo.pretty"
            "#,
        );
        let out = project.kicad_output_dir(None).unwrap();
        let footprint_export = project
            .kicad_footprint_export(&out, "demo", None, None, None)
            .unwrap();

        assert_eq!(
            footprint_export.output_dir,
            project.root.join("generated/kicad/libs/demo.pretty")
        );
    }

    #[test]
    fn kicad_export_requires_an_explicit_output_for_unresolved_uris() {
        let project = project_from_toml(
            r#"
                [project]
                name = "demo"

                [outputs.kicad]
                dir = "generated/kicad"
                footprint-library-path = "${CUSTOM_LIB}/demo.pretty"
            "#,
        );
        let out = project.kicad_output_dir(None).unwrap();
        let err = project
            .kicad_footprint_export(&out, "demo", None, None, None)
            .unwrap_err();
        let Error::Diagnostic(diagnostic) = err else {
            panic!("expected a diagnostic error");
        };
        assert_eq!(
            diagnostic.code(),
            Some("export.kicad.unresolved_footprint_library_uri")
        );
    }

    #[test]
    fn kicad_export_rejects_derived_paths_that_escape_the_project() {
        let project = project_from_toml(
            r#"
                [project]
                name = "demo"

                [outputs.kicad]
                dir = "generated/kicad"
                footprint-library-path = "${KIPRJMOD}/../outside.pretty"
            "#,
        );
        let out = project.kicad_output_dir(None).unwrap();
        let err = project
            .kicad_footprint_export(&out, "demo", None, None, None)
            .unwrap_err();
        let Error::Diagnostic(diagnostic) = err else {
            panic!("expected a diagnostic error");
        };
        assert_eq!(
            diagnostic.code(),
            Some("export.kicad.unsafe_footprint_library_path")
        );
    }

    #[test]
    fn kicad_export_rejects_unsafe_derived_library_names() {
        let project = project_from_toml(
            r#"
                [project]
                name = "demo"

                [outputs.kicad]
                dir = "generated/kicad"
            "#,
        );
        let out = project.kicad_output_dir(None).unwrap();
        let err = project
            .kicad_footprint_export(&out, "demo", Some("../outside".to_owned()), None, None)
            .unwrap_err();
        let Error::Diagnostic(diagnostic) = err else {
            panic!("expected a diagnostic error");
        };
        assert_eq!(diagnostic.code(), Some("export.invalid_file_stem"));
    }

    #[test]
    fn kicad_project_name_rejects_path_components() {
        let project = project_from_toml(
            r#"
                [project]
                name = "demo"

                [outputs.kicad]
                project = "../escape"
            "#,
        );
        let board = via_core::Design::new("board").into_unchecked_board();
        let err = project.kicad_project_name(&board).unwrap_err();
        let Error::Diagnostic(diagnostic) = err else {
            panic!("expected a diagnostic error");
        };
        assert_eq!(diagnostic.code(), Some("export.invalid_file_stem"));
    }

    #[test]
    fn kicad_export_still_requires_explicit_output_dir() {
        let project = project_from_toml(
            r#"
                [project]
                name = "demo"
                default-design = "main"

                [designs.main]
                provider = "file"
                path = "board.json"

                [outputs.kicad]
                project = "demo_board"
            "#,
        );

        let err = project.kicad_output_dir(None).unwrap_err();
        let Error::Diagnostic(diagnostic) = err else {
            panic!("expected diagnostic error");
        };
        assert_eq!(diagnostic.code(), Some("export.kicad.missing_output_dir"));
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

    #[test]
    fn bounded_reader_drains_but_only_retains_the_configured_limit() {
        let captured = read_bounded(std::io::Cursor::new(vec![b'x'; 4096]), 32).unwrap();
        assert_eq!(captured.bytes.len(), 32);
        assert!(captured.truncated);
    }

    #[test]
    fn provider_reports_output_that_exceeds_the_byte_limit() {
        let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_owned());
        let err = run_capture(
            Path::new("."),
            &rustc,
            &["--version".to_owned()],
            ProviderLimits {
                timeout: Duration::from_secs(10),
                max_output_bytes: 1,
            },
        )
        .unwrap_err();
        let Error::Diagnostic(diagnostic) = err else {
            panic!("expected a diagnostic error");
        };
        assert_eq!(diagnostic.code(), Some("provider.output_too_large"));
    }

    #[test]
    fn provider_timeout_terminates_a_stuck_command() {
        let (program, args) = sleeping_command();
        let started = Instant::now();
        let err = run_capture(
            Path::new("."),
            program,
            &args,
            ProviderLimits {
                timeout: Duration::from_millis(100),
                max_output_bytes: 1024,
            },
        )
        .unwrap_err();
        let Error::Diagnostic(diagnostic) = err else {
            panic!("expected a diagnostic error");
        };
        assert_eq!(diagnostic.code(), Some("provider.command_timed_out"));
        assert!(started.elapsed() < Duration::from_secs(5));
    }

    #[test]
    fn provider_timeout_terminates_descendant_processes() {
        let (program, args) = sleeping_process_tree_command();
        let started = Instant::now();
        let err = run_capture(
            Path::new("."),
            program,
            &args,
            ProviderLimits {
                timeout: Duration::from_millis(500),
                max_output_bytes: 1024,
            },
        )
        .unwrap_err();
        let Error::Diagnostic(diagnostic) = err else {
            panic!("expected a diagnostic error");
        };
        assert_eq!(diagnostic.code(), Some("provider.command_timed_out"));
        assert!(started.elapsed() < Duration::from_secs(5));
    }

    #[cfg(windows)]
    fn sleeping_command() -> (&'static str, Vec<String>) {
        (
            "powershell",
            vec![
                "-NoProfile".to_owned(),
                "-Command".to_owned(),
                "Start-Sleep -Seconds 2".to_owned(),
            ],
        )
    }

    #[cfg(windows)]
    fn sleeping_process_tree_command() -> (&'static str, Vec<String>) {
        (
            "cmd",
            vec![
                "/d".to_owned(),
                "/s".to_owned(),
                "/c".to_owned(),
                "powershell.exe -NoProfile -Command Start-Sleep -Seconds 30".to_owned(),
            ],
        )
    }

    #[cfg(not(windows))]
    fn sleeping_command() -> (&'static str, Vec<String>) {
        ("sh", vec!["-c".to_owned(), "sleep 2".to_owned()])
    }

    #[cfg(not(windows))]
    fn sleeping_process_tree_command() -> (&'static str, Vec<String>) {
        ("sh", vec!["-c".to_owned(), "sleep 30".to_owned()])
    }

    fn project_from_toml(text: &str) -> Project {
        let root = PathBuf::from("via-project-test-root");
        Project {
            root: root.clone(),
            manifest_path: root.join("via.toml"),
            config: toml::from_str(text).unwrap(),
        }
    }
}
