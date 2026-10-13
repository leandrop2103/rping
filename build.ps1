# Script de compilação automatizada para rping / ping
param (
    [switch]$Debug
)

$ErrorActionPreference = "Stop"
$root = $PSScriptRoot

if ($Debug) {
    Write-Host "[1/2] Compilando perfil Debug..." -ForegroundColor Cyan
    cargo build
    $binDir = Join-Path $root "target\debug"
} else {
    Write-Host "[1/2] Compilando perfil Release otimizado (LTO fat, strip, static CRT)..." -ForegroundColor Cyan
    cargo build --release
    $binDir = Join-Path $root "target\release"
}

Write-Host "[2/2] Copiando binários para a raiz do projeto..." -ForegroundColor Cyan
Copy-Item (Join-Path $binDir "rping.exe") (Join-Path $root "rping.exe") -Force
Copy-Item (Join-Path $binDir "rping.exe") (Join-Path $root "ping.exe") -Force

Write-Host "Sucesso! Binários salvos na raiz:" -ForegroundColor Green
Get-Item (Join-Path $root "rping.exe"), (Join-Path $root "ping.exe") | Select-Object Name, Length, LastWriteTime

if (-not $Debug) {
    $zipPath = Join-Path $root "rping-windows-x86_64.zip"
    Write-Host "[3/3] Gerando pacote ZIP para distribuição..." -ForegroundColor Cyan
    Compress-Archive -Path (Join-Path $root "rping.exe"), (Join-Path $root "ping.exe"), (Join-Path $root "instalar.ps1"), (Join-Path $root "instalar.cmd"), (Join-Path $root "README.md") -DestinationPath $zipPath -Force
    Get-Item $zipPath | Select-Object Name, Length, LastWriteTime
}
