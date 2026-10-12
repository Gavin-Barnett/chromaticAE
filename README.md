# Chromatic AE

A small dockable After Effects panel for RGB splitting, channel recolouring and soft edges. Uses AE's built-in effects, so projects render without the panel installed. This is an ExtendScript panel (`.jsx`), not a compiled `.aex` effect.

## Fastest start

1. In After Effects, choose **File > Scripts > Run Script File**.
2. Open **Chromatic AE.jsx**.
3. Select your footage, text, shape layer, or precomp and click **Apply to selected layers**.

The panel opens the **CHROMATIC - animate these controls** layer in Effect Controls. Use its stopwatches just like any other AE effect. If Effect Controls is hidden, press **F3**.

On Windows you can also double-click **Start Chromatic AE.cmd**. It finds an installed AE version and runs the panel. No restart or installation is required for the floating panel.

## Dock it permanently

Double-click **Install Chromatic AE.cmd**. It installs to the newest detected After Effects version, requesting administrator access only when Program Files requires it. Restart AE, then choose **Window > Chromatic AE.jsx** and dock the panel.

For another version, run PowerShell with:

```powershell
.\Install-Chromatic.ps1 -AfterEffectsPath 'C:\Program Files\Adobe\Adobe After Effects 2025'
```

Manual installation: copy `Chromatic AE.jsx` into your version's `Support Files\Scripts\ScriptUI Panels` folder. To uninstall, remove that one file. Existing projects continue rendering.

## Demo

Run **Create Demo.jsx** from File > Scripts > Run Script File to create an animated example with independent red and blue movement and a teal green channel. The downloadable package also contains a rendered preview and an AE 2026 demo project.

## Controls

| Control | What it does |
| --- | --- |
| Split Amount (%) | Scales all distances and XY offsets. 0 aligns the channels; 100 uses the distances as entered. |
| Direction | Rotates the shared split axis. 0 is horizontal; 90 is vertical. |
| Edge Softness (px) | Adds Gaussian softness to every channel and its transparency. |
| Red / Green / Blue Distance (px) | Independent signed distances along the shared axis. Animate each separately. |
| Red / Green / Blue XY Offset | Additional independent offsets for two-dimensional movement. |
| Red / Green / Blue Colour | Replaces that channel's full colour contribution, retaining its original intensity. |
| Red / Green / Blue Softness (px) | Adds softness to one channel. Combines with the master softness. |

For green-to-teal, select the **Green to teal** starting look, or change **Green Colour** to teal. Colours can be keyframed too. Colour remapping is additive: remapped contributions can brighten overlaps. Use a 32-bpc project when you want to retain values above white.

The softness controls blur each channel's image edges and fine detail. They are not a separate frame-border vignette. To restore an unchanged image, use Split Amount 0, all softness values 0 and the default pure RGB colours.

## Editing and animation

- The panel creates one output precomp in your timeline and a named project folder for its source and internal passes.
- **Open controls** finds the selected Chromatic output and opens its controls at the current source time.
- **Edit source** opens your original editable layers. Their existing effects and animation remain inside the source precomp.
- Keyframe the controls inside the Chromatic comp. They travel with the source when its output is trimmed, moved or time-remapped.
- **Edit > Undo** reverts an application in one step.
- Each newly applied rig is independent. Duplicating an existing output layer shares its underlying source and controls, as with any AE precomp. Apply to a fresh source for a separate rig.
- Negative distances are supported. Negative total softness is clamped to zero.

## Scope

Tested in After Effects 26.5 on Windows: 16 integration checks passed, including rendered pixel samples in 8-, 16- and 32-bpc projects. Ordinary 2D footage, text, shapes and precomps are supported. Adjacent selected layers are processed together. For 3D scenes, adjustment layers, track mattes, blend-mode composites, or parenting outside the selection, precompose the complete visual group first.

As with normal precomposition, expressions depending on the original comp's layer indices, external layer names or `thisComp` may need updating. Keep the generated control names and internal layers intact. The rig crops at composition bounds; large splits on full-frame footage may expose coloured frame edges. There is no automatic edge extension or lens distortion in this release.

The rig uses three colour passes plus three alpha-coverage passes. This preserves displaced transparent fringes and avoids piling up alpha at overlapping edges. It costs more than a single native effect; preview at half resolution if needed. Audio passes through once.

## Development

- `tests/integration.jsx`: AE pixel samples for identity, transparency, channel remapping, independent offsets, keyframing, softness and trim preservation. Run in a disposable AE project.
- `tests/test-expressions.cjs`: dependency-free Node checks for the generated offset expressions.
- `test-output/`: local test projects and reports; excluded from Git.

Implementation references: [Adobe scripting](https://helpx.adobe.com/after-effects/using/scripts.html), [channel effects](https://helpx.adobe.com/after-effects/using/channel-effects.html), and [Adobe scripting API reference](https://ae-scripting.docsforadobe.dev/).
