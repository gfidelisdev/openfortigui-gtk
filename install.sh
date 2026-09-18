#!/usr/bin/env bash
#
# Install openfortigui-gtk under /opt, register its desktop launcher and
# install the polkit privileged helper + policy action it needs at runtime.
#
# Usage: sudo ./install.sh
#
# Expects `cargo build --release` to have already been run as the normal user
# (not as root, so target/ stays owned by that user) and expects the desktop
# entry to already exist at assets/<APP_ID>.desktop (Exec=/opt/.../openfortigui-gtk,
# Icon=/opt/.../icon.svg).

set -euo pipefail

APP_ID="org.openfortigui.Gtk"
INSTALL_DIR="/opt/openfortigui-gtk"
APPLICATIONS_DIR="/usr/share/applications"
POLKIT_HELPER_DIR="/usr/lib/openfortigui-gtk"
POLKIT_ACTIONS_DIR="/usr/share/polkit-1/actions"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINARY="$SCRIPT_DIR/target/release/openfortigui-gtk"
ICON="$SCRIPT_DIR/assets/icon.svg"
DESKTOP_FILE="$SCRIPT_DIR/assets/$APP_ID.desktop"
POLKIT_HELPER="$SCRIPT_DIR/polkit/openfortigui-privileged-helper"
POLKIT_POLICY="$SCRIPT_DIR/polkit/com.openfortigui.vpnhelper.policy"

log() { printf '==> %s\n' "$*"; }
die() { printf 'install.sh: %s\n' "$*" >&2; exit 1; }

[[ "$(id -u)" -eq 0 ]] || die "must run as root (sudo ./install.sh), to write to $INSTALL_DIR and $APPLICATIONS_DIR"
[[ -f "$BINARY" ]] || die "$BINARY not found -- run 'cargo build --release' first (as your normal user)"
[[ -f "$ICON" ]] || die "$ICON not found"
[[ -f "$DESKTOP_FILE" ]] || die "$DESKTOP_FILE not found -- create it with Exec=$INSTALL_DIR/openfortigui-gtk and Icon=$INSTALL_DIR/icon.svg"
[[ -f "$POLKIT_HELPER" ]] || die "$POLKIT_HELPER not found"
[[ -f "$POLKIT_POLICY" ]] || die "$POLKIT_POLICY not found"

log "installing binary and icon to $INSTALL_DIR"
install -d -m 755 "$INSTALL_DIR"
install -m 755 "$BINARY" "$INSTALL_DIR/openfortigui-gtk"
install -m 644 "$ICON" "$INSTALL_DIR/icon.svg"

log "linking launcher into $APPLICATIONS_DIR"
install -d -m 755 "$APPLICATIONS_DIR"
ln -sf "$DESKTOP_FILE" "$APPLICATIONS_DIR/$APP_ID.desktop"

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$APPLICATIONS_DIR" >/dev/null 2>&1 || true
fi

log "installing polkit privileged helper and policy action"
install -Dm755 "$POLKIT_HELPER" "$POLKIT_HELPER_DIR/openfortigui-privileged-helper"
install -Dm644 "$POLKIT_POLICY" "$POLKIT_ACTIONS_DIR/com.openfortigui.vpnhelper.policy"

log "done: $INSTALL_DIR/openfortigui-gtk"
