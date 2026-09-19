//! Shared UI/deployer policy. A multiplier is a ceiling, not a promise that a
//! game's Streamline integration requests that many generated frames.
use crate::core::{
    gpu::{GpuInfo, NvidiaArch},
    install_routes::InstallRoute,
    scan::GameEntry,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FrameGenBackend {
    None,
    RenoDxAda,
    DlssgSm86,
    RtxMfg,
}

impl FrameGenBackend {
    pub fn label(self, lang: &str) -> &'static str {
        match self {
            Self::None => crate::core::i18n::t(lang, "backend_fg_disabled"),
            Self::RenoDxAda => crate::core::i18n::t(lang, "backend_fg_renodx_ada"),
            Self::DlssgSm86 => crate::core::i18n::t(lang, "backend_fg_dlssg_sm86"),
            Self::RtxMfg => crate::core::i18n::t(lang, "backend_fg_rtx_mfg"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FrameGenCapability {
    pub backend: FrameGenBackend,
    pub reason: Option<&'static str>,
}

pub fn framegen_capability(
    game: &GameEntry,
    gpu: &GpuInfo,
    route: InstallRoute,
) -> FrameGenCapability {
    let reject = |reason| FrameGenCapability {
        backend: FrameGenBackend::None,
        reason: Some(reason),
    };
    if game.bitness != 64 {
        return reject("Frame Generation requires a 64-bit game.");
    }
    if gpu.nvidia_arch() == NvidiaArch::Ampere {
        if !game.api.eq_ignore_ascii_case("DirectX 12") && !game.api.eq_ignore_ascii_case("d3d12") {
            return reject("RTX 30 Frame Generation requires DirectX 12 (not DX11 or Vulkan).");
        }
        if !has_native_dlssg(game) {
            return reject("RTX 30 Frame Generation requires the game's original nvngx_dlssg.dll or sl.dlss_g.dll.");
        }
        return FrameGenCapability {
            backend: FrameGenBackend::DlssgSm86,
            reason: None,
        };
    }
    if gpu.nvidia_arch() == NvidiaArch::Ada
        && crate::core::install_routes::is_frame_generation_supported(game)
    {
        return FrameGenCapability {
            backend: if route == InstallRoute::OptiScaler {
                FrameGenBackend::RtxMfg
            } else {
                FrameGenBackend::RenoDxAda
            },
            reason: None,
        };
    }
    reject("Managed Frame Generation requires a compatible RTX 30/40 GPU and game. RTX 20 and unknown GPUs are not enabled.")
}

pub fn has_native_dlssg(game: &GameEntry) -> bool {
    let manifest = crate::core::journal::read_manifest(&game.dir);
    game.has_frame_generation
        && game.files.iter().any(|file| {
            let rel = file.rel.replace('\\', "/").to_ascii_lowercase();
            let name = rel.rsplit('/').next().unwrap_or("");
            let native_name = name == "nvngx_dlssg.dll" || name == "sl.dlss_g.dll";
            let injected = manifest
                .as_ref()
                .map(|m| {
                    m.added
                        .iter()
                        .any(|p| p.replace('\\', "/").eq_ignore_ascii_case(&rel))
                })
                .unwrap_or(false);
            native_name && !injected
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gpu(model: &str) -> GpuInfo {
        GpuInfo {
            name: format!("NVIDIA GeForce RTX {model}"),
            vendor_id: 0x10de,
            device_id: 0,
            dedicated_video_memory: 0,
            is_rtx_40: model.starts_with("40"),
        }
    }

    fn game() -> GameEntry {
        GameEntry {
            bitness: 64,
            api: "DirectX 12".into(),
            has_frame_generation: true,
            files: vec![crate::core::scan::GameFileItem {
                rel: "bin/nvngx_dlssg.dll".into(),
                version: None,
            }],
            ..Default::default()
        }
    }

    #[test]
    fn ampere_native_fg_supported_on_each_render_route() {
        for route in [
            InstallRoute::Native,
            InstallRoute::OptiScaler,
            InstallRoute::Feeder,
        ] {
            assert_eq!(
                framegen_capability(&game(), &gpu("3080"), route).backend,
                FrameGenBackend::DlssgSm86
            );
        }
    }

    #[test]
    fn ampere_fails_closed_on_api_bitness_and_sr_only() {
        for api in ["DirectX 11", "Vulkan", "Undetected", "DirectX 11/12"] {
            let mut game = game();
            game.api = api.into();
            assert!(
                framegen_capability(&game, &gpu("3070"), InstallRoute::Native)
                    .reason
                    .is_some()
            );
        }
        let mut game = game();
        game.bitness = 32;
        assert!(
            framegen_capability(&game, &gpu("3070"), InstallRoute::Native)
                .reason
                .is_some()
        );
        game.bitness = 64;
        game.files[0].rel = "sl.dlss.dll".into();
        game.mfg_unlock_installed = true;
        assert!(
            framegen_capability(&game, &gpu("3070"), InstallRoute::Native)
                .reason
                .is_some()
        );
        game.files.clear();
        assert!(
            framegen_capability(&game, &gpu("3070"), InstallRoute::Native)
                .reason
                .is_some()
        );
    }

    #[test]
    fn ada_routes_unchanged_and_turing_not_exposed() {
        assert_eq!(
            framegen_capability(&game(), &gpu("4080"), InstallRoute::Native).backend,
            FrameGenBackend::RenoDxAda
        );
        assert_eq!(
            framegen_capability(&game(), &gpu("4080"), InstallRoute::OptiScaler).backend,
            FrameGenBackend::RtxMfg
        );
        assert!(
            framegen_capability(&game(), &gpu("2080"), InstallRoute::Native)
                .reason
                .is_some()
        );
    }

    #[test]
    fn injected_dlssg_does_not_become_native_evidence() {
        let dir = std::env::temp_dir().join(format!("sm86-native-evidence-{}", std::process::id()));
        let mut game = game();
        game.dir = dir.clone();
        crate::core::journal::save_manifest(
            &dir,
            &crate::core::journal::ActiveManifest {
                added: vec!["bin\\nvngx_dlssg.dll".into()],
                ..Default::default()
            },
        )
        .unwrap();
        assert!(!has_native_dlssg(&game));
        std::fs::remove_dir_all(dir).unwrap();
    }
}
