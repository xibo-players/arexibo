%global debug_package %{nil}

Name:           arexibo
Version:        0.3.1
Release:        1%{?dist}
Summary:        Rust-based digital signage player for Xibo CMS

License:        AGPLv3+
URL:            https://github.com/linuxnow/arexibo
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

%package        kiosk
Summary:        Kiosk session scripts for Xibo players
BuildArch:      noarch
Requires:       gnome-kiosk-script-session
Requires:       dunst
Requires:       unclutter
Requires:       zenity
Requires:       opendoas
Requires:       keyd
Requires:       mesa-va-drivers
Requires:       libva
Recommends:     libva-intel-driver
Provides:       xibo-kiosk = %{version}-%{release}
Provides:       arexibo-kiosk = %{version}-%{release}

%description    kiosk
Kiosk session scripts for running Xibo digital signage players as full-screen 
displays under GNOME Kiosk. Includes a first-boot registration wizard,
session holder with health monitoring, dunst notification config, and
a systemd user unit for the player process.

Supports multiple Xibo player implementations:
- arexibo (Rust-based)
- xiboplayer-electron (@xibo-players/xiboplayer-electron)
- xiboplayer-chromium (@xibo-players/xiboplayer-chromium)

%install
install -Dm755 target/release/arexibo %{buildroot}%{_bindir}/arexibo

# Kiosk scripts
install -Dm755 kiosk/gnome-kiosk-script.xibo.sh %{buildroot}%{_datadir}/xibo-kiosk/gnome-kiosk-script.xibo.sh
install -Dm755 kiosk/gnome-kiosk-script.xibo-init.sh %{buildroot}%{_datadir}/xibo-kiosk/gnome-kiosk-script.xibo-init.sh
install -Dm644 kiosk/dunstrc %{buildroot}%{_datadir}/xibo-kiosk/dunstrc
install -Dm644 kiosk/xibo-player.service %{buildroot}%{_userunitdir}/xibo-player.service
install -Dm755 kiosk/xibo-keyd-run.sh %{buildroot}%{_datadir}/xibo-kiosk/xibo-keyd-run.sh
install -Dm755 kiosk/xibo-show-ip.sh %{buildroot}%{_datadir}/xibo-kiosk/xibo-show-ip.sh
install -Dm755 kiosk/xibo-show-cms.sh %{buildroot}%{_datadir}/xibo-kiosk/xibo-show-cms.sh
install -Dm644 kiosk/keyd-xibo.conf %{buildroot}%{_sysconfdir}/keyd/xibo.conf

%files
%license LICENSE
%doc README.md CHANGELOG.md
%{_bindir}/arexibo

%files          kiosk
%dir %{_datadir}/xibo-kiosk
%{_datadir}/xibo-kiosk/gnome-kiosk-script.xibo.sh
%{_datadir}/xibo-kiosk/gnome-kiosk-script.xibo-init.sh
%{_datadir}/xibo-kiosk/dunstrc
%{_datadir}/xibo-kiosk/xibo-keyd-run.sh
%{_datadir}/xibo-kiosk/xibo-show-ip.sh
%{_datadir}/xibo-kiosk/xibo-show-cms.sh
%{_userunitdir}/xibo-player.service
%{_sysconfdir}/keyd/xibo.conf

%changelog
* Mon Jan 27 2026 Pau Aliagas <pau@linuxnow.com> - 0.3.1-2
- Add arexibo-kiosk subpackage with session scripts and systemd unit
- Fix exit code on error (exit 1 instead of 0)

* Sat Jan 18 2026 Pau Aliagas <pau@linuxnow.com> - 0.3.1-1
- Initial RPM package
