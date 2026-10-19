#!/bin/bash
set -euo pipefail
[[ "$(uname -s)" == Darwin ]] || { echo 'Build this package on macOS with Xcode Command Line Tools.' >&2; exit 1; }
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"
export MACOSX_DEPLOYMENT_TARGET=12.0
export CARGO_TARGET_DIR="$root/native/target"
version=2.2.0
package="$root/dist/Chromatic-AE-$version-macOS-Universal"
bundle="$package/plugin/Chromatic AE.plugin"
# Refuse to merge stale signed files into a new package.
[[ ! -e "$package" ]] || { echo "Move the existing package aside before rebuilding: $package" >&2; exit 1; }
mkdir -p "$root/dist"
exec > >(tee "$root/dist/mac-build-report.txt") 2>&1
rustc -Vv
sw_vers
uname -m
python3 tests/verify-v22-source.py
cargo test --locked --manifest-path native/Cargo.toml
cargo run --locked --release --manifest-path native/Cargo.toml --example project_compatibility
for target in x86_64-apple-darwin aarch64-apple-darwin; do
    rustup target add "$target"
    cargo build --locked --release --manifest-path native/Cargo.toml --target "$target"
done
mkdir -p "$bundle/Contents/MacOS" "$bundle/Contents/Resources"
intel="$CARGO_TARGET_DIR/x86_64-apple-darwin/release"
arm="$CARGO_TARGET_DIR/aarch64-apple-darwin/release"
cmp "$intel/chromatic-ae.rsrc" "$arm/chromatic-ae.rsrc"
cp "$intel/chromatic-ae.rsrc" "$bundle/Contents/Resources/Chromatic AE.rsrc"
cp "$intel/chromatic-ae_PkgInfo" "$bundle/Contents/PkgInfo"
cp "$intel/chromatic-ae_Info.plist" "$bundle/Contents/Info.plist"
lipo -create "$intel/libchromatic_ae.dylib" "$arm/libchromatic_ae.dylib" -output "$bundle/Contents/MacOS/Chromatic AE"
python3 - "$bundle/Contents/Info.plist" <<'PY'
import plistlib,sys
p=sys.argv[1]
with open(p,'rb') as f: info=plistlib.load(f)
info.update(CFBundleIdentifier='com.gavinbarnett.chromaticae',CFBundleExecutable='Chromatic AE',CFBundleName='Chromatic AE',CFBundleShortVersionString='2.2.0',CFBundleVersion='2.2.0',LSMinimumSystemVersion='12.0',CFBundleSupportedPlatforms=['MacOSX'])
with open(p,'wb') as f: plistlib.dump(info,f)
PY
chmod 755 "$bundle/Contents/MacOS/Chromatic AE"
codesign --force --sign - --timestamp=none "$bundle"
python3 tests/check-macos-bundle.py "$bundle"
cp 'Install Chromatic AE.command' MAC-INSTALL.md README.md LICENSE THIRD-PARTY-NOTICES.txt RUST-RUNTIME-NOTICES.html PERFORMANCE.md VALIDATION.md "$package/"
chmod 755 "$package/Install Chromatic AE.command"
mkdir -p "$package/examples" "$package/tests" "$package/test-output"
cp 'Create Demo.jsx' "$package/"
cp 'examples/Chromatic AE Native Demo.aep' examples/preview.jpg "$package/examples/"
cp tests/native-suite.jsx tests/native-integration.jsx tests/native-centre.jsx tests/native-bounds.jsx "$package/tests/"
mkdir -p "$package/tests/fixtures"
cp tests/fixtures/v2.0-demo.aep "$package/tests/fixtures/"
cp "$root/dist/mac-build-report.txt" "$package/BUILD-REPORT.txt"
(cd "$package" && find plugin -type f -exec shasum -a 256 '{}' \; > SHA256SUMS.txt)
ditto -c -k --sequesterRsrc --keepParent "$package" "$package.zip"
echo "Created $package.zip"
