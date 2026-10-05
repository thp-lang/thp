$ErrorActionPreference = 'Stop'

if ([Environment]::OSVersion.Platform -ne 'Win32NT' -or -not [Environment]::Is64BitOperatingSystem) {
    throw 'This installer requires 64-bit Windows.'
}

$version = $env:THP_VERSION
if (-not $version) {
    $releases = Invoke-RestMethod 'https://api.github.com/repos/thp-lang/thp/releases?per_page=100'
    $release = $releases | Where-Object { $_.tag_name -match '^v\d+\.\d+\.\d+$' -and -not $_.prerelease } | Select-Object -First 1
    if (-not $release) { throw 'No stable THP release found.' }
    $version = $release.tag_name.Substring(1)
}
if ($version -notmatch '^\d+\.\d+\.\d+$') { throw 'THP_VERSION must be a stable version such as 0.8.0.' }

$archive = "thp-$version-windows-x86_64.zip"
$base = "https://github.com/thp-lang/thp/releases/download/v$version"
$tempDir = Join-Path ([IO.Path]::GetTempPath()) ([guid]::NewGuid().ToString())
New-Item -ItemType Directory -Path $tempDir | Out-Null
try {
    $archivePath = Join-Path $tempDir $archive
    $checksumsPath = Join-Path $tempDir 'SHA256SUMS'
    Invoke-WebRequest "$base/$archive" -OutFile $archivePath -UseBasicParsing
    Invoke-WebRequest "$base/SHA256SUMS" -OutFile $checksumsPath -UseBasicParsing

    $line = Get-Content $checksumsPath | Where-Object { ($_ -split '\s+', 2)[1] -eq $archive } | Select-Object -First 1
    if (-not $line) { throw "Checksum missing for $archive." }
    $expected = ($line -split '\s+')[0]
    if ($expected -notmatch '^[0-9a-fA-F]{64}$') { throw "Invalid checksum for $archive." }
    $actual = (Get-FileHash -Algorithm SHA256 $archivePath).Hash
    if ($actual -ne $expected) { throw "Checksum mismatch for $archive." }

    Expand-Archive $archivePath -DestinationPath $tempDir
    $releaseDir = Join-Path $tempDir "thp-$version-windows-x86_64"
    $binDir = if ($env:THP_INSTALL_DIR) { $env:THP_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA 'Programs\THP' }
    if (-not [IO.Path]::IsPathRooted($binDir)) { throw 'THP_INSTALL_DIR must be an absolute path.' }
    New-Item -ItemType Directory -Path $binDir -Force | Out-Null
    $staged = Join-Path $binDir ('.thp-' + [guid]::NewGuid().ToString() + '.exe')
    try {
        Copy-Item (Join-Path $releaseDir 'bin\thp.exe') $staged
        & $staged --version
        if ($LASTEXITCODE -ne 0) { throw 'Downloaded thp.exe failed to start.' }
        $noticeDir = Join-Path $binDir 'thp-notices'
        $stagedNotices = Join-Path $binDir ('.thp-notices-' + [guid]::NewGuid().ToString())
        $oldNotices = Join-Path $binDir ('.thp-notices-old-' + [guid]::NewGuid().ToString())
        New-Item -ItemType Directory -Path $stagedNotices | Out-Null
        try {
            'LICENSE', 'LICENSE-BINARY', 'LICENSING.md', 'NOTICE', 'THIRD-PARTY-NOTICES', 'INSTALL.md' |
                ForEach-Object { Copy-Item (Join-Path $releaseDir $_) $stagedNotices }
            $hadNotices = Test-Path $noticeDir
            if ($hadNotices) { Move-Item $noticeDir $oldNotices }
            try {
                Move-Item $stagedNotices $noticeDir
                $installed = Join-Path $binDir 'thp.exe'
                if (Test-Path $installed) {
                    [IO.File]::Replace($staged, $installed, $null)
                } else {
                    [IO.File]::Move($staged, $installed)
                }
            } catch {
                if (Test-Path $noticeDir) { Remove-Item $noticeDir -Recurse -Force }
                if ($hadNotices) { Move-Item $oldNotices $noticeDir }
                throw
            }
            if ($hadNotices) { Remove-Item $oldNotices -Recurse -Force -ErrorAction SilentlyContinue }
        } finally {
            Remove-Item $stagedNotices -Recurse -Force -ErrorAction SilentlyContinue
        }
    } finally {
        Remove-Item $staged -Force -ErrorAction SilentlyContinue
    }

    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    if ($binDir -notin ($userPath -split ';')) {
        [Environment]::SetEnvironmentVariable('Path', "$binDir;$userPath", 'User')
    }
    if ($binDir -notin ($env:Path -split ';')) { $env:Path = "$binDir;$env:Path" }
    Write-Host 'Open a new terminal to use thp from PATH.'
} finally {
    Remove-Item $tempDir -Recurse -Force
}
