param([switch]$Demo, [switch]$Run)
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
Set-Location $root
$log = Join-Path $root 'COMPILAZIONE.log'
Start-Transcript -Path $log -Force | Out-Null
try {
    Write-Host '=== AmbraMusic - Windows x64 / Slint ===' -ForegroundColor DarkYellow
    $vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (!(Test-Path $vswhere)) { throw 'Installa Visual Studio 2026 / Build Tools con il carico Sviluppo di applicazioni desktop con C++ e Windows SDK.' }
    $vs = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if (!$vs) { throw 'Componenti C++ di Visual Studio non trovati. Abilita Sviluppo di applicazioni desktop con C++ nel Visual Studio Installer.' }
    Import-Module (Join-Path $vs 'Common7\Tools\Microsoft.VisualStudio.DevShell.dll')
    Enter-VsDevShell -VsInstallPath $vs -SkipAutomaticLocation -DevCmdArguments '-arch=x64 -host_arch=x64'
    $cargo = Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe'
    if (!(Test-Path $cargo)) { throw 'Installa Rust da https://rustup.rs/ (profilo stable MSVC x64), poi riavvia questo script.' }
    & (Join-Path (Split-Path $cargo) 'rustup.exe') target add x86_64-pc-windows-msvc
    if ($LASTEXITCODE -ne 0) { throw 'Installazione target Rust fallita.' }
    # Cache comune a tutte le versioni (fuori dalla cartella del progetto): libmpv si scarica una
    # volta sola e le librerie Rust già compilate si riusano quando estrai uno ZIP nuovo.
    $cache = Join-Path $env:LOCALAPPDATA 'AmbraMusic\build-cache'
    New-Item -ItemType Directory -Force $cache | Out-Null
    $env:CARGO_TARGET_DIR = Join-Path $cache 'target'
    $native = Join-Path $root 'native'
    New-Item -ItemType Directory -Force $native | Out-Null
    $dll = Join-Path $native 'libmpv-2.dll'
    $mpvName = 'libmpv-2-x86_64-20260610-git-304426c'
    $expected = 'af708edcdf6ed12ac7b2bb31b4cbeba0bf7d4eef345a715ef5de782c0ddd6671'
    $cachedDir = Join-Path $cache $mpvName
    $cachedDll = Join-Path $cachedDir 'libmpv-2.dll'
    if (!(Test-Path $cachedDll)) {
        $archive = Join-Path $cache "$mpvName.7z"
        $ok = (Test-Path $archive) -and ((Get-FileHash $archive -Algorithm SHA256).Hash.ToLowerInvariant() -eq $expected)
        if (!$ok) {
            $url = "https://github.com/SimoHypers/limusic/releases/download/deps/$mpvName.7z"
            Write-Host 'Scarico libmpv (solo la prima volta), la stessa libreria audio fissata dal progetto originale...'
            Invoke-WebRequest -UseBasicParsing -Uri $url -OutFile $archive
            if ((Get-FileHash $archive -Algorithm SHA256).Hash.ToLowerInvariant() -ne $expected) { Remove-Item $archive; throw 'Il controllo SHA256 di libmpv non corrisponde. Download annullato.' }
        }
        New-Item -ItemType Directory -Force $cachedDir | Out-Null
        $seven = Get-Command '7z.exe' -ErrorAction SilentlyContinue
        $sevenPath = if ($seven) { $seven.Source } else { $null }
        if (!$sevenPath -and (Test-Path "$env:ProgramFiles\7-Zip\7z.exe")) { $sevenPath = "$env:ProgramFiles\7-Zip\7z.exe" }
        if ($sevenPath) { & $sevenPath x $archive "-o$cachedDir" -y | Out-Null }
        else { & tar.exe -xf $archive -C $cachedDir }
        if ($LASTEXITCODE -ne 0 -or !(Test-Path $cachedDll)) { throw 'Estrazione non riuscita. Installa 7-Zip e riprova.' }
    } else {
        Write-Host "libmpv già presente nella cache ($cachedDir)."
    }
    Copy-Item $cachedDll $dll -Force
    $exports = & dumpbin.exe /nologo /exports $dll
    if ($LASTEXITCODE -ne 0) { throw 'dumpbin non riesce a leggere libmpv.' }
    $names = @($exports | Select-String -Pattern '^\s+\d+\s+[0-9A-Fa-f]+\s+[0-9A-Fa-f]+\s+(\w+)' | ForEach-Object { $_.Matches[0].Groups[1].Value })
    if ($names.Count -lt 50) { throw 'Non riesco a ricavare le funzioni esportate da libmpv.' }
    $def = Join-Path $native 'mpv.def'
    @('EXPORTS') + ($names | ForEach-Object { "    $_" }) | Set-Content $def -Encoding ascii
    & lib.exe /nologo "/def:$def" /name:libmpv-2.dll "/out:$native\mpv.lib" /machine:x64
    if ($LASTEXITCODE -ne 0) { throw 'Creazione mpv.lib fallita.' }
    # Senza --locked: la 0.2 aggiunge wry/tao e Cargo.lock va completato al primo avvio.
    & $cargo build --release --target x86_64-pc-windows-msvc
    if ($LASTEXITCODE -ne 0) { throw 'Compilazione fallita: consulta COMPILAZIONE.log.' }
    # La cartella di distribuzione prende la versione da Cargo.toml.
    $ver = (Select-String -Path (Join-Path $root 'Cargo.toml') -Pattern '^version = "(.+)"' | Select-Object -First 1).Matches[0].Groups[1].Value
    $dist = Join-Path $root "dist\AmbraMusic-$ver"
    New-Item -ItemType Directory -Force $dist | Out-Null
    Copy-Item (Join-Path $env:CARGO_TARGET_DIR 'x86_64-pc-windows-msvc\release\AmbraMusic.exe') $dist -Force
    Copy-Item $dll $dist -Force
    Copy-Item "$root\README.md", "$root\LICENSE", "$root\THIRD-PARTY.md" $dist -Force
    '@echo off', 'cd /d "%~dp0"', 'start "" "%~dp0AmbraMusic.exe" --demo' | Set-Content "$dist\AVVIA-DEMO.cmd" -Encoding ascii
    Write-Host "Pronto: $dist\AmbraMusic.exe" -ForegroundColor Green
    if ($Demo) { Start-Process "$dist\AmbraMusic.exe" -ArgumentList '--demo' }
    elseif ($Run) { Start-Process "$dist\AmbraMusic.exe" }
}
catch { Write-Host $_.Exception.Message -ForegroundColor Red; exit 1 }
finally { Stop-Transcript | Out-Null }
