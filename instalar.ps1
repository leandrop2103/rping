# ==============================================================================
# Script de Instalação Automática do rping no Windows
# Copia o rping.exe para C:\Windows com elevação UAC automática
# ==============================================================================

$currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
$isAdmin = $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if (-not $isAdmin) {
    Write-Host "[*] Solicitando privilégios de Administrador (UAC)..." -ForegroundColor Yellow
    $scriptPath = $MyInvocation.MyCommand.Definition
    if (-not $scriptPath) {
        $scriptPath = $PSCommandPath
    }

    try {
        $psi = New-Object System.Diagnostics.ProcessStartInfo
        $psi.FileName = "powershell.exe"
        $psi.Arguments = "-NoProfile -ExecutionPolicy Bypass -File `"$scriptPath`""
        $psi.Verb = "runas"
        $psi.UseShellExecute = $true
        [System.Diagnostics.Process]::Start($psi) | Out-Null
        exit 0
    } catch {
        Write-Host "[ERRO] Permissão de Administrador negada ou cancelada pelo usuário." -ForegroundColor Red
        Read-Host "Pressione ENTER para sair..."
        exit 1
    }
}

Clear-Host
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "          Instalação do rping (Ping em Rust)              " -ForegroundColor White
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host ""

$sourceDir = $PSScriptRoot
if (-not $sourceDir) {
    $sourceDir = Split-Path -Parent $PSCommandPath
}

$sourceExe = Join-Path $sourceDir "rping.exe"
$destFolder = $env:SystemRoot
$destExe = Join-Path $destFolder "rping.exe"

if (-not (Test-Path $sourceExe)) {
    Write-Host "[ERRO] Arquivo 'rping.exe' não foi encontrado na pasta:" -ForegroundColor Red
    Write-Host "       $sourceDir" -ForegroundColor Red
    Write-Host ""
    Read-Host "Pressione ENTER para fechar..."
    exit 1
}

try {
    Write-Host "[1/2] Copiando rping.exe para $destFolder..." -ForegroundColor Cyan
    Copy-Item -Path $sourceExe -Destination $destExe -Force

    if (Test-Path $destExe) {
        $hashSource = (Get-FileHash $sourceExe -Algorithm SHA256).Hash
        $hashDest = (Get-FileHash $destExe -Algorithm SHA256).Hash

        if ($hashSource -eq $hashDest) {
            Write-Host "[2/2] Integridade validada com sucesso (SHA-256 verificado)!" -ForegroundColor Green
        }
    }

    Write-Host ""
    Write-Host "SUCESSO! O rping foi instalado globalmente no sistema!" -ForegroundColor Green
    Write-Host "Destino: $destExe" -ForegroundColor Gray
    Write-Host ""
    Write-Host "Agora você pode abrir qualquer terminal (CMD, PowerShell ou Terminal)" -ForegroundColor White
    Write-Host "e executar diretamente:" -ForegroundColor White
    Write-Host "   rping 1.1.1.1" -ForegroundColor Yellow
    Write-Host "   rping -th 8.8.8.8" -ForegroundColor Yellow
    Write-Host "   rping -n 4 google.com" -ForegroundColor Yellow
    Write-Host ""
} catch {
    Write-Host "[ERRO] Falha ao copiar o arquivo para $destFolder : $_" -ForegroundColor Red
}

Write-Host "==========================================================" -ForegroundColor Cyan
Read-Host "Pressione ENTER para finalizar..."
