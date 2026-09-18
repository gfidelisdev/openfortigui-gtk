Name:           openfortigui-gtk
Version:        0.1.0
Release:        1%{?dist}
Summary:        GTK frontend for openfortivpn, a Fortinet SSL-VPN client

License:        GPL-3.0-or-later
URL:            https://github.com/theinvisible/openfortigui
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  pkgconfig(gtk4)
BuildRequires:  pkgconfig(libadwaita-1)
Requires:       openfortivpn
Requires:       polkit

%description
openfortiGUI GTK is an independent Rust/GTK4 + libadwaita frontend for
managing Fortinet SSL-VPN profiles with openfortivpn.

Root privileges for starting/stopping the tunnel are obtained through a
dedicated PolicyKit action (auth_admin_keep), so the password is only asked
once per desktop session instead of on every connect/disconnect.

%prep
%setup -q

%build
cargo build --release

%install
rm -rf %{buildroot}
install -Dm755 target/release/%{name} %{buildroot}/opt/%{name}/%{name}
install -Dm644 assets/icon.svg %{buildroot}/opt/%{name}/icon.svg
install -Dm644 assets/org.openfortigui.Gtk.desktop %{buildroot}%{_datadir}/applications/org.openfortigui.Gtk.desktop
install -Dm755 polkit/openfortigui-privileged-helper %{buildroot}%{_prefix}/lib/%{name}/openfortigui-privileged-helper
install -Dm644 polkit/com.openfortigui.vpnhelper.policy %{buildroot}%{_datadir}/polkit-1/actions/com.openfortigui.vpnhelper.policy

%files
/opt/%{name}/%{name}
/opt/%{name}/icon.svg
%{_datadir}/applications/org.openfortigui.Gtk.desktop
%{_prefix}/lib/%{name}/openfortigui-privileged-helper
%{_datadir}/polkit-1/actions/com.openfortigui.vpnhelper.policy

%changelog
* Fri Sep 18 2026 Rene Hadler <rene@hadler.me> - 0.1.0-1
- Initial packaging: SRP module split, PolicyKit-based root auth
  (auth_admin_keep) and /opt install layout.
