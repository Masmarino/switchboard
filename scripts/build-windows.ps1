# Build le frontend natif Windows (WinUI3) et le lance.
# Non teste sur une vraie machine Windows — a verifier/corriger au premier build reel.
$ErrorActionPreference = "Stop"
Set-Location "$PSScriptRoot\.."

cargo build -p switchboard-ffi --release --target x86_64-pc-windows-msvc

Push-Location windows
dotnet build -c Release
Pop-Location

$exe = Get-ChildItem "windows\bin\Release" -Recurse -Filter "Switchboard.exe" | Select-Object -First 1
if (-not $exe) {
    Write-Error "Switchboard.exe introuvable apres build."
    exit 1
}

Start-Process $exe.FullName
