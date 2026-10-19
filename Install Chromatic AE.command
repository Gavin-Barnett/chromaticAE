#!/bin/bash
set -euo pipefail
root="$(cd "$(dirname "$0")" && pwd)"
source_bundle="$root/plugin/Chromatic AE.plugin"
destination='/Library/Application Support/Adobe/Common/Plug-ins/7.0/MediaCore/Chromatic AE.plugin'
[[ "$(uname -s)" == Darwin ]] || { echo 'This installer is for macOS.' >&2; exit 1; }
[[ -d "$source_bundle" ]] || { echo 'Extract the entire release ZIP before installing.' >&2; exit 1; }
if pgrep -x 'After Effects' >/dev/null || pgrep -x aerender >/dev/null || pgrep -x aerendercore >/dev/null; then
    echo 'Save your work and close After Effects and aerender, then run this installer again.' >&2
    exit 1
fi
(cd "$root" && shasum -a 256 -c SHA256SUMS.txt)
codesign --verify --deep --strict "$source_bundle"
echo 'Installing Chromatic AE 2.2 for Apple Silicon and Intel Macs.'
echo 'The plugin uses a local ad-hoc signature, not Apple Developer ID notarization.'
echo 'This installer approves only this plugin by removing its download quarantine attribute.'
echo 'Windows v2.2 projects retain the same effect controls and animation.'
echo "Destination: $destination"
echo 'Enter your Mac administrator password if requested (typing is invisible).'
sudo -v
staging="$(mktemp -d "${TMPDIR:-/tmp}/chromaticae.XXXXXX")"
trap 'rm -rf "$staging"' EXIT
ditto "$source_bundle" "$staging/Chromatic AE.plugin"
xattr -dr com.apple.quarantine "$staging/Chromatic AE.plugin" 2>/dev/null || true
codesign --verify --deep --strict "$staging/Chromatic AE.plugin"
sudo mkdir -p "$(dirname "$destination")"
if [[ -e "$destination" ]]; then
    backup="/Library/Application Support/Chromatic AE/Backups/$(date +%Y%m%d-%H%M%S)-$$"
    sudo mkdir -p "$backup"
    sudo mv "$destination" "$backup/Chromatic AE.plugin"
    echo "Previous installation preserved at: $backup"
fi
sudo ditto "$staging/Chromatic AE.plugin" "$destination"
codesign --verify --deep --strict "$destination"
echo 'Installed. Open After Effects and choose Effect > Chromatic > Chromatic AE.'
if [[ -t 0 ]]; then read -r -p 'Press Return to close this installer.' _; fi
