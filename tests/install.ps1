$ErrorActionPreference = 'Stop'

$root = Split-Path $PSScriptRoot -Parent
$fixture = Join-Path ([IO.Path]::GetTempPath()) ([guid]::NewGuid().ToString())
$releaseName = 'thp-9.9.9-windows-x86_64'
$releaseDir = Join-Path $fixture $releaseName
$installDir = Join-Path $fixture 'install'
$archive = Join-Path $fixture "$releaseName.zip"
$previousVersion = $env:THP_VERSION
$previousInstallDir = $env:THP_INSTALL_DIR
try {
    New-Item -ItemType Directory -Path (Join-Path $releaseDir 'bin'), (Join-Path $installDir 'thp-notices') -Force | Out-Null
    Copy-Item (Join-Path $root 'target/debug/thp.exe') (Join-Path $releaseDir 'bin/thp.exe')
    Copy-Item (Join-Path $root 'target/debug/thp.exe') (Join-Path $installDir 'thp.exe')
    foreach ($notice in 'LICENSE', 'LICENSE-BINARY', 'LICENSING.md', 'THIRD-PARTY-NOTICES', 'INSTALL.md') {
        Set-Content (Join-Path $releaseDir $notice) $notice
    }
    Set-Content (Join-Path $installDir 'thp-notices/LICENSE') 'old LICENSE'
    Compress-Archive -Path $releaseDir -DestinationPath $archive
    $hash = (Get-FileHash -Algorithm SHA256 $archive).Hash
    Set-Content (Join-Path $fixture 'SHA256SUMS') "$hash  $releaseName.zip"

    function Invoke-WebRequest {
        param([string] $Uri, [string] $OutFile, [switch] $UseBasicParsing)
        Copy-Item (Join-Path $fixture ([IO.Path]::GetFileName($Uri))) $OutFile
    }
    $env:THP_VERSION = '9.9.9'
    $env:THP_INSTALL_DIR = $installDir
    $failure = $null
    try { & (Join-Path $root 'install.ps1') } catch { $failure = $_ }
    if (-not $failure) { throw 'installer accepted an incomplete notice set' }
    if ($failure.ToString() -notmatch 'NOTICE') { throw $failure }
    if ((Get-Content (Join-Path $installDir 'thp-notices/LICENSE') -Raw).Trim() -ne 'old LICENSE') {
        throw 'failed update replaced an existing notice'
    }
    $command = Get-Content (Join-Path $root 'README.md') | Where-Object { $_ -match 'raw.githubusercontent.com/thp-lang/thp/main/install.ps1' } | Select-Object -First 1
    function Invoke-RestMethod {
        [CmdletBinding()]
        param([string] $Uri)
        $script:fetched = $true
        Get-Content (Join-Path $root 'install.ps1') -Raw
    }
    $script:fetched = $false
    $env:THP_VERSION = 'invalid'
    $ErrorActionPreference = 'Continue'
    $scopeFailure = $null
    try { Invoke-Expression $command } catch { $scopeFailure = $_ }
    if (-not $script:fetched) { throw 'documented command did not fetch the installer' }
    if ($scopeFailure.ToString() -notmatch 'THP_VERSION') { throw $scopeFailure }
    if ($ErrorActionPreference -ne 'Continue') { throw 'documented command changed the caller error preference' }
    $ErrorActionPreference = 'Stop'
    Write-Host 'PowerShell installer smoke passed'
} finally {
    $env:THP_VERSION = $previousVersion
    $env:THP_INSTALL_DIR = $previousInstallDir
    Remove-Item $fixture -Recurse -Force -ErrorAction SilentlyContinue
}
