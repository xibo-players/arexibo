#!/bin/bash
set -e

VERSION="${1:-0.3.1}"

echo "Building arexibo RPM for version ${VERSION}"

# Install build dependencies
dnf install -y \
    git \
    rust cargo \
    cmake gcc-c++ \
    dbus-devel zeromq-devel \
    qt6-qtwebengine-devel \
    rpm-build rpmdevtools

# Setup rpmbuild directory structure
rpmdev-setuptree

# Determine version and release
if [[ "$VERSION" == *-* ]]; then
    RPM_VERSION="${VERSION%%-*}"
    RPM_RELEASE="${VERSION#*-}"
else
    RPM_VERSION="$VERSION"
    RPM_RELEASE="1"
fi

# Create source tarball
echo "Creating source tarball for version ${RPM_VERSION}"
mkdir -p "arexibo-${RPM_VERSION}"
# Copy all files except hidden ones and target directory
cp -r Cargo.* *.md *.toml LICENSE src build.rs assets gui kiosk *.wsdl "arexibo-${RPM_VERSION}/" 2>/dev/null || true
tar -czf ~/rpmbuild/SOURCES/arexibo-${RPM_VERSION}.tar.gz "arexibo-${RPM_VERSION}"
rm -rf "arexibo-${RPM_VERSION}"

# Copy and update spec file
echo "Preparing spec file with Version: ${RPM_VERSION}, Release: ${RPM_RELEASE}"
sed -e "s/^Version:.*/Version:        ${RPM_VERSION}/" \
    -e "s/^Release:.*/Release:        ${RPM_RELEASE}%{?dist}/" \
    rpm/arexibo.spec > ~/rpmbuild/SPECS/arexibo.spec

# Build RPM
echo "Building RPM package"
rpmbuild -bb ~/rpmbuild/SPECS/arexibo.spec

# Copy built RPMs to dist directory
mkdir -p dist
find ~/rpmbuild/RPMS -name "*.rpm" -exec cp {} dist/ \;

echo "RPM build complete:"
ls -lh dist/*.rpm
