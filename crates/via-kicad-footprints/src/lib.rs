use std::fmt;
use std::io;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const DEFAULT_KICAD_FOOTPRINTS_VERSION: &str = "10.0.4";
pub const VIA_KICAD_FOOTPRINTS_DIR_ENV: &str = "VIA_KICAD_FOOTPRINTS_DIR";
pub const VIA_KICAD_FOOTPRINTS_URL_ENV: &str = "VIA_KICAD_FOOTPRINTS_URL";

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
    MissingFootprint {
        library: String,
        name: String,
        version: String,
    },
    UnsafeManifestPath {
        path: String,
    },
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
                "KiCad footprint cache {version} is missing at {}; run `via footprints import --version {version} --from <KiCad footprints dir>` or `via footprints fetch --version {version} --url <release asset URL>`",
                path.display()
            ),
            Error::VersionMismatch {
                requested,
                manifest,
            } => write!(
                f,
                "KiCad footprint cache version mismatch: requested {requested}, manifest is {manifest}"
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
        Self::open_at(version.clone(), cache_dir_for_version(&version))
    }

    pub fn open_at(version: impl Into<String>, root: impl Into<PathBuf>) -> Result<Self> {
        let version = version.into();
        let root = root.into();
        let manifest_path = root.join("manifest.json");
        if !manifest_path.exists() {
            return Err(Error::CacheMissing {
                version,
                path: root,
            });
        }
        let manifest = Manifest::read(&manifest_path)?;
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
        let destination = pretty_dir.join(format!("{}.kicad_mod", id.name));
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
}

pub fn cache_status(version: impl Into<String>) -> Result<CacheStatus> {
    let version = version.into();
    let root = cache_dir_for_version(&version);
    let manifest_path = root.join("manifest.json");
    if !manifest_path.exists() {
        return Ok(CacheStatus {
            version,
            root,
            manifest_exists: false,
            footprint_count: 0,
        });
    }
    let manifest = Manifest::read(manifest_path)?;
    Ok(CacheStatus {
        version,
        root,
        manifest_exists: true,
        footprint_count: manifest.footprints.len(),
    })
}

pub fn cache_dir_for_version(version: &str) -> PathBuf {
    if let Ok(path) = std::env::var(VIA_KICAD_FOOTPRINTS_DIR_ENV)
        && !path.trim().is_empty()
    {
        return PathBuf::from(path);
    }

    if let Ok(path) = std::env::var("LOCALAPPDATA")
        && !path.trim().is_empty()
    {
        return PathBuf::from(path)
            .join("via")
            .join("kicad-footprints")
            .join(version);
    }

    if let Ok(path) = std::env::var("XDG_CACHE_HOME")
        && !path.trim().is_empty()
    {
        return PathBuf::from(path)
            .join("via")
            .join("kicad-footprints")
            .join(version);
    }

    home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".cache")
        .join("via")
        .join("kicad-footprints")
        .join(version)
}

pub fn import_from_kicad_dir(
    source: impl AsRef<Path>,
    version: impl Into<String>,
    cache_dir: Option<PathBuf>,
) -> Result<Manifest> {
    let version = version.into();
    let source = source.as_ref();
    let root = cache_dir.unwrap_or_else(|| cache_dir_for_version(&version));
    std::fs::create_dir_all(&root)?;

    let mut footprints = Vec::new();
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let library_dir = entry.path();
        if !library_dir.is_dir()
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
            if source_file.extension().and_then(|ext| ext.to_str()) != Some("kicad_mod") {
                continue;
            }
            let Some(name) = source_file.file_stem().and_then(|stem| stem.to_str()) else {
                continue;
            };
            let destination_file = destination_library.join(format!("{name}.kicad_mod"));
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

    let manifest = Manifest {
        schema: "via-kicad-footprints-manifest-v1".to_owned(),
        version: version.clone(),
        upstream: ManifestUpstream {
            project: "KiCad official kicad-footprints".to_owned(),
            version,
            repository: Some("https://gitlab.com/kicad/libraries/kicad-footprints".to_owned()),
            commit: None,
            source: Some(source.display().to_string()),
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
    let root = cache_dir.unwrap_or_else(|| cache_dir_for_version(&version));
    std::fs::create_dir_all(&root)?;

    let response = ureq::get(url)
        .call()
        .map_err(|err| Error::Http(format!("failed to download {url}: {err}")))?;
    let decoder = zstd::stream::read::Decoder::new(response.into_reader())?;
    let mut archive = tar::Archive::new(decoder);
    archive.unpack(&root)?;

    let cache = FootprintCache::open_at(version, root)?;
    cache.validate_all()?;
    Ok(cache.manifest().clone())
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
    fn cache_reads_and_validates_manifest_footprint() {
        let root = temp_root("via_kicad_footprints_cache_test");
        let pretty = root.join("Fixture_Lib.pretty");
        std::fs::create_dir_all(&pretty).unwrap();
        let footprint_text = "(footprint \"Fixture_Footprint\" (pad \"1\") (pad \"2\"))\n";
        let footprint_path = pretty.join("Fixture_Footprint.kicad_mod");
        std::fs::write(&footprint_path, footprint_text).unwrap();
        let manifest = Manifest {
            schema: "via-kicad-footprints-manifest-v1".to_owned(),
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
    fn rejects_manifest_path_traversal() {
        let err = safe_relative_path("../bad.kicad_mod").unwrap_err();

        assert!(format!("{err}").contains("not safe"));
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
