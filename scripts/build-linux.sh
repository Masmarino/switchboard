#!/usr/bin/env bash
# Build et lance le frontend natif Linux (GTK4 + libadwaita).
# Necessite les paquets de dev gtk4/libadwaita (ex: `apt install libgtk-4-dev libadwaita-1-dev`,
# ou `dnf install gtk4-devel libadwaita-devel`).
set -euo pipefail
cd "$(dirname "$0")/.."

cargo build -p switchboard-linux --release
./target/release/switchboard
