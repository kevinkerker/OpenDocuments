# OpenDocuments installer for Windows (x86_64)
# Usage: irm https://raw.githubusercontent.com/kevinkerker/OpenDocuments/main/install.ps1 | iex
# Installs opendoc.exe and opendoc-engine-lancedb.exe into %LOCALAPPDATA%\Programs\OpenDocuments
# and adds that folder to the user PATH. No administrator rights required.
# OPENDOC_RELEASE_URL overrides the download base (e.g. a fork's release).

& {
    $ErrorActionPreference = 'Stop'
    $ProgressPreference = 'SilentlyContinue'  # Invoke-WebRequest is very slow with the progress bar in PowerShell 5.1
    [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12

    $repo = 'kevinkerker/OpenDocuments'
    $base = if ($env:OPENDOC_RELEASE_URL) { $env:OPENDOC_RELEASE_URL.TrimEnd('/') } else { "https://github.com/$repo/releases/latest/download" }
    $asset = 'opendoc-windows-x86_64.zip'
    $installDir = Join-Path $env:LOCALAPPDATA 'Programs\OpenDocuments'

    if (-not [Environment]::Is64BitOperatingSystem) {
        throw 'OpenDocuments requires 64-bit Windows.'
    }

    $tmp = Join-Path ([IO.Path]::GetTempPath()) ("opendoc-install-" + [Guid]::NewGuid())
    New-Item -ItemType Directory -Path $tmp | Out-Null
    try {
        Write-Host "Downloading $asset ..."
        $zip = Join-Path $tmp $asset
        Invoke-WebRequest -UseBasicParsing -Uri "$base/$asset" -OutFile $zip
        Invoke-WebRequest -UseBasicParsing -Uri "$base/checksums.txt" -OutFile (Join-Path $tmp 'checksums.txt')

        # checksums.txt lines: "<sha256>  <name>" (binary mode may prefix the name with '*')
        $expected = $null
        foreach ($line in Get-Content (Join-Path $tmp 'checksums.txt')) {
            if ($line -match "^\s*([0-9a-fA-F]{64})\s+\*?$([regex]::Escape($asset))\s*$") { $expected = $Matches[1].ToLower(); break }
        }
        if (-not $expected) { throw "No checksum for $asset in checksums.txt." }
        $actual = (Get-FileHash -Algorithm SHA256 $zip).Hash.ToLower()
        if ($expected -ne $actual) { throw "Checksum mismatch for $asset (expected $expected, got $actual)." }
        Write-Host 'Checksum verified.'

        $running = Get-Process -Name opendoc, opendoc-engine-lancedb -ErrorAction SilentlyContinue |
            Where-Object { $_.Path -and $_.Path.StartsWith($installDir, [StringComparison]::OrdinalIgnoreCase) }
        if ($running) { throw "OpenDocuments is running from $installDir. Stop it and run the installer again." }

        Expand-Archive -Path $zip -DestinationPath (Join-Path $tmp 'x') -Force
        New-Item -ItemType Directory -Force -Path $installDir | Out-Null
        Copy-Item (Join-Path $tmp 'x\*.exe') $installDir -Force
    }
    finally {
        Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
    }

    # Edit the registry directly: [Environment]::SetEnvironmentVariable would rewrite PATH as REG_SZ
    # with %VARIABLES% expanded.
    $envKey = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Environment', $true)
    try {
        $rawPath = [string]$envKey.GetValue('Path', '', [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)
        $entries = @($rawPath -split ';' | Where-Object { $_ })
        if ($entries -notcontains $installDir) {
            $envKey.SetValue('Path', (($entries + $installDir) -join ';'), [Microsoft.Win32.RegistryValueKind]::ExpandString)
            # Setting and removing a dummy variable broadcasts WM_SETTINGCHANGE so new terminals see the change.
            [Environment]::SetEnvironmentVariable('OPENDOC_INSTALL_REFRESH', '1', 'User')
            [Environment]::SetEnvironmentVariable('OPENDOC_INSTALL_REFRESH', $null, 'User')
            Write-Host "Added $installDir to the user PATH (new terminals pick it up)."
        }
    }
    finally {
        $envKey.Close()
    }
    if (($env:Path -split ';') -notcontains $installDir) { $env:Path = "$env:Path;$installDir" }

    Write-Host ''
    Write-Host "OpenDocuments installed to $installDir"
    Write-Host 'Start it with:  opendoc start   (then open http://127.0.0.1:3000)'
    Write-Host 'The first start without an LLM provider downloads the local embedding model once.'
}
