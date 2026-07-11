use std::fmt;
use std::fs::File;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const DEFAULT_KICAD_FOOTPRINTS_VERSION: &str = "10.0.4";
pub const VIA_KICAD_FOOTPRINTS_DIR_ENV: &str = "VIA_KICAD_FOOTPRINTS_DIR";
pub const VIA_KICAD_FOOTPRINTS_URL_ENV: &str = "VIA_KICAD_FOOTPRINTS_URL";
pub const DEFAULT_KICAD_FOOTPRINTS_RELEASE_BASE_URL: &str =
    "https://github.com/jz315/via-rs/releases/download";
const MANIFEST_SCHEMA: &str = "via-kicad-footprints-manifest-v1";
const MAX_CACHE_ARCHIVE_ENTRIES: usize = 100_000;
const MAX_CACHE_ARCHIVE_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const KICAD_FOOTPRINT_LICENSE: &[u8] = include_bytes!("../KICAD_FOOTPRINT_LICENSE.md");
const THIRD_PARTY_NOTICES: &[u8] = include_bytes!("../THIRD_PARTY_NOTICES.md");
static TEMP_DIR_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Json(serde_json::Error),
    Http(String),
    CacheMissing {
        version: String,
        path: PathBuf,
    },
    VersionMismatch {
        requested: String,
        manifest: String,
    },
    UnsupportedSchema {
        schema: String,
    },
    UnsafeVersion {
        version: String,
    },
    MissingFootprint {
        library: String,
        name: String,
        version: String,
    },
    UnsafeManifestPath {
        path: String,
    },
    UnsafeFootprintName {
        name: String,
    },
    InvalidImport(String),
    Sha256Mismatch {
        path: PathBuf,
        expected: String,
        actual: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "{err}"),
            Error::Json(err) => write!(f, "{err}"),
            Error::Http(err) => write!(f, "{err}"),
            Error::CacheMissing { version, path } => write!(
                f,
                "KiCad footprint cache {version} is missing at {}; run `via footprints install --version {version}` or `via footprints import --version {version} --from <KiCad footprints dir>`",
                path.display()
            ),
            Error::VersionMismatch {
                requested,
                manifest,
            } => write!(
                f,
                "KiCad footprint cache version mismatch: requested {requested}, manifest is {manifest}"
            ),
            Error::UnsupportedSchema { schema } => write!(
                f,
                "unsupported KiCad footprint manifest schema {schema:?}; expected {MANIFEST_SCHEMA:?}"
            ),
            Error::UnsafeVersion { version } => write!(
                f,
                "KiCad footprint version is not a safe path segment: {version:?}; use only ASCII letters, digits, '.', '-', '_', or '+'"
            ),
            Error::MissingFootprint {
                library,
                name,
                version,
            } => write!(
                f,
                "KiCad footprint {library}:{name} is missing from footprint cache {version}"
            ),
            Error::UnsafeManifestPath { path } => {
                write!(f, "manifest footprint path is not safe: {path}")
            }
            Error::UnsafeFootprintName { name } => {
                write!(f, "footprint name is not safe as a file name: {name}")
            }
            Error::InvalidImport(message) => write!(f, "invalid KiCad footprint import: {message}"),
            Error::Sha256Mismatch {
                path,
                expected,
                actual,
            } => write!(
                f,
                "SHA256 mismatch for {}: expected {expected}, got {actual}",
                path.display()
            ),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FootprintId {
    pub library: String,
    pub name: String,
}

impl FootprintId {
    pub fn new(library: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            library: library.into(),
            name: name.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub schema: String,
    pub version: String,
    pub upstream: ManifestUpstream,
    pub footprints: Vec<ManifestFootprint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestUpstream {
    pub project: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestFootprint {
    pub library: String,
    pub name: String,
    pub path: String,
    pub sha256: String,
}

impl Manifest {
    pub fn read(path: impl AsRef<Path>) -> Result<Self> {
        let text = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&text)?)
    }

    pub fn write(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serde_json::to_string_pretty(self)? + "\n")?;
        Ok(())
    }

    pub fn find(&self, id: &FootprintId) -> Option<&ManifestFootprint> {
        self.footprints
            .iter()
            .find(|entry| entry.library == id.library && entry.name == id.name)
    }
}

#[derive(Debug, Clone)]
pub struct FootprintCache {
    root: PathBuf,
    version: String,
    manifest: Manifest,
}

impl FootprintCache {
    pub fn open(version: impl Into<String>) -> Result<Self> {
        let version = version.into();
        let root = cache_dir_for_version(&version)?;
        Self::open_at(version, root)
    }

    pub fn open_at(version: impl Into<String>, root: impl Into<PathBuf>) -> Result<Self> {
        let version = version.into();
        validate_cache_version(&version)?;
        let root = root.into();
        let manifest_path = root.join("manifest.json");
        if !manifest_path.exists() {
            return Err(Error::CacheMissing {
                version,
                path: root,
            });
        }
        let manifest = Manifest::read(&manifest_path)?;
        if manifest.schema != MANIFEST_SCHEMA {
            return Err(Error::UnsupportedSchema {
                schema: manifest.schema,
            });
        }
        if manifest.version != version {
            return Err(Error::VersionMismatch {
                requested: version,
                manifest: manifest.version,
            });
        }
        Ok(Self {
            root,
            version: manifest.version.clone(),
            manifest,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    pub fn footprint_path(&self, id: &FootprintId) -> Result<PathBuf> {
        let entry = self
            .manifest
            .find(id)
            .ok_or_else(|| Error::MissingFootprint {
                library: id.library.clone(),
                name: id.name.clone(),
                version: self.version.clone(),
            })?;
        let relative = safe_relative_path(&entry.path)?;
        let path = self.root.join(relative);
        verify_sha256_file(&path, &entry.sha256)?;
        Ok(path)
    }

    pub fn footprint_text(&self, id: &FootprintId) -> Result<String> {
        let path = self.footprint_path(id)?;
        Ok(std::fs::read_to_string(path)?)
    }

    pub fn copy_footprint_to_pretty_dir(
        &self,
        id: &FootprintId,
        pretty_dir: impl AsRef<Path>,
    ) -> Result<PathBuf> {
        let source = self.footprint_path(id)?;
        let pretty_dir = pretty_dir.as_ref();
        std::fs::create_dir_all(pretty_dir)?;
        let destination = pretty_dir.join(footprint_file_name(&id.name)?);
        std::fs::copy(source, &destination)?;
        Ok(destination)
    }

    pub fn validate_all(&self) -> Result<usize> {
        for entry in &self.manifest.footprints {
            let relative = safe_relative_path(&entry.path)?;
            verify_sha256_file(self.root.join(relative), &entry.sha256)?;
        }
        Ok(self.manifest.footprints.len())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheStatus {
    pub version: String,
    pub root: PathBuf,
    pub manifest_exists: bool,
    pub footprint_count: usize,
    pub validation_error: Option<String>,
}

impl CacheStatus {
    /// Returns whether the cache manifest and every footprint hash validate.
    pub fn is_ready(&self) -> bool {
        self.manifest_exists && self.validation_error.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleReport {
    pub version: String,
    pub root: PathBuf,
    pub output: PathBuf,
    pub footprint_count: usize,
}

/// Options for importing a local official KiCad footprint checkout.
#[derive(Debug, Clone)]
pub struct ImportOptions {
    source: PathBuf,
    version: String,
    cache_dir: Option<PathBuf>,
    upstream_source: Option<String>,
}

impl ImportOptions {
    /// Starts an import using a local KiCad footprint directory and version.
    pub fn new(source: impl Into<PathBuf>, version: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            version: version.into(),
            cache_dir: None,
            upstream_source: None,
        }
    }

    /// Overrides the destination cache directory.
    pub fn cache_dir(mut self, cache_dir: impl Into<PathBuf>) -> Self {
        self.cache_dir = Some(cache_dir.into());
        self
    }

    /// Records the archive URL or revision used to obtain the source tree.
    pub fn upstream_source(mut self, upstream_source: impl Into<String>) -> Self {
        self.upstream_source = Some(upstream_source.into());
        self
    }
}

/// Options for creating a deterministic release bundle from an installed cache.
#[derive(Debug, Clone)]
pub struct BundleOptions {
    version: String,
    cache_dir: Option<PathBuf>,
    output: PathBuf,
}

impl BundleOptions {
    /// Starts a bundle request for one installed cache version.
    pub fn new(version: impl Into<String>, output: impl Into<PathBuf>) -> Self {
        Self {
            version: version.into(),
            cache_dir: None,
            output: output.into(),
        }
    }

    /// Overrides the cache directory used as bundle input.
    pub fn cache_dir(mut self, cache_dir: impl Into<PathBuf>) -> Self {
        self.cache_dir = Some(cache_dir.into());
        self
    }
}

pub fn cache_status(version: impl Into<String>) -> Result<CacheStatus> {
    let version = version.into();
    let root = cache_dir_for_version(&version)?;
    let manifest_path = root.join("manifest.json");
    if !manifest_path.exists() {
        return Ok(CacheStatus {
            version,
            root,
            manifest_exists: false,
            footprint_count: 0,
            validation_error: None,
        });
    }
    match FootprintCache::open_at(&version, &root) {
        Ok(cache) => match cache.validate_all() {
            Ok(count) => Ok(CacheStatus {
                version,
                root,
                manifest_exists: true,
                footprint_count: count,
                validation_error: None,
            }),
            Err(err) => Ok(CacheStatus {
                version,
                root,
                manifest_exists: true,
                footprint_count: cache.manifest().footprints.len(),
                validation_error: Some(err.to_string()),
            }),
        },
        Err(err) => Ok(CacheStatus {
            version,
            root,
            manifest_exists: true,
            footprint_count: 0,
            validation_error: Some(err.to_string()),
        }),
    }
}

pub fn validate_cache_version(version: &str) -> Result<()> {
    let safe = !version.is_empty()
        && !matches!(version, "." | "..")
        && version
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b'+'));
    if !safe {
        return Err(Error::UnsafeVersion {
            version: version.to_owned(),
        });
    }
    Ok(())
}

pub fn cache_dir_for_version(version: &str) -> Result<PathBuf> {
    validate_cache_version(version)?;
    if let Ok(path) = std::env::var(VIA_KICAD_FOOTPRINTS_DIR_ENV)
        && !path.trim().is_empty()
    {
        return Ok(PathBuf::from(path));
    }

    if let Ok(path) = std::env::var("LOCALAPPDATA")
        && !path.trim().is_empty()
    {
        return Ok(PathBuf::from(path)
            .join("via")
            .join("kicad-footprints")
            .join(version));
    }

    if let Ok(path) = std::env::var("XDG_CACHE_HOME")
        && !path.trim().is_empty()
    {
        return Ok(PathBuf::from(path)
            .join("via")
            .join("kicad-footprints")
            .join(version));
    }

    Ok(home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".cache")
        .join("via")
        .join("kicad-footprints")
        .join(version))
}

pub fn footprint_file_name(name: &str) -> Result<String> {
    let unsafe_name = name.trim().is_empty()
        || matches!(name, "." | "..")
        || name.chars().any(|ch| {
            ch.is_control() || matches!(ch, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|')
        });
    if unsafe_name {
        return Err(Error::UnsafeFootprintName {
            name: name.to_owned(),
        });
    }
    Ok(format!("{name}.kicad_mod"))
}

pub fn cache_bundle_file_name(version: &str) -> String {
    format!("kicad-footprints-{version}.tar.zst")
}

pub fn cache_bundle_release_tag(version: &str) -> String {
    format!("kicad-footprints-{version}")
}

pub fn default_cache_bundle_url(version: &str) -> String {
    format!(
        "{}/{}/{}",
        DEFAULT_KICAD_FOOTPRINTS_RELEASE_BASE_URL,
        cache_bundle_release_tag(version),
        cache_bundle_file_name(version)
    )
}

pub fn import_from_kicad_dir(
    source: impl AsRef<Path>,
    version: impl Into<String>,
    cache_dir: Option<PathBuf>,
) -> Result<Manifest> {
    import_from_kicad_dir_with_source(source, version, cache_dir, None)
}

pub fn import_from_kicad_dir_with_source(
    source: impl AsRef<Path>,
    version: impl Into<String>,
    cache_dir: Option<PathBuf>,
    upstream_source: Option<String>,
) -> Result<Manifest> {
    let mut options = ImportOptions::new(source.as_ref().to_path_buf(), version);
    if let Some(cache_dir) = cache_dir {
        options = options.cache_dir(cache_dir);
    }
    if let Some(upstream_source) = upstream_source {
        options = options.upstream_source(upstream_source);
    }
    import(options)
}

/// Imports an official KiCad footprint checkout transactionally.
pub fn import(options: ImportOptions) -> Result<Manifest> {
    let source = options.source;
    let version = options.version;
    validate_cache_version(&version)?;
    let root = match options.cache_dir {
        Some(cache_dir) => cache_dir,
        None => cache_dir_for_version(&version)?,
    };
    if !source.is_dir() {
        return Err(Error::InvalidImport(format!(
            "{} is not a directory",
            source.display()
        )));
    }
    let parent = root
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent)?;
    let stage = unique_sibling_path(&root, "import");
    std::fs::create_dir(&stage)?;
    let stage_guard = DirectoryCleanup::new(stage.clone());
    let result = (|| {
        let manifest = import_into_stage(&source, &version, &stage, options.upstream_source)?;
        let cache = FootprintCache::open_at(&version, &stage)?;
        cache.validate_all()?;
        install_staged_cache(&stage, &root)?;
        Ok(manifest)
    })();
    drop(stage_guard);
    result
}

fn import_into_stage(
    source: &Path,
    version: &str,
    root: &Path,
    upstream_source: Option<String>,
) -> Result<Manifest> {
    let mut footprints = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let library_dir = entry.path();
        let metadata = std::fs::symlink_metadata(&library_dir)?;
        if !metadata.file_type().is_dir()
            || library_dir.extension().and_then(|ext| ext.to_str()) != Some("pretty")
        {
            continue;
        }
        let Some(library_stem) = library_dir.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        let destination_library = root.join(format!("{library_stem}.pretty"));
        std::fs::create_dir_all(&destination_library)?;
        for footprint in std::fs::read_dir(&library_dir)? {
            let footprint = footprint?;
            let source_file = footprint.path();
            let metadata = std::fs::symlink_metadata(&source_file)?;
            if !metadata.file_type().is_file()
                || source_file.extension().and_then(|ext| ext.to_str()) != Some("kicad_mod")
            {
                continue;
            }
            let Some(name) = source_file.file_stem().and_then(|stem| stem.to_str()) else {
                continue;
            };
            let file_name = footprint_file_name(name)?;
            if !seen.insert((library_stem.to_owned(), name.to_owned())) {
                return Err(Error::InvalidImport(format!(
                    "duplicate footprint {library_stem}:{name}"
                )));
            }
            let destination_file = destination_library.join(file_name);
            std::fs::copy(&source_file, &destination_file)?;
            let relative_path = format!("{library_stem}.pretty/{name}.kicad_mod");
            footprints.push(ManifestFootprint {
                library: library_stem.to_owned(),
                name: name.to_owned(),
                path: relative_path,
                sha256: sha256_file(&destination_file)?,
            });
        }
    }

    footprints.sort_by(|a, b| a.library.cmp(&b.library).then_with(|| a.name.cmp(&b.name)));
    if footprints.is_empty() {
        return Err(Error::InvalidImport(
            "no regular .kicad_mod files were found below *.pretty libraries".to_owned(),
        ));
    }

    let manifest = Manifest {
        schema: MANIFEST_SCHEMA.to_owned(),
        version: version.to_owned(),
        upstream: ManifestUpstream {
            project: "KiCad official kicad-footprints".to_owned(),
            version: version.to_owned(),
            repository: Some("https://gitlab.com/kicad/libraries/kicad-footprints".to_owned()),
            commit: None,
            source: Some(upstream_source.unwrap_or_else(|| source.display().to_string())),
        },
        footprints,
    };
    manifest.write(root.join("manifest.json"))?;
    Ok(manifest)
}

pub fn fetch_from_url(
    url: &str,
    version: impl Into<String>,
    cache_dir: Option<PathBuf>,
) -> Result<Manifest> {
    let version = version.into();
    validate_cache_version(&version)?;
    let root = match cache_dir {
        Some(cache_dir) => cache_dir,
        None => cache_dir_for_version(&version)?,
    };
    let response = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(30))
        .timeout_read(Duration::from_secs(120))
        .build()
        .get(url)
        .call()
        .map_err(|err| Error::Http(format!("failed to download {url}: {err}")))?;
    let decoder = zstd::stream::read::Decoder::new(response.into_reader())?;
    install_cache_archive(decoder, &version, &root)
}

pub fn bundle_cache_archive(
    version: impl Into<String>,
    cache_dir: Option<PathBuf>,
    output: impl AsRef<Path>,
) -> Result<BundleReport> {
    let mut options = BundleOptions::new(version, output.as_ref().to_path_buf());
    if let Some(cache_dir) = cache_dir {
        options = options.cache_dir(cache_dir);
    }
    bundle(options)
}

/// Creates a deterministic, atomically-replaced release bundle.
pub fn bundle(options: BundleOptions) -> Result<BundleReport> {
    let version = options.version;
    validate_cache_version(&version)?;
    let root = match options.cache_dir {
        Some(cache_dir) => cache_dir,
        None => cache_dir_for_version(&version)?,
    };
    let cache = FootprintCache::open_at(&version, &root)?;
    cache.validate_all()?;

    let output = options.output;
    if let Some(parent) = output.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }
    let staged_output = unique_sibling_path(&output, "bundle");
    let output_guard = FileCleanup::new(staged_output.clone());
    let file = File::create(&staged_output)?;
    let encoder = zstd::stream::write::Encoder::new(file, 19)?;
    let mut archive = tar::Builder::new(encoder);

    append_file_to_archive(&mut archive, &root.join("manifest.json"), "manifest.json")?;

    let mut footprints = cache.manifest().footprints.clone();
    footprints.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then_with(|| a.library.cmp(&b.library))
            .then_with(|| a.name.cmp(&b.name))
    });
    for footprint in &footprints {
        let relative = safe_relative_path(&footprint.path)?;
        let source = root.join(&relative);
        append_file_to_archive(&mut archive, &source, &relative)?;
    }

    append_bytes_to_archive(
        &mut archive,
        Path::new("KICAD_FOOTPRINT_LICENSE.md"),
        KICAD_FOOTPRINT_LICENSE,
    )?;
    append_bytes_to_archive(
        &mut archive,
        Path::new("THIRD_PARTY_NOTICES.md"),
        THIRD_PARTY_NOTICES,
    )?;

    let encoder = archive.into_inner()?;
    encoder.finish()?;
    install_staged_file(&staged_output, &output)?;
    drop(output_guard);
    Ok(BundleReport {
        version,
        root,
        output,
        footprint_count: cache.manifest().footprints.len(),
    })
}

fn append_file_to_archive(
    archive: &mut tar::Builder<zstd::stream::write::Encoder<'_, File>>,
    source: &Path,
    archive_path: impl AsRef<Path>,
) -> Result<()> {
    let metadata = std::fs::symlink_metadata(source)?;
    if !metadata.file_type().is_file() {
        return Err(Error::Http(format!(
            "footprint cache source {} is not a regular file",
            source.display()
        )));
    }
    let file = File::open(source)?;
    append_reader_to_archive(archive, archive_path.as_ref(), metadata.len(), file)
}

fn append_bytes_to_archive(
    archive: &mut tar::Builder<zstd::stream::write::Encoder<'_, File>>,
    archive_path: &Path,
    bytes: &[u8],
) -> Result<()> {
    append_reader_to_archive(archive, archive_path, bytes.len() as u64, bytes)
}

fn append_reader_to_archive(
    archive: &mut tar::Builder<zstd::stream::write::Encoder<'_, File>>,
    archive_path: &Path,
    size: u64,
    reader: impl io::Read,
) -> Result<()> {
    let relative = safe_relative_path(&archive_path.to_string_lossy())?;
    let mut header = tar::Header::new_gnu();
    header.set_entry_type(tar::EntryType::Regular);
    header.set_mode(0o644);
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(0);
    header.set_size(size);
    header.set_cksum();
    archive.append_data(&mut header, relative, reader)?;
    Ok(())
}

fn install_cache_archive(reader: impl io::Read, version: &str, root: &Path) -> Result<Manifest> {
    let parent = root
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent)?;
    let stage = unique_sibling_path(root, "download");
    std::fs::create_dir(&stage)?;
    let stage_guard = DirectoryCleanup::new(stage.clone());

    let result = (|| {
        unpack_cache_archive(reader, &stage)?;

        let cache = FootprintCache::open_at(version, &stage)?;
        cache.validate_all()?;
        let manifest = cache.manifest().clone();
        install_staged_cache(&stage, root)?;
        Ok(manifest)
    })();

    drop(stage_guard);
    result
}

fn unpack_cache_archive(reader: impl io::Read, stage: &Path) -> Result<()> {
    let mut archive = tar::Archive::new(reader);
    let mut entries = 0usize;
    let mut unpacked_bytes = 0u64;
    for entry in archive.entries()? {
        let mut entry = entry?;
        entries += 1;
        if entries > MAX_CACHE_ARCHIVE_ENTRIES {
            return Err(Error::Http(format!(
                "footprint cache archive exceeds the {MAX_CACHE_ARCHIVE_ENTRIES} entry limit"
            )));
        }
        let entry_type = entry.header().entry_type();
        if !entry_type.is_file() && !entry_type.is_dir() {
            return Err(Error::Http(
                "footprint cache archive contains unsupported links or special files".to_owned(),
            ));
        }
        unpacked_bytes = unpacked_bytes
            .checked_add(entry.header().size()?)
            .ok_or_else(|| Error::Http("footprint cache archive size overflow".to_owned()))?;
        if unpacked_bytes > MAX_CACHE_ARCHIVE_BYTES {
            return Err(Error::Http(format!(
                "footprint cache archive exceeds the {MAX_CACHE_ARCHIVE_BYTES} byte unpacked limit"
            )));
        }
        if !entry.unpack_in(stage)? {
            return Err(Error::UnsafeManifestPath {
                path: entry.path()?.display().to_string(),
            });
        }
    }
    Ok(())
}

fn install_staged_cache(stage: &Path, root: &Path) -> Result<()> {
    let backup = unique_sibling_path(root, "backup");
    let had_existing = root.exists();
    if had_existing {
        std::fs::rename(root, &backup)?;
    }

    if let Err(install_error) = std::fs::rename(stage, root) {
        if had_existing && let Err(restore_error) = std::fs::rename(&backup, root) {
            return Err(Error::Http(format!(
                "failed to install footprint cache: {install_error}; also failed to restore previous cache from {}: {restore_error}",
                backup.display()
            )));
        }
        return Err(Error::Io(install_error));
    }

    if had_existing {
        remove_path(&backup)?;
    }
    Ok(())
}

fn install_staged_file(stage: &Path, output: &Path) -> Result<()> {
    let backup = unique_sibling_path(output, "backup");
    let had_existing = output.exists();
    if had_existing {
        std::fs::rename(output, &backup)?;
    }

    if let Err(install_error) = std::fs::rename(stage, output) {
        if had_existing && let Err(restore_error) = std::fs::rename(&backup, output) {
            return Err(Error::Http(format!(
                "failed to replace footprint bundle: {install_error}; also failed to restore previous bundle from {}: {restore_error}",
                backup.display()
            )));
        }
        return Err(Error::Io(install_error));
    }

    if had_existing {
        remove_path(&backup)?;
    }
    Ok(())
}

fn unique_sibling_path(root: &Path, label: &str) -> PathBuf {
    let parent = root
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let name = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("via-kicad-footprints");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let sequence = TEMP_DIR_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    parent.join(format!(
        ".{name}.{label}-{}-{timestamp}-{sequence}",
        std::process::id()
    ))
}

fn remove_path(path: &Path) -> io::Result<()> {
    if path.is_dir() {
        std::fs::remove_dir_all(path)
    } else if path.exists() {
        std::fs::remove_file(path)
    } else {
        Ok(())
    }
}

struct DirectoryCleanup {
    path: PathBuf,
}

struct FileCleanup {
    path: PathBuf,
}

impl FileCleanup {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Drop for FileCleanup {
    fn drop(&mut self) {
        let _ = remove_path(&self.path);
    }
}

impl DirectoryCleanup {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Drop for DirectoryCleanup {
    fn drop(&mut self) {
        let _ = remove_path(&self.path);
    }
}

fn safe_relative_path(path: &str) -> Result<PathBuf> {
    let path_buf = PathBuf::from(path);
    if path_buf.is_absolute() {
        return Err(Error::UnsafeManifestPath {
            path: path.to_owned(),
        });
    }
    for component in path_buf.components() {
        if matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        ) {
            return Err(Error::UnsafeManifestPath {
                path: path.to_owned(),
            });
        }
    }
    Ok(path_buf)
}

fn verify_sha256_file(path: impl AsRef<Path>, expected: &str) -> Result<()> {
    let path = path.as_ref();
    let actual = sha256_file(path)?;
    if !actual.eq_ignore_ascii_case(expected) {
        return Err(Error::Sha256Mismatch {
            path: path.to_path_buf(),
            expected: expected.to_owned(),
            actual,
        });
    }
    Ok(())
}

fn sha256_file(path: impl AsRef<Path>) -> Result<String> {
    let bytes = std::fs::read(path)?;
    Ok(sha256_hex(&bytes))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()
        .filter(|path| !path.trim().is_empty())
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn cache_versions_are_safe_single_path_segments() {
        for version in ["10.0.4", "nightly-2026_07+1"] {
            validate_cache_version(version).unwrap();
        }

        for version in [
            "",
            ".",
            "..",
            "../10.0.4",
            "10.0.4/other",
            r"C:\Windows",
            "10.0.4\n",
            "版本-10",
        ] {
            let err = validate_cache_version(version).unwrap_err();
            assert!(matches!(err, Error::UnsafeVersion { .. }), "{version:?}");
        }
    }

    #[test]
    fn explicit_cache_roots_do_not_bypass_version_validation() {
        let err = FootprintCache::open_at(r"C:\Windows", temp_root("unsafe_version")).unwrap_err();
        assert!(matches!(err, Error::UnsafeVersion { .. }));
    }

    #[test]
    fn cache_reads_and_validates_manifest_footprint() {
        let root = temp_root("via_kicad_footprints_cache_test");
        let pretty = root.join("Fixture_Lib.pretty");
        std::fs::create_dir_all(&pretty).unwrap();
        let footprint_text = "(footprint \"Fixture_Footprint\" (pad \"1\") (pad \"2\"))\n";
        let footprint_path = pretty.join("Fixture_Footprint.kicad_mod");
        std::fs::write(&footprint_path, footprint_text).unwrap();
        let manifest = Manifest {
            schema: MANIFEST_SCHEMA.to_owned(),
            version: "10.0.4".to_owned(),
            upstream: ManifestUpstream {
                project: "fixture".to_owned(),
                version: "10.0.4".to_owned(),
                repository: None,
                commit: None,
                source: None,
            },
            footprints: vec![ManifestFootprint {
                library: "Fixture_Lib".to_owned(),
                name: "Fixture_Footprint".to_owned(),
                path: "Fixture_Lib.pretty/Fixture_Footprint.kicad_mod".to_owned(),
                sha256: sha256_hex(footprint_text.as_bytes()),
            }],
        };
        manifest.write(root.join("manifest.json")).unwrap();

        let cache = FootprintCache::open_at("10.0.4", &root).unwrap();
        let text = cache
            .footprint_text(&FootprintId::new("Fixture_Lib", "Fixture_Footprint"))
            .unwrap();

        assert_eq!(text, footprint_text);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn invalid_import_leaves_existing_cache_untouched() {
        let root = temp_root("via_kicad_footprints_import_existing");
        let pretty = root.join("Fixture_Lib.pretty");
        std::fs::create_dir_all(&pretty).unwrap();
        let footprint_path = pretty.join("Fixture_Footprint.kicad_mod");
        std::fs::write(&footprint_path, "old footprint").unwrap();
        Manifest {
            schema: MANIFEST_SCHEMA.to_owned(),
            version: "10.0.4".to_owned(),
            upstream: ManifestUpstream {
                project: "fixture".to_owned(),
                version: "10.0.4".to_owned(),
                repository: None,
                commit: None,
                source: None,
            },
            footprints: vec![ManifestFootprint {
                library: "Fixture_Lib".to_owned(),
                name: "Fixture_Footprint".to_owned(),
                path: "Fixture_Lib.pretty/Fixture_Footprint.kicad_mod".to_owned(),
                sha256: sha256_hex(b"old footprint"),
            }],
        }
        .write(root.join("manifest.json"))
        .unwrap();

        let empty_source = temp_root("via_kicad_footprints_empty_import");
        std::fs::create_dir_all(&empty_source).unwrap();
        let err = import(ImportOptions::new(&empty_source, "10.0.4").cache_dir(&root)).unwrap_err();
        assert!(matches!(err, Error::InvalidImport(_)));
        assert_eq!(
            std::fs::read_to_string(footprint_path).unwrap(),
            "old footprint"
        );
        FootprintCache::open_at("10.0.4", &root)
            .unwrap()
            .validate_all()
            .unwrap();

        std::fs::remove_dir_all(root).unwrap();
        std::fs::remove_dir_all(empty_source).unwrap();
    }

    #[test]
    fn rejects_manifest_path_traversal() {
        let err = safe_relative_path("../bad.kicad_mod").unwrap_err();

        assert!(format!("{err}").contains("not safe"));
    }

    #[test]
    fn rejects_unknown_manifest_schema() {
        let root = temp_root("via_kicad_footprints_schema_test");
        std::fs::create_dir_all(&root).unwrap();
        let manifest = Manifest {
            schema: "future-schema".to_owned(),
            version: "10.0.4".to_owned(),
            upstream: ManifestUpstream {
                project: "fixture".to_owned(),
                version: "10.0.4".to_owned(),
                repository: None,
                commit: None,
                source: None,
            },
            footprints: Vec::new(),
        };
        manifest.write(root.join("manifest.json")).unwrap();

        let err = FootprintCache::open_at("10.0.4", &root).unwrap_err();

        assert!(matches!(err, Error::UnsupportedSchema { .. }));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn staged_cache_install_replaces_the_old_cache_and_removes_the_backup() {
        let base = temp_root("via_kicad_footprints_install_test");
        let root = base.join("cache");
        let stage = base.join("stage");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&stage).unwrap();
        std::fs::write(root.join("marker.txt"), "old").unwrap();
        std::fs::write(stage.join("marker.txt"), "new").unwrap();

        install_staged_cache(&stage, &root).unwrap();

        assert_eq!(
            std::fs::read_to_string(root.join("marker.txt")).unwrap(),
            "new"
        );
        assert!(!stage.exists());
        assert_eq!(std::fs::read_dir(&base).unwrap().count(), 1);
        std::fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn invalid_archive_leaves_the_existing_cache_untouched() {
        let base = temp_root("via_kicad_footprints_invalid_archive_test");
        let root = base.join("cache");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("marker.txt"), "old").unwrap();

        let err =
            install_cache_archive(std::io::Cursor::new(b"not a tar archive"), "10.0.4", &root)
                .unwrap_err();

        assert!(matches!(err, Error::Io(_)));
        assert_eq!(
            std::fs::read_to_string(root.join("marker.txt")).unwrap(),
            "old"
        );
        assert_eq!(std::fs::read_dir(&base).unwrap().count(), 1);
        std::fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn rejects_unsafe_footprint_file_names() {
        for name in ["../Bad", r"Bad\Name", "Lib:Bad", ""] {
            let err = footprint_file_name(name).unwrap_err();
            assert!(format!("{err}").contains("not safe"), "{name}");
        }

        assert_eq!(
            footprint_file_name("CP_Radial_D6.3mm_P2.50mm").unwrap(),
            "CP_Radial_D6.3mm_P2.50mm.kicad_mod"
        );
    }

    #[test]
    fn default_bundle_url_uses_versioned_release_asset() {
        assert_eq!(
            default_cache_bundle_url("10.0.4"),
            "https://github.com/jz315/via-rs/releases/download/kicad-footprints-10.0.4/kicad-footprints-10.0.4.tar.zst"
        );
    }

    #[test]
    fn bundles_cache_archive_deterministically_and_installs_roundtrip() {
        let base = temp_root("via_kicad_footprints_bundle_test");
        let source = base.join("source");
        write_fixture_cache(&source);

        let out_a = base.join("bundle-a.tar.zst");
        let out_b = base.join("bundle-b.tar.zst");
        let report_a = bundle_cache_archive("10.0.4", Some(source.clone()), &out_a).unwrap();
        let report_b = bundle_cache_archive("10.0.4", Some(source.clone()), &out_b).unwrap();

        assert_eq!(report_a.footprint_count, 1);
        assert_eq!(report_b.footprint_count, 1);
        assert_eq!(
            std::fs::read(&out_a).unwrap(),
            std::fs::read(&out_b).unwrap()
        );

        let installed = base.join("installed");
        let file = File::open(&out_a).unwrap();
        let decoder = zstd::stream::read::Decoder::new(file).unwrap();
        let manifest = install_cache_archive(decoder, "10.0.4", &installed).unwrap();
        assert_eq!(manifest.footprints.len(), 1);
        let cache = FootprintCache::open_at("10.0.4", &installed).unwrap();
        assert_eq!(cache.validate_all().unwrap(), 1);
        assert!(installed.join("KICAD_FOOTPRINT_LICENSE.md").is_file());
        assert!(installed.join("THIRD_PARTY_NOTICES.md").is_file());

        std::fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn import_can_record_upstream_source_override() {
        let base = temp_root("via_kicad_footprints_import_source_test");
        let source = base.join("kicad-source");
        let pretty = source.join("Fixture_Lib.pretty");
        std::fs::create_dir_all(&pretty).unwrap();
        std::fs::write(
            pretty.join("Fixture_Footprint.kicad_mod"),
            "(footprint \"Fixture_Footprint\")\n",
        )
        .unwrap();

        let manifest = import_from_kicad_dir_with_source(
            &source,
            "10.0.4",
            Some(base.join("cache")),
            Some("https://example.invalid/kicad-footprints-10.0.4.tar.gz".to_owned()),
        )
        .unwrap();

        assert_eq!(
            manifest.upstream.source.as_deref(),
            Some("https://example.invalid/kicad-footprints-10.0.4.tar.gz")
        );
        std::fs::remove_dir_all(base).unwrap();
    }

    fn write_fixture_cache(root: &Path) {
        let pretty = root.join("Fixture_Lib.pretty");
        std::fs::create_dir_all(&pretty).unwrap();
        let footprint_text = "(footprint \"Fixture_Footprint\" (pad \"1\") (pad \"2\"))\n";
        let footprint_path = pretty.join("Fixture_Footprint.kicad_mod");
        std::fs::write(&footprint_path, footprint_text).unwrap();
        let manifest = Manifest {
            schema: MANIFEST_SCHEMA.to_owned(),
            version: "10.0.4".to_owned(),
            upstream: ManifestUpstream {
                project: "fixture".to_owned(),
                version: "10.0.4".to_owned(),
                repository: None,
                commit: None,
                source: None,
            },
            footprints: vec![ManifestFootprint {
                library: "Fixture_Lib".to_owned(),
                name: "Fixture_Footprint".to_owned(),
                path: "Fixture_Lib.pretty/Fixture_Footprint.kicad_mod".to_owned(),
                sha256: sha256_hex(footprint_text.as_bytes()),
            }],
        };
        manifest.write(root.join("manifest.json")).unwrap();
    }

    fn temp_root(prefix: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "{}_{}_{}",
            prefix,
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }
}
