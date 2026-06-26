#!/usr/bin/env bash
# Installe Switchboard pour l'utilisateur courant : binaire dans ~/.local/bin,
# icone dans le theme hicolor, et fichier .desktop pour que GNOME/KDE affichent
# le bon nom et la bonne icone (sinon ils retombent sur l'application-id brut
# "com.skolln.switchboard" et une icone generique).
set -euo pipefail
cd "$(dirname "$0")"

BIN_DIR="$HOME/.local/bin"
ICON_DIR="$HOME/.local/share/icons/hicolor/512x512/apps"
APPS_DIR="$HOME/.local/share/applications"

mkdir -p "$BIN_DIR" "$ICON_DIR" "$APPS_DIR"

cp bin/switchboard "$BIN_DIR/switchboard"
chmod +x "$BIN_DIR/switchboard"
cp icons/512x512.png "$ICON_DIR/com.skolln.switchboard.png"
cp com.skolln.switchboard.desktop "$APPS_DIR/"

update-desktop-database "$APPS_DIR" 2>/dev/null || true
gtk-update-icon-cache "$HOME/.local/share/icons/hicolor" 2>/dev/null || true

echo "Switchboard installe. Verifie que $BIN_DIR est dans ton PATH, puis lance-le depuis le menu d'applications ou via 'switchboard'."
