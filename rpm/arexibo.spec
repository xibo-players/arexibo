%global debug_package %{nil}

Name:           arexibo
Version:        0.3.1
Release:        1%{?dist}
Summary:        Rust-based digital signage player for Xibo CMS

License:        AGPLv3+
URL:            https://github.com/xibo-players/arexibo
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust >= 1.75
BuildRequires:  cargo
BuildRequires:  cmake
BuildRequires:  gcc-c++
BuildRequires:  dbus-devel >= 1.6
BuildRequires:  zeromq-devel >= 4.1
BuildRequires:  qt6-qtwebengine-devel

Requires:       qt6-qtwebengine
Requires:       dbus
Requires:       zeromq

%description
Arexibo is a Rust-based digital signage player compatible with Xibo CMS.
It provides a lightweight alternative to the official Xibo player,
designed for kiosk and digital signage deployments on Linux.

%prep
%autosetup -n %{name}-%{version}

%build
export CARGO_NET_GIT_FETCH_WITH_CLI=true
cargo build --release

%install
install -Dm755 target/release/arexibo %{buildroot}%{_bindir}/arexibo
install -Dm644 arexibo.desktop \
    %{buildroot}%{_datadir}/applications/arexibo.desktop
install -Dm644 assets/arexibo-256.png \
    %{buildroot}%{_datadir}/icons/hicolor/256x256/apps/arexibo.png

%post
touch --no-create %{_datadir}/icons/hicolor &>/dev/null || :
gtk-update-icon-cache %{_datadir}/icons/hicolor &>/dev/null || :

%postun
if [ $1 -eq 0 ]; then
    touch --no-create %{_datadir}/icons/hicolor &>/dev/null
    gtk-update-icon-cache %{_datadir}/icons/hicolor &>/dev/null || :
fi

%files
%license LICENSE
%doc README.md CHANGELOG.md
%{_bindir}/arexibo
%{_datadir}/applications/arexibo.desktop
%{_datadir}/icons/hicolor/256x256/apps/arexibo.png

%changelog
* Tue Feb 18 2026 Pau Aliagas <pau@linuxnow.com> - 0.3.1-1
- Clean rebuild: PDF support, NotAuthorized exit code, dependency updates
- Thin workflow callers for RPM, DEB and image builds
