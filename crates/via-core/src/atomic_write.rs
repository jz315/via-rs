//! Small transactional file-writing helpers used by exporters.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{Error, Result};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Replaces a file only after all new bytes have been written and synced.
///
/// The temporary and backup files are created beside the destination, so the
/// final rename remains on the same filesystem. If replacement fails after an
/// existing file was moved aside, the previous file is restored.
pub fn atomic_write(path: impl AsRef<Path>, bytes: impl AsRef<[u8]>) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }

    let staged = sibling_path(path, "write");
    let backup = sibling_path(path, "backup");
    let mut stage_guard = FileCleanup::new(staged.clone());
    {
        let mut file = std::fs::File::create(&staged)?;
        file.write_all(bytes.as_ref())?;
        file.sync_all()?;
    }

    let had_existing = path.exists();
    if had_existing {
        std::fs::rename(path, &backup)?;
    }
    if let Err(write_error) = std::fs::rename(&staged, path) {
        if had_existing && let Err(restore_error) = std::fs::rename(&backup, path) {
            return Err(Error::Io(format!(
                "failed to replace {}: {write_error}; also failed to restore {}: {restore_error}",
                path.display(),
                backup.display()
            )));
        }
        return Err(Error::from(write_error));
    }
    stage_guard.disarm();

    if had_existing {
        std::fs::remove_file(&backup)?;
    }
    Ok(())
}

fn sibling_path(path: &Path, label: &str) -> PathBuf {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("via-artifact");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
    parent.join(format!(
        ".{name}.{label}-{}-{timestamp}-{sequence}",
        std::process::id()
    ))
}

struct FileCleanup {
    path: PathBuf,
    armed: bool,
}

impl FileCleanup {
    fn new(path: PathBuf) -> Self {
        Self { path, armed: true }
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for FileCleanup {
    fn drop(&mut self) {
        if self.armed {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_existing_file_without_leaving_temp_artifacts() {
        let root = std::env::temp_dir().join(format!(
            "via_atomic_write_{}_{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path = root.join("artifact.txt");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(&path, "old").unwrap();

        atomic_write(&path, "new").unwrap();

        assert_eq!(std::fs::read_to_string(&path).unwrap(), "new");
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 1);
        std::fs::remove_dir_all(root).unwrap();
    }
}
