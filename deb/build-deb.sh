#!/bin/bash
set -e

VERSION="${1:-0.3.1}"

echo "Building arexibo DEB for version ${VERSION}"

# Install build dependencies
export DEBIAN_FRONTEND=noninteractive
apt-get update
apt-get install -y \
    git \
    build-essential \
    cargo \
    cmake \
    g++ \
    libdbus-1-dev \
    libzmq3-dev \
    qt6-webengine-dev \
    debhelper \
    devscripts \
    dpkg-dev

# Determine version and release
if [[ "$VERSION" == *-* ]]; then
    DEB_VERSION="${VERSION%%-*}"
    DEB_RELEASE="${VERSION#*-}"
else
    DEB_VERSION="$VERSION"
    DEB_RELEASE="1"
fi

# Detect architecture
ARCH=$(dpkg --print-architecture)
echo "Building for architecture: ${ARCH}"

# Build Rust binary
echo "Building Rust binary"
export CARGO_NET_GIT_FETCH_WITH_CLI=true
cargo build --release

# Create DEB package directory structure
echo "Creating DEB package structure"
mkdir -p deb-pkg/arexibo/DEBIAN
mkdir -p deb-pkg/arexibo/usr/bin
mkdir -p deb-pkg/arexibo/usr/share/doc/arexibo

# Install files
echo "Installing files"
install -m755 target/release/arexibo deb-pkg/arexibo/usr/bin/arexibo
install -m644 LICENSE deb-pkg/arexibo/usr/share/doc/arexibo/
install -m644 README.md deb-pkg/arexibo/usr/share/doc/arexibo/
install -m644 CHANGELOG.md deb-pkg/arexibo/usr/share/doc/arexibo/

# Create control file
echo "Creating control file"
cat > deb-pkg/arexibo/DEBIAN/control << EOF
Package: arexibo
Version: ${DEB_VERSION}-${DEB_RELEASE}
Section: misc
Priority: optional
Architecture: ${ARCH}
Depends: libqt6webenginecore6, libqt6webenginewidgets6, libdbus-1-3, libzmq5
Maintainer: Pau Aliagas <pau@linuxnow.com>
Description: Rust-based digital signage player for Xibo CMS
 Arexibo is a Rust-based digital signage player compatible with Xibo CMS.
 It provides a lightweight alternative to the official Xibo player,
 designed for kiosk and digital signage deployments on Linux.
Homepage: https://github.com/linuxnow/arexibo
EOF

# Build DEB package
echo "Building DEB package"
mkdir -p dist
dpkg-deb --build deb-pkg/arexibo "dist/arexibo_${DEB_VERSION}-${DEB_RELEASE}_${ARCH}.deb"

echo "DEB build complete:"
ls -lh dist/*.deb
