# Native effect validation - 2.2.0

Tested on Windows x64 on 12 October 2026.

- **20/20 Rust kernel tests passed.** Includes signed/fractional offsets, recolouring, HDR, premultiplied filtering, repeated boundaries, mix bypass and cancellation, plus directional blur, blur energy beyond source bounds, padding calculations and matching cropped/expanded renders. New tests compare optimized output against the frozen 2.1 renderer across cropped regions, fractional offsets, blur modes, mixed results and edge policies, and verify centre restoration with transparency and expanded inputs.
- **63/63 AE 2026 checks passed** with After Effects **26.5x89**: 27 regression checks, 15 centre restoration checks and 21 bounds/dimensions/upgrade checks against the final release plugin.
- **60/60 AE 2025 checks passed** with After Effects **25.6.6x4** against the same release plugin. The three saved-project upgrade checks require the AE 2026 fixture and are explicitly skipped on AE 2025.
- **Actual full- and half-resolution frames rendered successfully with aerender.** Pixel checks confirm that expanded red and blue fringes remain outside the physical source layer, the source remains centred, and the exterior is clear at both resolutions. Centre-restoration renders also match the original green centre while retaining the recoloured outside at full and half resolution.
- The installed AE 2026 plugin and the distributable have matching SHA-256 hashes. Diagnostic tracing is disabled in the release build.

## Checked behaviour

- Centre restoration at 8-, 16- and 32-bit depth; 0/50/100% strength, animated strength, soft transition, Mix, blurred detail, expanded fringes and an asymmetrically expanded upstream input.
- One native effect on an adjustment layer processes multiple underlying visual layers.
- Identity and RGB separation at 8-, 16- and 32-bit depth.
- Green-to-teal recolouring, HDR values above white, independent channel keyframes, direction, X/Y offsets, master split and Mix.
- Global and per-channel softness; adjustment-layer opacity and masks.
- Translucent source pixels, transparency, displaced fringes and soft alpha without dark edges.
- RGB contributions continue beyond the actual source layer's left, right, top and bottom edges.
- Expansion keeps the original image aligned; two expanding Chromatic AE effects can be stacked.
- Horizontal, Vertical and Both blur modes; per-channel overrides and master inheritance.
- Blur tails extend beyond the source and reach transparency instead of being clipped.
- Expansion can be disabled; Mix at zero bypasses the expanded treatment.
- A saved 2.0 project retains its distance keyframes and custom colours, with compatible defaults for the new controls, including centre restoration at zero.

Reports: `tests/native-results-26.5.txt`, `tests/native-centre-results-26.5.txt` and `tests/native-bounds-results-26.5.txt`, with corresponding `25.6` reports. Kernel performance measurements and reproduction instructions are in `PERFORMANCE.md`.

## Implementation and limits

The effect grows its SmartFX maximum output rectangle using signed channel offsets and the finite support of its three fractional box passes. The current output request is intersected with those bounds. Cropped requests, empty bounds queries, downsampling and input/output origins are handled explicitly. Blur intermediates include complete convolution support around the requested output region so the kernel can carry colour and alpha past the original image edge. The optimization retains full input checkout for correct repeat-edge sampling and upstream effects. See the [Adobe SmartFX guide](https://ae-plugins.docsforadobe.dev/smartfx/smartfx/) for the host's rectangle contract.

AE colour values are converted to premultiplied values for filtering and back to straight RGB for output. Repeat Edge Pixels intentionally extends edge colours; turn it off for naturally fading transparent blur tails. Layer bounds can grow, but the composition frame still limits the exported picture.

This is a Windows x64 CPU effect. Interactive gestures, Windows ARM, macOS, unusually large canvases and every possible upstream effect have not been tested. The plugin advertises multi-frame rendering and has no mutable shared render state; a long multi-frame stress test has not been performed. Large images and softness values need additional temporary memory.

In headless AE 2026, expression pixel probes returned stale empty values after `app.open`, including on an unprocessed solid. The test suite therefore performs all pixel checks before the saved-project metadata checks. Independent aerender output was also verified after reopening the project.

## Reproduce

1. Run `native\build.ps1`.
2. Close AE and run `Install Chromatic AE.cmd`.
3. Run `tests\run-native-tests.ps1`. It launches a separate headless process and runs `tests/native-suite.jsx`.
4. Run `tests/bounds-resolution.jsx` and `tests/centre-resolution.jsx` in fresh AE processes after the suite. Render their saved `test-output/bounds-resolution.aep` and `test-output/centre-resolution.aep` with aerender, then run `tests/check-bounds-renders.py` and `tests/check-centre-renders.py` using Python and Pillow.

The 2.0 demo fixture is stored under `tests/fixtures/`. The original ScriptUI version and its separate evidence are preserved under `legacy/`.

## macOS universal build (19 October 2026)

The Mac package is v2.2.0 with unchanged rendering code, controls, defaults and effect match name. `tests/verify-v22-source.py` checks the engine, AE adapter, dependency manifests and released Windows binary against their original v2.2 hashes. The Mac build changes only platform registration and packaging.

- Apple Silicon: all 20 rendering tests pass, and the actual parameter-setup callback produces the same 32 control IDs in the same order as the recorded Windows AE report.
- Intel: all 20 rendering tests pass, with the same 32 Windows control IDs and order.
- The packaged Mach-O contains both arm64 and x86_64 architectures. Code-signature verification, PiPL checks and dynamic loading/registration pass on both Apple Silicon and Intel.
- The packaged installer passes a fresh installation and a second installation on a clean Mac runner, including the backup of the existing bundle outside Adobe's plugin folders. The installed plugin passes the same signature and loading checks.
- The downloaded ZIP passes integrity, executable permission and per-file SHA-256 checks.

This is an ad-hoc signed build, not Apple-notarized. AE itself is not installed on the Mac runners, so Mac AE-host rendering and opening an existing project on Mac remain unverified. The optional AE-host suite is included in the Mac ZIP; see `MAC-INSTALL.md`.

Mac package source: `7b3d0cbb39befa716f8fe32e1aa042ff65b52b05`.

- [Mac build and architecture checks](https://github.com/Gavin-Barnett/chromaticAE/actions/runs/39205096183)
- [Mac installer and upgrade checks](https://github.com/Gavin-Barnett/chromaticAE/actions/runs/39205428496)

Mac ZIP SHA-256: `f2d5b67518a8cb44330d077a993ea3e43141674ae9597f193a0cb2cb441c9d83`.
Windows `.aex` SHA-256 remains `ae8aac580cc89a8453320e88357389cbdcf978ed984f0c81e4c1fbbeb6c382b7`.
