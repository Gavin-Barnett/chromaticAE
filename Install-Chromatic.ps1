param([string]$AfterEffectsPath, [switch]$Elevated)
$ErrorActionPreference = 'Stop'
$source = Join-Path $PSScriptRoot 'plugin\Chromatic AE.aex'
if (-not (Test-Path -LiteralPath $source)) { throw 'Extract the complete ZIP first. The plugin folder must be beside this installer.' }
if (-not $AfterEffectsPath) {
    $adobe = Join-Path $env:ProgramFiles 'Adobe'
    if (Test-Path -LiteralPath $adobe) {
        $AfterEffectsPath = Get-ChildItem -LiteralPath $adobe -Directory |
            Where-Object { $_.Name -like 'Adobe After Effects *' -and (Test-Path -LiteralPath (Join-Path $_.FullName 'Support Files\AfterFX.exe')) } |
            Sort-Object @{ Expression = { (Get-Item -LiteralPath (Join-Path $_.FullName 'Support Files\AfterFX.exe')).VersionInfo.FileMajorPart } } -Descending |
            Select-Object -First 1 -ExpandProperty FullName
    }
}
if (-not $AfterEffectsPath -or -not (Test-Path -LiteralPath (Join-Path $AfterEffectsPath 'Support Files\AfterFX.exe'))) {
    throw 'After Effects was not found. Supply -AfterEffectsPath with its installation folder, or copy plugin\Chromatic AE.aex manually into AE Support Files\Plug-ins.'
}
$directory = Join-Path $AfterEffectsPath 'Support Files\Plug-ins\Chromatic AE'
$destination = Join-Path $directory 'Chromatic AE.aex'
if ((Test-Path -LiteralPath $destination) -and (Get-FileHash -LiteralPath $source).Hash -eq (Get-FileHash -LiteralPath $destination).Hash) {
    Write-Host 'Chromatic AE 2.2 is already installed. Restart AE if it was open during installation.'
    exit 0
}
try {
    New-Item -ItemType Directory -Path $directory -Force | Out-Null
    Copy-Item -LiteralPath $source -Destination $destination -Force
    if ((Get-FileHash -LiteralPath $source).Hash -ne (Get-FileHash -LiteralPath $destination).Hash) { throw 'Installed file verification failed.' }
    Write-Host "Installed and verified: $destination"
    Write-Host 'Restart After Effects. Select an adjustment layer and choose Effect > Chromatic > Chromatic AE.'
} catch [System.UnauthorizedAccessException] {
    if ($Elevated) { throw }
    Write-Host 'Windows needs administrator permission to write the Plug-ins folder.'
    $arguments = '-NoProfile -ExecutionPolicy Bypass -File "' + $PSCommandPath + '" -AfterEffectsPath "' + $AfterEffectsPath + '" -Elevated'
    $process = Start-Process -FilePath 'powershell.exe' -ArgumentList $arguments -Verb RunAs -WindowStyle Hidden -Wait -PassThru
    if ($process.ExitCode -ne 0 -or -not (Test-Path -LiteralPath $destination)) { throw 'Installation failed or was cancelled. Close AE and run this installer as administrator.' }
    if ((Get-FileHash -LiteralPath $source).Hash -ne (Get-FileHash -LiteralPath $destination).Hash) { throw 'Installed file verification failed.' }
    Write-Host 'Installed and verified. Restart AE, then choose Effect > Chromatic > Chromatic AE.'
} catch [System.IO.IOException] {
    throw 'The plugin could not be replaced. Close After Effects and aerender, then run the installer again.'
}
