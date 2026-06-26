.PHONY: macos linux windows test

macos:
	./scripts/build-macos.sh

linux:
	./scripts/build-linux.sh

windows:
	@echo "Sur Windows, lance scripts/build-windows.ps1 depuis PowerShell."

test:
	cargo test -p switchboard-core -p switchboard-ffi -p switchboard-linux
