# Validation - 1.0.0

Tested in After Effects 26.5x89 on Windows, using a disposable headless project.

- 16 integration checks passed: zero-split identity at 8/16/32 bpc, half-opacity preservation, transparent background, green-to-teal mapping, red/blue fringes outside source coverage, master and per-channel softness, straight colour on soft edges, independent keyframes, split rotation, master amount, source timing, output trim and unsupported-selection handling.
- 16 dependency-free Node expression assertions passed.
- A 960 x 540 demo frame rendered successfully through `aerender` using the installed JPEG preset and was visually inspected. The preview is in `examples/preview.jpg`.
- The installed AE 2026 panel's SHA-256 matches the delivered source.
- JSX and PowerShell syntax checks passed.

The tests exercise native effects, property values, expressions and rendered pixel samples. Interactive panel testing was blocked by existing AE startup dialogs, so the docked/floating UI was not visually verified. `tests/ui.jsx` is supplied for a GUI smoke check. Headless AE does not expose ScriptUI constructors.

The installed TIFF-named output preset resolved to Photoshop format and failed during demo export; the JPEG preset succeeded. The panel does not configure or depend on output-module presets.

Run the renderer checks with `tests/run-ae-tests.ps1` and the expression checks with `node tests/test-expressions.cjs`. Run integration scripts only in a disposable project; the runner starts a separate headless instance. No test files or app preference data are included in the release archive.
