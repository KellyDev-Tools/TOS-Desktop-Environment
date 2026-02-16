Name: tos
Version: 0.1.0
Release: 1%{?dist}
Summary: Tactical Operating System
License: GPLv3+
URL: https://tos-project.org
Source0: %{name}-%{version}.tar.gz

BuildRequires: rust, cargo, libwayland-devel, libxkbcommon-devel
Requires: wayland, libxkbcommon

%description
TOS is a reimagining of the Linux desktop with a recursive zoom hierarchy
and command-first philosophy.

%build
cd tos-dream && cargo build --release

%install
mkdir -p %{buildroot}%{_bindir}
install -m 755 tos-dream/target/release/tos %{buildroot}%{_bindir}/tos-session
mkdir -p %{buildroot}%{_datadir}/xsessions
install -m 644 packaging/tos.desktop %{buildroot}%{_datadir}/xsessions/

%files
%{_bindir}/tos-session
%{_datadir}/xsessions/tos.desktop
