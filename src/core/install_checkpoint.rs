//! Temporary previous-installation backup, separate from vanilla originals.
//! Kept on disk until commit so interrupted/failed reinstalls remain recoverable.
use crate::core::journal::{self, ActiveManifest};
use serde::{Deserialize, Serialize};
use std::{fs, io, path::{Component, Path, PathBuf}};

const MARKER: &str = "pending-switch.json";

#[derive(Serialize, Deserialize)]
struct Checkpoint {
    directory: String,
    previous: ActiveManifest,
    files: Vec<String>,
    directories: Vec<String>,
    missing: Vec<String>,
}

pub fn is_pending(game: &Path) -> bool {
    journal::backup_dir(game).join(MARKER).exists()
}

fn relative_path(rel: &str) -> io::Result<&Path> {
    let path = Path::new(rel);
    if rel.is_empty() || path.components().any(|c| !matches!(c, Component::Normal(_))) {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid checkpoint path"));
    }
    Ok(path)
}

// Refuse symlinks/junctions in managed paths rather than traversing outside
// the game while saving or restoring the previous installation.
fn target_path(game: &Path, rel: &str) -> io::Result<PathBuf> {
    let mut path = game.to_path_buf();
    for part in relative_path(rel)?.components() {
        path.push(part);
        match fs::symlink_metadata(&path) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Checkpoint target is a symlink"));
            }
            Err(e) if e.kind() != io::ErrorKind::NotFound => return Err(e),
            _ => {}
        }
    }
    Ok(path)
}

pub fn begin(game: &Path) -> io::Result<bool> {
    if is_pending(game) {
        return Err(io::Error::new(io::ErrorKind::AlreadyExists,
            "An interrupted installation needs recovery. Use Restore originals before installing again."));
    }
    let Some(previous) = journal::read_manifest_checked(game)? else { return Ok(false); };
    if previous.deployment_in_progress {
        return Err(io::Error::new(io::ErrorKind::InvalidData,
            "An incomplete installation needs Restore originals before installing again."));
    }
    let bdir = journal::backup_dir(game);
    let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let directory = format!("switch-{}-{stamp}", std::process::id());
    let root = bdir.join(&directory);
    fs::create_dir_all(root.join("files"))?;
    let mut checkpoint = Checkpoint {
        directory, previous, files: Vec::new(), directories: Vec::new(), missing: Vec::new(),
    };
    let result = (|| {
        let paths: Vec<String> = checkpoint.previous.added.iter().cloned()
            .chain(checkpoint.previous.replaced.iter().map(|item| item.rel.clone()))
            .chain(checkpoint.previous.added_dirs.iter().cloned()).collect();
        for rel in paths { checkpoint.capture(game, &root, &rel)?; }
        // Validate vanilla backups before any existing installation is changed.
        for item in &checkpoint.previous.replaced {
            let path = bdir.join(checkpoint.previous.backup_prefix.as_deref().unwrap_or(""))
                .join(relative_path(&item.rel)?);
            if !path.is_file() {
                return Err(io::Error::new(io::ErrorKind::NotFound, "Original backup is missing"));
            }
        }
        let temporary = root.join("pending.json");
        fs::write(&temporary, serde_json::to_vec_pretty(&checkpoint)?)?;
        fs::OpenOptions::new().write(true).open(&temporary)?.sync_all()?;
        fs::rename(&temporary, bdir.join(MARKER))
    })();
    if result.is_err() { let _ = fs::remove_dir_all(&root); }
    result.map(|_| true)
}

impl Checkpoint {
    fn capture(&mut self, game: &Path, root: &Path, rel: &str) -> io::Result<()> {
        if self.files.iter().chain(&self.directories).chain(&self.missing).any(|p| p == rel) {
            return Ok(());
        }
        let path = target_path(game, rel)?;
        let meta = match fs::symlink_metadata(&path) {
            Ok(meta) => meta,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                self.missing.push(rel.into());
                return Ok(());
            }
            Err(e) => return Err(e),
        };
        if meta.is_dir() {
            self.directories.push(rel.into());
            for entry in fs::read_dir(&path)? {
                let child = Path::new(rel).join(entry?.file_name());
                self.capture(game, root, &child.to_string_lossy())?;
            }
        } else if meta.is_file() {
            let backup = root.join("files").join(rel);
            fs::create_dir_all(backup.parent().unwrap())?;
            fs::copy(&path, &backup)?;
            fs::OpenOptions::new().write(true).open(&backup)?.sync_all()?;
            self.files.push(rel.into());
        } else {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Unsupported checkpoint file type"));
        }
        Ok(())
    }
}

pub fn recover(game: &Path) -> io::Result<()> {
    let bdir = journal::backup_dir(game);
    let checkpoint: Checkpoint = serde_json::from_slice(&fs::read(bdir.join(MARKER))?)?;
    let exe = checkpoint.previous.game_exe.as_ref()
        .or_else(|| checkpoint.previous.game.as_ref().and_then(|g| g.exe.as_ref()))
        .map(|rel| game.join(rel));
    crate::core::install_guards::assert_game_closed(game, exe.as_deref())
        .map_err(|e| io::Error::new(io::ErrorKind::PermissionDenied, e))?;
    let root = bdir.join(relative_path(&checkpoint.directory)?);
    // Check the complete checkpoint before removing the failed installation.
    for rel in &checkpoint.files {
        target_path(game, rel)?;
        if !root.join("files").join(relative_path(rel)?).is_file() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Previous-installation backup is missing"));
        }
    }
    if let Some(current) = journal::read_manifest_checked(game)? {
        if current.backup_prefix != checkpoint.previous.backup_prefix {
            journal::restore_failed_install(game)?;
        }
    }
    for rel in &checkpoint.directories { fs::create_dir_all(target_path(game, rel)?)?; }
    for rel in &checkpoint.files {
        let target = target_path(game, rel)?;
        fs::create_dir_all(target.parent().unwrap())?;
        fs::copy(root.join("files").join(rel), target)?;
    }
    for rel in &checkpoint.missing {
        let target = target_path(game, rel)?;
        if target.is_file() { fs::remove_file(target)?; }
        else if target.is_dir() { fs::remove_dir(target)?; }
    }
    journal::save_manifest(game, &checkpoint.previous)?;
    finish(game)
}

pub fn finish(game: &Path) -> io::Result<()> {
    let bdir = journal::backup_dir(game);
    let marker = bdir.join(MARKER);
    if !marker.exists() { return Ok(()); }
    let checkpoint: Checkpoint = serde_json::from_slice(&fs::read(&marker)?)?;
    let root = bdir.join(relative_path(&checkpoint.directory)?);
    // Removing the marker commits the transaction; leftover snapshot data is
    // harmless if cleanup fails. Never report a failed install after commit.
    fs::remove_file(marker)?;
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{framegen::FrameGenBackend, journal::ManifestItem};

    fn fixture() -> PathBuf {
        let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let game = std::env::temp_dir().join(format!("sm86-checkpoint-{stamp}"));
        fs::create_dir_all(game.join("_DLSS5_Backup/originals/old")).unwrap();
        fs::write(game.join("_DLSS5_Backup/originals/old/game.dll"), b"vanilla").unwrap();
        fs::write(game.join("game.dll"), b"previous renderer").unwrap();
        fs::write(game.join("managed.ini"), b"previous user settings").unwrap();
        fs::create_dir_all(game.join("shaders/empty")).unwrap();
        fs::write(game.join("shaders/custom.fx"), b"user shader").unwrap();
        journal::save_manifest(&game, &ActiveManifest {
            backup_prefix: Some("originals/old".into()),
            frame_gen_backend: Some(FrameGenBackend::DlssgSm86),
            replaced: vec![ManifestItem { rel: "game.dll".into(), ..Default::default() }],
            added: vec!["managed.ini".into(), "already-absent.cfg".into()],
            added_dirs: vec!["shaders".into()],
            ..Default::default()
        }).unwrap();
        game
    }

    #[test]
    fn failed_reinstall_restores_previous_files_settings_and_manifest() {
        let game = fixture();
        let before = fs::read(journal::backup_dir(&game).join("manifest.json")).unwrap();
        assert!(begin(&game).unwrap());
        assert!(begin(&game).is_err(), "pending recovery must block another install");
        let mut current = journal::read_manifest(&game).unwrap();
        current.backup_prefix = Some("originals/new".into());
        current.deployment_in_progress = true;
        current.added.push("new.cfg".into());
        fs::create_dir_all(game.join("_DLSS5_Backup/originals/new")).unwrap();
        fs::write(game.join("_DLSS5_Backup/originals/new/game.dll"), b"vanilla").unwrap();
        journal::save_manifest(&game, &current).unwrap();
        fs::write(game.join("game.dll"), b"partial renderer").unwrap();
        fs::write(game.join("managed.ini"), b"partial new settings").unwrap();
        fs::write(game.join("new.cfg"), b"new file").unwrap();
        fs::remove_dir_all(game.join("shaders")).unwrap();
        recover(&game).unwrap();
        assert_eq!(fs::read(game.join("game.dll")).unwrap(), b"previous renderer");
        assert_eq!(fs::read(game.join("managed.ini")).unwrap(), b"previous user settings");
        assert_eq!(fs::read(game.join("shaders/custom.fx")).unwrap(), b"user shader");
        assert!(game.join("shaders/empty").is_dir());
        assert!(!game.join("new.cfg").exists());
        assert!(!game.join("already-absent.cfg").exists());
        assert_eq!(fs::read(journal::backup_dir(&game).join("manifest.json")).unwrap(), before);
        assert!(!is_pending(&game));
        journal::restore_game(&game).unwrap();
        assert_eq!(fs::read(game.join("game.dll")).unwrap(), b"vanilla");
        fs::remove_dir_all(game).unwrap();
    }

    #[test]
    fn interrupted_switch_is_recoverable_through_restore_originals() {
        let game = fixture();
        begin(&game).unwrap();
        // Simulate interruption in route cleanup, before a new manifest exists.
        fs::remove_file(game.join("managed.ini")).unwrap();
        journal::restore_game(&game).unwrap();
        assert_eq!(fs::read(game.join("game.dll")).unwrap(), b"vanilla");
        assert!(!game.join("managed.ini").exists());
        assert!(!journal::has_backup_available(&game));
        fs::remove_dir_all(game).unwrap();
    }

    #[test]
    fn missing_checkpoint_keeps_journal_and_current_files_for_retry() {
        let game = fixture();
        begin(&game).unwrap();
        let marker = journal::backup_dir(&game).join(MARKER);
        let checkpoint: Checkpoint = serde_json::from_slice(&fs::read(&marker).unwrap()).unwrap();
        let saved = journal::backup_dir(&game).join(checkpoint.directory).join("files/managed.ini");
        fs::remove_file(&saved).unwrap();
        assert!(recover(&game).is_err());
        assert!(marker.exists());
        assert_eq!(fs::read(game.join("game.dll")).unwrap(), b"previous renderer");
        fs::write(saved, b"previous user settings").unwrap();
        recover(&game).unwrap();
        fs::remove_dir_all(game).unwrap();
    }

    #[test]
    fn corrupt_journal_blocks_checkpoint_creation_and_recovery() {
        let game = fixture();
        let manifest = journal::backup_dir(&game).join("manifest.json");
        let valid = fs::read(&manifest).unwrap();
        fs::write(&manifest, b"{broken").unwrap();
        assert!(begin(&game).is_err());
        assert!(!is_pending(&game));
        fs::write(&manifest, &valid).unwrap();
        begin(&game).unwrap();
        fs::write(&manifest, b"{broken").unwrap();
        fs::write(game.join("managed.ini"), b"current contents").unwrap();
        assert!(recover(&game).is_err());
        assert!(is_pending(&game));
        assert_eq!(fs::read(game.join("managed.ini")).unwrap(), b"current contents");
        fs::write(&manifest, valid).unwrap();
        recover(&game).unwrap();
        fs::remove_dir_all(game).unwrap();
    }
}
