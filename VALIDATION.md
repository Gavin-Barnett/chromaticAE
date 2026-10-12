# Native effect validation - 2.0.0

Tested on Windows x64 with Adobe After Effects 2026, version **26.5x89**, on 12 October 2026.

- **11/11 Rust kernel tests passed.** Independent signed and fractional offsets, channel recolouring, HDR values, premultiplied filtering, blur energy/symmetry, repeated boundaries, mix bypass, cropped output origins, and cancellation.
- **27/27 AE integration checks passed.** The actual `.aex` was loaded by a fresh AE process. A composition with three visual layers was processed by one native effect on one adjustment layer.
- **A real 960 x 540 frame rendered successfully using aerender** from the saved native demo, and the resulting image was visually inspected (`examples/preview.jpg`).
- The installed `.aex` hash matches the distributable. No diagnostic tracing is enabled in the release build.

The AE checks cover:

- Adjustment-layer identity and RGB separation in 8-, 16- and 32-bit projects.
- Green-to-teal recolouring and HDR colour contributions above 1.0.
- Independent red, green and blue distance keyframes and interpolation.
- Direction, X/Y offsets, master separation amount and Mix.
- Master and per-channel edge softness.
- Adjustment-layer opacity and masks.
- Translucent and transparent source pixels, shifted fringes, and soft alpha without dark edges.
- Smart Render requests with cropped output origins (AE's sampleImage rendering uses small regions).

Full host-test output: `tests/native-results-26.5.txt`.

## Scope

The native plugin and its renders were exercised through AE scripting and aerender. Interactive panel gestures were not manually tested. Other AE releases, Windows ARM, macOS, unusually large canvases and all possible upstream effects have not been validated. The build is Windows x64 only. CPU rendering is used. Multi-frame rendering is advertised by the plugin and uses no mutable shared render state, but a long multi-frame stress test has not been performed.

Offsets and softness are scaled by AE's downsample factors in the implementation. A separate reduced-resolution image comparison has not been performed. Output is confined to the existing image bounds. The complete input is requested for reliable edge repetition and regional rendering, so large images with softness enabled use more memory than a simple colour effect.

AE effect-world RGB values are converted to premultiplied values for filtering, then converted back to straight RGB before returning to AE. The 27 host checks include the alpha regression that motivated this handling.

## Reproduce

1. Run `native\build.ps1`.
2. Close AE and run `Install Chromatic AE.cmd`.
3. Run `tests\run-native-tests.ps1` in PowerShell. It launches a separate headless AE instance; it does not close or alter an existing user project.

The original ScriptUI version's evidence is preserved under `legacy/VALIDATION.md`; those tests do not count toward the native version's results.
