# Chromatic AE 2.2 for Mac

This is the same v2.2 effect as the Windows release, packaged as a universal macOS plugin for **Apple Silicon and Intel**. Rendering code, controls, defaults, saved parameter IDs and effect identity are unchanged. The existing Windows installer and `.aex` are unchanged.

## Install

1. Download **Chromatic-AE-2.2.0-macOS-Universal.zip** from the v2.2.0 release and extract it completely.
2. Save your work and close After Effects and aerender.
3. Open **Install Chromatic AE.command**. Enter your Mac administrator password if requested; Terminal does not show characters while typing it.
4. Open AE and choose **Effect > Chromatic > Chromatic AE**, or open your existing project.

The installer places the bundle in Adobe's shared MediaCore folder, so installed AE versions can find it. It verifies the packaged file hashes and code signature, and preserves an existing installation in `/Library/Application Support/Chromatic AE/Backups` before replacing it. Avoid also installing another copy in an individual AE application's Plug-ins folder.

## If macOS blocks the installer

This build has an **ad-hoc code signature** and is **not Apple-notarized**. To run the downloaded installer, open Terminal, type `bash` followed by a space, drag **Install Chromatic AE.command** from Finder into Terminal, and press Return. The installer removes the quarantine attribute only from the plugin it installs; it does not turn off Gatekeeper or change system-wide security settings.

Manual location (copy the complete `.plugin` bundle, not its inner executable):

```text
/Library/Application Support/Adobe/Common/Plug-ins/7.0/MediaCore/Chromatic AE.plugin
```

Use the installer if the manually copied plugin is blocked by macOS download quarantine.

## Existing projects

The Mac build uses the exact Windows v2.2 effect match name, all 32 saved control IDs in the same order, and the same rendering source. Existing v2.0/v2.1 projects retain the compatible new defaults already provided by v2.2. Keyframes, custom channel colours, blur directions, centre restoration and adjustment layers use the same controls. Project media and fonts must also be available on the other machine. Open projects with the same or a newer compatible AE version; the plugin does not change AE's own project-file compatibility.

The binary deployment minimum is macOS 12. Your installed AE version may require a newer macOS release. No Rust, Xcode or Adobe SDK is required to install the finished ZIP.

## Validation and source build

The build runs the existing 20 rendering tests, checks the actual parameter setup against the recorded Windows control IDs, builds both architectures, verifies the universal bundle's signature/resources and loads its registration entry point. A separate Intel runner checks the same packaged binary. These checks do not replace an After Effects session on a Mac: AE itself is not available on the build runners.

For an AE-host check on your Mac, the ZIP includes `tests/native-suite.jsx`. Enable AE's scripting permission to write files, then run it in a separate empty project via **File > Scripts > Run Script File**. It creates test compositions and opens a bundled fixture, so do not run it with unsaved work. Reports go into the extracted `test-output` folder. This optional test is not needed for installation.

To build from source on macOS, install Xcode Command Line Tools and Rust 1.99.0, add the `x86_64-apple-darwin` and `aarch64-apple-darwin` Rust targets, and run `bash native/build-macos.sh`. The GitHub **Mac universal build** workflow performs the same steps. `BUILD-REPORT.txt` is included with the Mac package.
