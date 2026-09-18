//! External, pinned dlssg_for_sm86 payload. No upstream code or binaries are
//! linked into Studio. Hashes cover the signed 310.9 DLLs at RELEASE_COMMIT.
use crate::core::{downloader, journal, optiscaler::set_ini};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub const RELEASE_VERSION: &str = "0.3.3";
pub const RELEASE_COMMIT: &str = "5e79459c2d521f8c3276ce9ee02342c0e2686982";
pub const INI_NAME: &str = "dlssg_sm86.ini";
pub const NOTICE_NAME: &str = "dlssg_sm86_THIRD_PARTY_NOTICES.txt";

// (installed name, upstream path, SHA-256). These are distinct forwarding
// builds: do not rename version.dll to manufacture the other three proxies.
pub const PROXIES: [(&str, &str, &str); 4] = [
    (
        "version.dll",
        "version.dll",
        "3b9ee60894766f2ea810f7aae171b163b923cff7ead71ef7d4eda624470c4642",
    ),
    (
        "winmm.dll",
        "alternatives/winmm.dll",
        "ed8d901000eb509c4d552bfbe0f66f2ee78c65226f703df9d8bf4b1bdb2726cd",
    ),
    (
        "dbghelp.dll",
        "alternatives/dbghelp.dll",
        "880e00cfd631cbd65e4d6490bfd25b894a5b6f40ba460b59b725edbc6e219e5d",
    ),
    (
        "dinput8.dll",
        "alternatives/dinput8.dll",
        "46742058b45e851d4b371fb84fa160ef3eff9677d50b839ec5aa28adf868abab",
    ),
];
const NOTICE_HASH: &str = "ac3b44ab30a4235edd18feca1ab4f802d57c8d3d0ee4878dc77b81a6b127155f";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sm86Payload {
    pub directory: PathBuf,
}

impl Sm86Payload {
    pub fn verify(&self) -> Result<(), String> {
        for (name, _, hash) in PROXIES {
            verify_hash(&self.directory.join(name), hash)?;
        }
        verify_hash(&self.directory.join(NOTICE_NAME), NOTICE_HASH)
    }
}

fn verify_hash(path: &Path, expected: &str) -> Result<(), String> {
    let actual =
        downloader::compute_sha256(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if actual != expected {
        return Err(format!("SM86 integrity check failed: {}", path.display()));
    }
    Ok(())
}

pub fn cache_directory() -> PathBuf {
    downloader::get_components_root().join(format!("dlssg-sm86-{RELEASE_VERSION}-{RELEASE_COMMIT}"))
}

pub fn find_payload() -> Option<Sm86Payload> {
    let payload = Sm86Payload {
        directory: cache_directory(),
    };
    payload.verify().ok().map(|_| payload)
}

pub async fn ensure_payload(log: &mut Vec<String>) -> Result<Sm86Payload, String> {
    let payload = Sm86Payload {
        directory: cache_directory(),
    };
    let base =
        format!("https://raw.githubusercontent.com/sdli1995/dlssg_for_sm86/{RELEASE_COMMIT}");
    // Retain the upstream notices verbatim before accepting the DLL set.
    downloader::download_file_with_sha256(
        &format!("{base}/THIRD_PARTY_NOTICES.txt"),
        &payload.directory.join(NOTICE_NAME),
        NOTICE_HASH,
    )
    .await?;
    for (name, source, hash) in PROXIES {
        downloader::download_file_with_sha256(
            &format!("{base}/{source}"),
            &payload.directory.join(name),
            hash,
        )
        .await?;
    }
    payload.verify()?;
    log.push(format!(
        "[SM86] Verified upstream {RELEASE_VERSION} ({RELEASE_COMMIT}), bundled 310.9 runtime"
    ));
    Ok(payload)
}

pub fn configure_ini(base: &str, multiplier: u32) -> Result<String, String> {
    if !(2..=4).contains(&multiplier) {
        return Err("SM86 multiplier ceiling must be 2x, 3x or 4x".into());
    }
    let values = [
        ("General", "Enabled", "1"),
        ("FrameGeneration", "Optimized", "1"),
        ("Compatibility", "Preset", "Auto"),
        ("Logging", "Level", "1"),
        ("Logging", "Directory", "dlssg_sm86\\logs"),
        ("Runtime", "Mode", "Bundled"),
        ("Runtime", "CacheDirectory", ""),
    ];
    // Remove every existing occurrence of managed keys first: Windows INI
    // readers differ in how they handle duplicate keys/sections. Updating only
    // the first occurrence could leave an old 6x ceiling effective at runtime.
    let mut section = String::new();
    let newline = if base.contains("\r\n") { "\r\n" } else { "\n" };
    let mut text = base.lines().filter(|line| {
        let trimmed = line.trim().trim_start_matches('\u{feff}');
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            section = trimmed[1..trimmed.len() - 1].trim().to_string();
        }
        if trimmed.starts_with(';') || trimmed.starts_with('#') { return true; }
        let Some((key, _)) = trimmed.split_once('=') else { return true; };
        let ceiling = section.eq_ignore_ascii_case("FrameGeneration") && key.trim().eq_ignore_ascii_case("MaxGeneratedFrames");
        !ceiling && !values.iter().any(|(s, k, _)| section.eq_ignore_ascii_case(s) && key.trim().eq_ignore_ascii_case(k))
    }).collect::<Vec<_>>().join(newline);
    for (section, key, value) in values {
        text = set_ini(&text, section, key, value);
    }
    Ok(set_ini(
        &text,
        "FrameGeneration",
        "MaxGeneratedFrames",
        &(multiplier - 1).to_string(),
    ))
}

/// Strong recognition: only the exact pinned DLLs are automatically removable.
/// A string such as "Streamline" alone must never authorize deleting a DLL.
pub fn is_proxy(path: &Path) -> bool {
    downloader::compute_sha256(path)
        .map(|hash| PROXIES.iter().any(|p| p.2 == hash))
        .unwrap_or(false)
}

/// Default deployment requires the complete coordinating tool-proxy set.
/// This intentionally fails closed on conflicts instead of guessing which
/// remaining name will load. Single-slot title profiles are deferred.
pub fn check_proxy_slots(mod_root: &Path, game_dir: &Path) -> Result<(), String> {
    let previous = journal::read_manifest(game_dir);
    for (name, _, _) in PROXIES {
        let path = mod_root.join(name);
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(format!(
                    "Cannot inspect SM86 proxy slot {}: {error}",
                    path.display()
                ))
            }
        };
        let managed = previous
            .as_ref()
            .map(|m| m.added.iter().any(|rel| game_dir.join(rel) == path))
            .unwrap_or(false);
        let prior_rtxmfg = name == "version.dll"
            && previous
                .as_ref()
                .map(|m| {
                    m.route == "optiscaler"
                        && m.frame_gen_backend
                            != Some(crate::core::framegen::FrameGenBackend::DlssgSm86)
                })
                .unwrap_or(false)
            && crate::core::pe::is_optiscaler_or_proxy(&path);
        if !managed || !metadata.is_file() || !(is_proxy(&path) || prior_rtxmfg) {
            return Err(format!("SM86 proxy slot is occupied: {}. Restore the previous installation or remove the conflicting mod explicitly; no files were changed.", path.display()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::optiscaler::get_ini;

    #[test]
    fn multiplier_ceiling_and_unrelated_settings() {
        for multiplier in 2..=4 {
            let ini = configure_ini("; keep me\n[Unrelated]\nKey=value\n", multiplier).unwrap();
            assert_eq!(
                get_ini(&ini, "FrameGeneration", "MaxGeneratedFrames"),
                Some((multiplier - 1).to_string())
            );
            assert_eq!(
                get_ini(&ini, "FrameGeneration", "Optimized"),
                Some("1".into())
            );
            assert!(ini.contains("; keep me"));
            assert!(ini.contains("Key=value"));
            assert!(!ini.contains("ForcePluginFrames"));
            assert!(!ini.contains("ForceGeneratedFrames"));
        }
        for invalid in [0, 1, 5, 6, u32::MAX] {
            assert!(configure_ini("", invalid).is_err());
        }
    }

    #[test]
    fn refuses_unknown_proxy_and_never_deletes_it() {
        let dir = std::env::temp_dir().join(format!("sm86-conflict-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("version.dll");
        fs::write(&path, b"original game DLL Streamline").unwrap();
        assert!(check_proxy_slots(&dir, &dir).is_err());
        assert!(!is_proxy(&path));
        assert_eq!(fs::read(&path).unwrap(), b"original game DLL Streamline");
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn duplicate_managed_ini_keys_cannot_override_the_ceiling() {
        let ini = configure_ini("[FrameGeneration]\nMaxGeneratedFrames=5\nmaxgeneratedframes=99\n; keep comment\n[FrameGeneration]\nMaxGeneratedFrames=5\n[Custom]\nMaxGeneratedFrames=77\n", 2).unwrap();
        assert_eq!(ini.matches("MaxGeneratedFrames=1").count(), 1);
        assert!(!ini.contains("=5"));
        assert!(!ini.contains("=99"));
        assert!(ini.contains("MaxGeneratedFrames=77"));
        assert!(ini.contains("; keep comment"));
    }

    #[test]
    fn payload_integrity_rejects_tampered_files() {
        let dir = std::env::temp_dir().join(format!("sm86-tamper-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("version.dll"), b"modified payload").unwrap();
        assert!(Sm86Payload { directory: dir.clone() }.verify().unwrap_err().contains("integrity"));
        fs::remove_dir_all(dir).unwrap();
    }
}
