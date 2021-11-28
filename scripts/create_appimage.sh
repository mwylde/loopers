#!/usr/bin/env bash
set -e

DIR=$(dirname "$(readlink -f "$0")")

TARGET=${DIR}/../target/appimage
mkdir -p "${TARGET}"
pushd "${TARGET}"

cargo build --manifest-path "../../Cargo.toml" --release

# binary
#mkdir -p AppDir/usr/bin
#cp release/loopers AppDir/usr/bin

# icon
#mkdir -p AppDir/usr/share/icons/default/512x512/apps
#cp ../loopers-gui/resources/icons/app-icon.png AppDir/usr/share/icons/default/512x512/apps/loopers.png
cp ../../loopers-gui/resources/icons/app-icon.png loopers.png

# desktop entry
# mkdir -p AppDir/usr/share/applications
cat > loopers.desktop <<EOF
[Desktop Entry]
Version=1.0
Name=Loopers
Exec=loopers
Icon=loopers
Type=Application
Terminal=false
Categories=AudioVideo;Audio;
Comment=Graphical live loopers
EOF

wget https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
chmod +x linuxdeploy-x86_64.AppImage
./linuxdeploy-x86_64.AppImage --appdir AppDir --output appimage \
-e ../release/loopers \
-i loopers.png \
-d loopers.desktop

popd
