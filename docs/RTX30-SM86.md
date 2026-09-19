# Experimental RTX 30 Frame Generation

This fork manages `dlssg_for_sm86` as an external runtime; it does not implement
DLSS-G itself. This is experimental and has not been validated on physical RTX
hardware by the implementation author. Existing RTX 40 routes remain available.

## Eligibility and setup

1. Use Windows x64 and a GeForce RTX 30-series GPU. Studio prefers a detected
   NVIDIA adapter over an integrated adapter; ensure the game uses that adapter.
2. Select a 64-bit DirectX 12 rendering executable. Ambiguous API detection,
   DX11, Vulkan, RTX 20, and unknown GPUs fail closed.
3. The game must include its original `nvngx_dlssg.dll` or `sl.dlss_g.dll`.
   `sl.dlss.dll` is Super Resolution, not Frame Generation. Files introduced by
   Studio's manifest do not qualify as native DLSS-G support.
4. Enable **Frame Generation — DLSSG SM86 (experimental, RTX 30)** in the game
   sheet. The feature is opt-in. Choose a 2x, 3x, or 4x ceiling.
5. Install while the game is closed. Studio downloads the pinned external
   runtime and notices from upstream, verifies SHA-256, then deploys it.
6. Start the game and enable DLSS Frame Generation in its own settings. The
   selected ceiling does not force the game to request that multiplier.

| Studio ceiling | SM86 `MaxGeneratedFrames` |
| --- | --- |
| 2x | 1 |
| 3x | 2 |
| 4x | 3 |

Optimized mode is `1`, the compatibility preset is `Auto`, logging level is `1`,
and the runtime mode is `Bundled`. Unrelated INI sections are retained. Advanced
experimental settings are not added or exposed by Studio.

## Payload and ownership

Pinned upstream: [dlssg_for_sm86 0.3.3, commit 5e79459](https://github.com/sdli1995/dlssg_for_sm86/tree/5e79459c2d521f8c3276ce9ee02342c0e2686982).
The 310.9 builds are individually hashed in `src/core/sm86_fg.rs`:

- `version.dll`
- `alternatives/winmm.dll` → `winmm.dll`
- `alternatives/dbghelp.dll` → `dbghelp.dll`
- `alternatives/dinput8.dll` → `dinput8.dll`
- `THIRD_PARTY_NOTICES.txt` → `dlssg_sm86_THIRD_PARTY_NOTICES.txt`

These are distinct forwarding DLLs, not four renamed copies of `version.dll`.
The first loaded proxy activates the runtime; the others forward system calls.
The initial implementation deploys all four or refuses the installation. An
occupied game/foreign-mod slot is never overwritten. A journal-owned RTXMFG
`version.dll` from the previous OptiScaler installation may be replaced.
Single-proxy profiles require title-specific load-path validation and are not
implemented. Neither `dxgi.dll` nor `d3d12.dll` is used as an SM86 proxy.

The payload is downloaded only when this backend is selected, cached separately
under Studio's components directory, and reverified before deployment. No SM86
binaries, NVIDIA runtime, or GPL runtime source are included in Studio's source
tree or executable. Downloading from upstream is not a determination that
redistribution rights have been granted. Any packaging or redistribution of
the runtime still requires review of the upstream and NVIDIA terms. Upstream
third-party notices are retained verbatim in the cache and the game directory.

## Rendering and restore

- SM86 never installs the RenoDX Ada MFG add-on or standalone RTXMFG alongside
  itself. The rendering route's non-FG functionality is retained.
- OptiScaler is configured to yield Frame Generation externally. Its automatic
  replacement of the game's Streamline stack is skipped for SM86; compatibility
  of this combination still needs real-game testing.
- Proxy names, backend, added files, and replaced configuration are journaled.
  A backup is required to succeed before an existing file is overwritten, and
  the manifest is persisted before each tracked write.
- Reinstalls and switches involving SM86 checkpoint the previous managed files,
  directories, settings, and manifest separately from vanilla backups. Reported
  installation errors recover that previous installation, including failures
  after new proxies and configuration have been written. First-install errors
  restore the originals.
- Interrupted switches retain a recovery marker and block further installs.
  **Restore originals** first recovers the checkpoint, then restores the vanilla
  files. If rollback itself fails (for
  example due to a file lock), keep `_DLSS5_Backup` and retry restore before
  another install.
- Completed installs refuse to delete SM86 proxies changed by another tool.
  In-progress manifests remain recoverable after an interrupted copy.
- Reinstallation carries forward original backups and added-file ownership.
  Disabling SM86 removes its managed proxies/configuration/notices and restores
  any original configuration. A proxy changed by another tool blocks switching.
- Runtime-created logs (`dlssg_sm86/logs`) and the upstream user-level bundle
  cache are not recursively deleted; Studio does not own those directories.

Allow temporary disk space for one additional copy of the existing managed
installation during a reinstall or backend switch. Its checkpoint is removed
after a successful commit or rollback. The headless `--deploy-optiscaler` command
uses the same GPU/backend checks and downloads SM86 when its cache is missing.

DLL injection may be incompatible with anti-cheat or game policies. Existing
process and anti-cheat safeguards still apply. Do not treat this feature as
approval to use it in protected online games.

## Validation

Run on Windows:

```powershell
cargo check --locked --all-targets
cargo test --locked --bin dlss-studio
cargo test --locked --bin dlss-studio sm86_verified_payload_lifecycle -- --ignored --nocapture
```

New tests cover GPU classification, API/bitness/native-FG gating, Ada routing,
multiplier translation, INI preservation, foreign proxy rejection, old-manifest
compatibility, repeated and failed manifest saves, interrupted-switch recovery,
restore, and backup continuity.
The final command downloads and verifies the four pinned DLLs (about 120 MB),
then exercises install, reinstall, rendering/backend switches, and restoration
in a synthetic game directory. It also injects late reinstall/route-switch
failures and compares every previous file and the manifest after rollback,
and verifies that a proxy in the wrong filename slot is rejected by scanning.
The DLLs are copied as data, never loaded or
executed. Windows CI runs this separately from the offline unit tests.

Before calling the feature production-ready, test Cyberpunk 2077, Black Myth:
Wukong, and Final Fantasy VII Rebirth on physical RTX 30 hardware. For each
rendering route, check 2x/3x/4x requests, logs showing one active proxy, game
restarts, repeat installation, disabling FG, route changes, restore, and a
deliberately occupied proxy slot. Compare original-file hashes after restore.
Also repeat existing RTX 40 workflows to verify behavior has not regressed.

Deferred: RTX 20, DX11, Vulkan, 6x, render-path proxies, title-specific single
slots, advanced tuning, and translations of the new experimental UI text.
