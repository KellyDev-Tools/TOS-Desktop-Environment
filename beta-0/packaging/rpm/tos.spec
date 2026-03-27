Name: tos
Version: 0.1.0_beta.0
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
cd beta-0 && make build-services

%install
mkdir -p %{buildroot}%{_bindir}
install -m 755 beta-0/target/release/tos-brain %{buildroot}%{_bindir}/tos-brain
install -m 755 beta-0/target/release/tos %{buildroot}%{_bindir}/tos
mkdir -p %{buildroot}%{_datadir}/xsessions
install -m 644 beta-0/packaging/tos.desktop %{buildroot}%{_datadir}/xsessions/

%files
%{_bindir}/tos-brain
%{_bindir}/tos
%{_datadir}/xsessions/tos.desktop
