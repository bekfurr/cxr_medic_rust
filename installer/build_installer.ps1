# BMI CXR-AI v2 - One-Click Installer Builder
# This script compiles the Rust release executable and builds the Inno Setup installer.

$ErrorActionPreference = "Stop"

Write-Host "==================================================" -ForegroundColor Cyan
Write-Host "   BMI CXR-AI v2 - Windows Installer Generator    " -ForegroundColor Cyan
Write-Host "==================================================" -ForegroundColor Cyan

# 1. Compile Release Binary
Write-Host "`n[1/2] Compiling Rust release binary..." -ForegroundColor Yellow
$env:RUST_MIN_STACK = "16777216"
$projectRoot = Resolve-Path "$PSScriptRoot\.."
Push-Location $projectRoot
try {
    cargo build --release -j 1
} finally {
    Pop-Location
}

# 2. Locate ISCC.exe
Write-Host "`n[2/2] Compiling Inno Setup installer..." -ForegroundColor Yellow
$isccCandidates = @(
    "C:\Program Files\Inno Setup 7\ISCC.exe",
    "C:\Program Files (x86)\Inno Setup 7\ISCC.exe",
    "C:\Program Files\Inno Setup 6\ISCC.exe",
    "C:\Program Files (x86)\Inno Setup 6\ISCC.exe"
)

$isccPath = $isccCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1

if (-not $isccPath) {
    $cmd = Get-Command ISCC.exe -ErrorAction SilentlyContinue
    if ($cmd) { $isccPath = $cmd.Source }
}

if (-not $isccPath) {
    Write-Host "XATOLIK: Inno Setup (ISCC.exe) kompyuterda topilmadi!" -ForegroundColor Red
    Write-Host "Iltimos, Inno Setup 6 yoki 7 ni o'rnating: https://jrsoftware.org/isdl.php" -ForegroundColor Red
    exit 1
}

Write-Host "Inno Setup topildi: $isccPath" -ForegroundColor Gray
& $isccPath "$PSScriptRoot\setup.iss"

Write-Host "`n==================================================" -ForegroundColor Green
Write-Host " MUVAFFAQIYATLI YAKUNLANDI!" -ForegroundColor Green
Write-Host " Tayyor o'rnatuvchi fayli: $PSScriptRoot\Output\BMI_CXR_AI_v2_Setup_v1.0.0.exe" -ForegroundColor Green
Write-Host "==================================================" -ForegroundColor Green

