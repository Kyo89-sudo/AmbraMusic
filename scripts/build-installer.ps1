param([switch]$SkipBuild, [string]$CacheDir)
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
Set-Location $root
if (!$SkipBuild) {
    $buildArgs = @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', (Join-Path $PSScriptRoot 'build-windows.ps1'))
    if ($CacheDir) { $buildArgs += @('-CacheDir', $CacheDir) }
    & powershell.exe @buildArgs
    if ($LASTEXITCODE -ne 0) { throw 'Build applicazione non riuscita.' }
}
$version = (Select-String -Path Cargo.toml -Pattern '^version = "(.+)"' | Select-Object -First 1).Matches[0].Groups[1].Value
$dist = Join-Path $root "dist\AmbraMusic-$version"
$exe = Join-Path $dist 'AmbraMusic.exe'
if (!(Test-Path $exe) -or !(Test-Path (Join-Path $dist 'libmpv-2.dll'))) { throw 'Eseguibile o motore audio assenti. Esegui prima la build release.' }
if ((Get-Item $exe).VersionInfo.ProductVersion -notlike "$version*") { throw 'La versione dell''eseguibile non corrisponde ai sorgenti.' }
$vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
$vs = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (!$vs) { throw 'Visual Studio con componenti C++ non trovato.' }
$redist = Get-ChildItem -LiteralPath (Join-Path $vs 'VC\Redist\MSVC') -Directory |
    Where-Object { $_.Name -match '^\d+\.\d+\.\d+$' } | Sort-Object { [Version]$_.Name } -Descending | Select-Object -First 1
$crt = Get-ChildItem -LiteralPath (Join-Path $redist.FullName 'x64') -Directory |
    Where-Object { (Test-Path (Join-Path $_.FullName 'vcruntime140.dll')) -and (Test-Path (Join-Path $_.FullName 'vcruntime140_1.dll')) } | Select-Object -First 1
if (!$crt) { throw 'Runtime Visual C++ x64 ridistribuibile non trovato.' }
foreach ($dll in @('vcruntime140.dll', 'vcruntime140_1.dll')) { Copy-Item -LiteralPath (Join-Path $crt.FullName $dll) -Destination $dist -Force }
$csc = Join-Path $env:WINDIR 'Microsoft.NET\Framework64\v4.0.30319\csc.exe'
if (!(Test-Path $csc)) { throw 'Compilatore .NET Framework x64 non disponibile.' }
$obj = Join-Path $root 'target\installer'
New-Item -ItemType Directory -Force $obj | Out-Null
$info = Join-Path $obj 'BuildInfo.cs'
$assemblyVersion = "$version.0"
function Write-BuildInfo([string]$PayloadHash) {
    "namespace AmbraMusic.Setup { internal static class BuildInfo { internal const string Version = `"$version`"; internal const string AssemblyVersion = `"$assemblyVersion`"; internal const string PayloadHash = `"$PayloadHash`"; } }" | Set-Content -LiteralPath $info -Encoding UTF8
}
$common = @('/nologo', '/target:winexe', '/platform:x64', '/optimize+', '/codepage:65001',
    '/r:System.dll', '/r:System.Core.dll', '/r:System.Windows.Forms.dll', '/r:System.Drawing.dll',
    '/r:System.IO.Compression.dll', '/r:System.IO.Compression.FileSystem.dll', '/r:Accessibility.dll',
    "/win32manifest:$root\installer\app.manifest", "/win32icon:$root\assets\icon.ico",
    "/resource:$root\assets\icon.ico,AmbraMusic.Icon", "/resource:$root\assets\icon-96.png,AmbraMusic.Logo",
    "/resource:$root\LICENSE,AmbraMusic.License", "/resource:$root\THIRD-PARTY.md,AmbraMusic.Credits")
$sources = @((Join-Path $root 'installer\Program.cs'), (Join-Path $root 'installer\InstallerEngine.cs'), $info)
Write-BuildInfo ''
& $csc @common '/define:UNINSTALLER' "/out:$dist\Uninstall.exe" @sources
if ($LASTEXITCODE -ne 0) { throw 'Compilazione del disinstallatore non riuscita.' }
Copy-Item (Join-Path $root 'README.md'), (Join-Path $root 'LICENSE'), (Join-Path $root 'THIRD-PARTY.md'), (Join-Path $root 'VERIFICA.md'), (Join-Path $root 'CHANGELOG.md'), (Join-Path $root "docs\RELEASE-$version.md") $dist -Force
New-Item -ItemType Directory -Force (Join-Path $dist 'docs') | Out-Null
Get-ChildItem -LiteralPath (Join-Path $root 'docs') | ForEach-Object { Copy-Item -LiteralPath $_.FullName -Destination (Join-Path $dist 'docs') -Recurse -Force }
if (!(Test-Path (Join-Path $root 'installer\licenses\mpv\SOURCE.md'))) { throw 'Informazioni sul motore audio mancanti.' }
New-Item -ItemType Directory -Force (Join-Path $dist 'licenses') | Out-Null
Get-ChildItem -LiteralPath (Join-Path $root 'installer\licenses') | ForEach-Object { Copy-Item -LiteralPath $_.FullName -Destination (Join-Path $dist 'licenses') -Recurse -Force }

# Package the actual license files distributed with each resolved Rust crate.
$metadataText = & cargo metadata --format-version 1 --locked --offline --filter-platform x86_64-pc-windows-msvc
if ($LASTEXITCODE -ne 0) { throw 'Raccolta metadati delle dipendenze non riuscita.' }
$metadata = ($metadataText -join "`n") | ConvertFrom-Json
$notices = @('# Dipendenze Rust', '', 'Versioni del lockfile e licenze dichiarate dai relativi pacchetti.', '', '| Pacchetto | Versione | Licenza | Origine |', '| --- | --- | --- | --- |')
foreach ($package in ($metadata.packages | Sort-Object name, version)) {
    $sourceUrl = if ($package.repository) { $package.repository } elseif ($package.source) { "https://crates.io/crates/$($package.name)/$($package.version)" } else { 'https://github.com/Kyo89-sudo/AmbraMusic' }
    $notices += "| $($package.name) | $($package.version) | $($package.license) | $sourceUrl |"
    $packageRoot = Split-Path $package.manifest_path -Parent
    $licenseFiles = @(Get-ChildItem -LiteralPath $packageRoot -File | Where-Object { $_.Name -match '^(LICENSE|LICENCE|COPYING|NOTICE|COPYRIGHT)([._-]|$)' })
    if ($package.license_file) { $licenseFiles += Get-Item -LiteralPath (Join-Path $packageRoot $package.license_file) }
    if ($licenseFiles.Count -eq 0) { continue }
    $licenseDir = Join-Path $dist "licenses\rust\$($package.name)-$($package.version)"
    New-Item -ItemType Directory -Force $licenseDir | Out-Null
    $licenseFiles | Select-Object -Unique FullName | ForEach-Object { Copy-Item -LiteralPath $_.FullName -Destination $licenseDir -Force }
}
$notices | Set-Content -LiteralPath (Join-Path $dist 'licenses\RUST-DEPENDENCIES.md') -Encoding UTF8
@(
    '# Microsoft Visual C++ Runtime', '',
    'vcruntime140.dll e vcruntime140_1.dll, componenti Microsoft redistribuibili per applicazioni Windows.',
    "Versione dei file: $((Get-Item (Join-Path $dist 'vcruntime140.dll')).VersionInfo.FileVersion)",
    'Copyright Microsoft Corporation. Le relative condizioni restano quelle del produttore.',
    'Distribuzione locale accanto all''eseguibile; nessuna modifica alle librerie di sistema.',
    'https://learn.microsoft.com/cpp/windows/redistributing-visual-cpp-files',
    'https://visualstudio.microsoft.com/license-terms/'
) | Set-Content -LiteralPath (Join-Path $dist 'licenses\MICROSOFT-VCRUNTIME.md') -Encoding UTF8
$payload = Join-Path $obj 'payload.zip'
Add-Type -AssemblyName System.IO.Compression.FileSystem
if (Test-Path $payload) { Remove-Item -LiteralPath $payload }
[IO.Compression.ZipFile]::CreateFromDirectory($dist, $payload, [IO.Compression.CompressionLevel]::Optimal, $false)
Write-BuildInfo ((Get-FileHash -LiteralPath $payload -Algorithm SHA256).Hash.ToLowerInvariant())
$output = Join-Path $root "dist\AmbraMusic-Setup-$version-x64.exe"
& $csc @common "/resource:$payload,AmbraMusic.Payload" "/out:$output" @sources
if ($LASTEXITCODE -ne 0) { throw 'Compilazione installer non riuscita.' }
Write-Host "Installer personalizzato pronto: $output"
Write-Host 'Esegui la verifica isolata prima della pubblicazione: Setup.exe --self-test <report.txt>'
