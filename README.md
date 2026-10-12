# Chromatic AE 2.2

A native Windows After Effects effect for RGB separation, channel recolouring and soft edges. Apply **one effect to an adjustment layer** to process all visible layers beneath it. It also works directly on footage, text, shapes and precomps.

## New in 2.2

- **Centre Colour Restore:** blends the original image back into the middle, with a smooth elliptical transition to the aberrated edges. Fully keyframeable, from 0 to 100%.
- Faster channel sampling and blur, with blur work limited to the pixels needed for the current output region. See `PERFORMANCE.md` for measured results and the benchmark method.
- Existing colours, split distances, blur directions and keyframes are retained. Centre restoration defaults to 0%, preserving older projects' appearance.

## Install

1. Download the release ZIP and **extract all of it**.
2. Close After Effects, then double-click **Install Chromatic AE.cmd**. Accept the Windows administrator prompt if it appears.
3. Start After Effects. Select an adjustment layer and choose **Effect > Chromatic > Chromatic AE**, or search for **Chromatic AE** in Effects & Presets.

The installer selects the newest installed AE version. To choose another installation:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\Install-Chromatic.ps1 -AfterEffectsPath "C:\Program Files\Adobe\Adobe After Effects 2025"
```

Manual installation: copy `plugin\Chromatic AE.aex` into:

```text
C:\Program Files\Adobe\Adobe After Effects 2026\Support Files\Plug-ins\Chromatic AE\
```

Create the final folder if needed, and restart AE. This is a native effect; the Window menu and ScriptUI Panels folder are for the legacy version.

**Another machine:** transfer the complete release ZIP, extract it, and run the same installer. No Rust, Adobe SDK, account, activation or ScriptUI panel is needed. Install the `.aex` on every machine that will render a project using it. This build is for Windows x64; it does not run on macOS or Windows ARM natively.

## Use

Create **Layer > New > Adjustment Layer**, put it above the layers to process, and apply **Chromatic AE** once. All controls are in Effect Controls; use their stopwatches to animate them.

| Control | Behaviour |
| --- | --- |
| Split Amount | Scales all channel distances and X/Y offsets. 0% collapses separation; 100% uses the entered distances. |
| Direction | Rotates the shared axis used by the three signed Distance controls. 0 degrees is horizontal; 90 degrees is vertical. |
| Edge Softness | Softens all channel edges and details. Starts at zero. |
| Blur Dimensions | Chooses Horizontal and Vertical, Horizontal, or Vertical for the shared softness direction. |
| Centre Colour Restore | 0% keeps the full effect. 100% restores the original middle while retaining aberration around the outer edges. Intermediate values blend between them. |
| Mix | Blends the processed result with the original image. 0% bypasses the treatment. |
| Repeat Edge Pixels | Extends boundary pixels when offsets or blur sample outside the image. Disable for transparent boundaries and naturally fading blur tails. |
| Expand Layer Bounds | Automatically grows the effect output to include the displaced channels and complete blur support. Enabled by default; disable to restore clipping to the input bounds. |
| Red / Green / Blue Distance | Moves that channel along Direction, in source pixels. Each can be animated independently. |
| Red / Green / Blue X/Y Offset | Adds independent horizontal and vertical movement to that channel. |
| Red / Green / Blue Colour | Replaces the colour contributed by that channel while retaining its intensity. Set Green Colour to teal to remap the full green channel. |
| Red / Green / Blue Softness | Adds softness to that channel only, on top of Edge Softness. |
| Red / Green / Blue Blur Dimensions | Use Master, Horizontal and Vertical, Horizontal, or Vertical. Overrides the direction of all softness for that channel. |

Start Centre Colour Restore around **75-100%** for a cleaner centre. The protected region follows the original layer centre and proportions; its inner area is fully restored at 100%, with a fixed smooth fade toward the outside. This restores original detail as well as colour, including detail softened by blur. On a composition-sized adjustment layer, it follows the composition centre.

Colour contributions add together. Custom colours can create values above white in a 32-bit project. Softness is a Gaussian-like blur of channel detail and coverage, measured in source pixels; it is not a vignette or a blur restricted to the outside border of the composition. Large softness values and large frames take longer to render.

Normal adjustment-layer opacity and masks work. With Expand Layer Bounds enabled, the result can extend beyond the source layer; the composition frame still defines the final exported picture. Keep Repeat Edge Pixels off when you want the extended edges to fade into transparency. The effect uses CPU rendering, supports 8-, 16- and 32-bit projects, and scales separation and softness for reduced-resolution previews.

## Demo

Open `examples\Chromatic AE Native Demo.aep`, or choose **File > Scripts > Run Script File** and select `Create Demo.jsx` to add the demo to your current project. The demo uses text, shapes, a background, and one adjustment layer with one Chromatic AE effect.

## Previous version

`legacy/` preserves the original 1.0 ScriptUI/precomp rig for existing projects. It is superseded by this native effect. Existing rigs are not automatically converted; use a new adjustment layer and the native effect for new work. The old panel can coexist with the native plugin.

## Build from source

Install stable Rust with the MSVC target and Visual Studio C++ Build Tools with the Windows SDK. Run:

```powershell
.\native\build.ps1
```

The build uses pinned open-source `after-effects` bindings and `pipl` resource tooling, including upstream Windows resource and concurrency fixes. No Adobe SDK download is required for this build. `Cargo.lock` is committed. `native/build.ps1` runs the kernel tests, builds the release library and copies it to `plugin/Chromatic AE.aex`.

`tests/native-suite.jsx` runs the original regression, centre restoration and bounds/dimensions checks inside AE. See `VALIDATION.md` for tested behaviour and limitations. Dependency notices are in `THIRD-PARTY-NOTICES.txt`.
