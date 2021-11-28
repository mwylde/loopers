#!/usr/bin/env bash
set -e

DIR=$(dirname "$0")

TARGET=${DIR}/../target/macos
mkdir -p "${TARGET}"
pushd "${TARGET}"

cargo build --manifest-path "../../Cargo.toml" --release

mkdir Loopers.app
mkdir -p Loopers.app/Contents/MacOS
mkdir -p Loopers.app/Contents/Resources

# create iconset
cp ../../loopers-gui/resources/icons/app-icon.png .
mkdir Loopers.iconset
sips -z 16 16     app-icon.png --out Loopers.iconset/icon_16x16.png
sips -z 32 32     app-icon.png --out Loopers.iconset/icon_16x16@2x.png
sips -z 32 32     app-icon.png --out Loopers.iconset/icon_32x32.png
sips -z 64 64     app-icon.png --out Loopers.iconset/icon_32x32@2x.png
sips -z 128 128   app-icon.png --out Loopers.iconset/icon_128x128.png
sips -z 256 256   app-icon.png --out Loopers.iconset/icon_128x128@2x.png
sips -z 256 256   app-icon.png --out Loopers.iconset/icon_256x256.png
cp app-icon.png Loopers.iconset/icon_256x256@2x.png
cp app-icon.png Loopers.iconset/icon_512x512.png
iconutil -c icns Loopers.iconset
mv Loopers.icns Loopers.app/Contents/Resources

# copy binary
cp ../release/loopers Loopers.app/Contents/MacOS

# create plist
cat > Loopers.app/Contents/Info.plist <<EOF
CFBundleName = loopers;
CFBundleDisplayName = Loopers;
CFBundleIdentifier = "com.micahw.loopers";
CFBundleVersion = "${VERSION}";
CFBundleShortVersionString = "${VERSION}";
CFBundleInfoDictionaryVersion = "6.0";
CFBundlePackageType = APPL;
CFBundleExectuable = loopers;
CFBundleIconFile = "Loopers.icns";
NSHighResolutionCapable = true;
EOF

# create DMG
hdiutil create -fs HFS+ -volname "Loopers" -srcfolder "Loopers.app" "Loopers-${VERSION}.dmg"

popd