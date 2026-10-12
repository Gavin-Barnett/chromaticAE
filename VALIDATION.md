# Native effect validation - 2.1.0

Tested on Windows x64 on 12 October 2026.

- **16/16 Rust kernel tests passed.** Includes signed/fractional offsets, recolouring, HDR, premultiplied filtering, repeated boundaries, mix bypass and cancellation, plus directional blur, blur energy beyond source bounds, padding calculations and matching cropped/expanded renders.
- **48/48 AE 2026 checks passed** with After Effects **26.5x89**: the original 27 regression checks and 21 bounds/dimensions/upgrade checks against the final release plugin.
- **45/45 AE 2025 checks passed** with After Effects **25.6.6x4** during development. The three saved-project upgrade checks require the AE 2026 fixture and are explicitly skipped on AE 2025.
- **Actual full- and half-resolution frames rendered successfully with aerender.** Pixel checks confirm that expanded red and blue fringes remain outside the physical source layer, the source remains centred, and the exterior is clear at both resolutions.
- The installed AE 2026 plugin and the distributable have matching SHA-256 hashes. Diagnostic tracing is disabled in the release build.

## Checked behaviour

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
- A saved 2.0 project retains its distance keyframes and custom colours, with compatible defaults for the new controls.

Reports: `tests/native-results-26.5.txt` and `tests/native-bounds-results-26.5.txt`.

## Implementation and limits

The effect grows its SmartFX maximum output rectangle using signed channel offsets and the finite support of its three fractional box passes. The current output request is intersected with those bounds. Cropped requests, empty bounds queries, downsampling and input/output origins are handled explicitly. Blur intermediates are padded so the kernel can carry colour and alpha past the original image edge. See the [Adobe SmartFX guide](https://ae-plugins.docsforadobe.dev/smartfx/smartfx/) for the host's rectangle contract.

AE colour values are converted to premultiplied values for filtering and back to straight RGB for output. Repeat Edge Pixels intentionally extends edge colours; turn it off for naturally fading transparent blur tails. Layer bounds can grow, but the composition frame still limits the exported picture.

This is a Windows x64 CPU effect. Interactive gestures, Windows ARM, macOS, unusually large canvases and every possible upstream effect have not been tested. The plugin advertises multi-frame rendering and has no mutable shared render state; a long multi-frame stress test has not been performed. Large images and softness values need additional temporary memory.

In headless AE 2026, expression pixel probes returned stale empty values after `app.open`, including on an unprocessed solid. The test suite therefore performs all pixel checks before the saved-project metadata checks. Independent aerender output was also verified after reopening the project.

## Reproduce

1. Run `native\build.ps1`.
2. Close AE and run `Install Chromatic AE.cmd`.
3. Run `tests\run-native-tests.ps1`. It launches a separate headless process and runs `tests/native-suite.jsx`.
4. Run `tests/bounds-resolution.jsx` in a fresh AE process after the suite. Render its saved `test-output/bounds-resolution.aep` with aerender, then run `tests/check-bounds-renders.py` using Python and Pillow.

The 2.0 demo fixture is stored under `tests/fixtures/`. The original ScriptUI version and its separate evidence are preserved under `legacy/`.
