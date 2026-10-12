param([string]$AfterEffectsPath)
$ErrorActionPreference = 'Stop'
$panelSource = Join-Path $PSScriptRoot 'Chromatic AE.jsx'
if (-not (Test-Path -LiteralPath $panelSource)) { throw 'Keep this installer beside Chromatic AE.jsx.' }
if (-not $AfterEffectsPath) {
    $AfterEffectsPath = Get-ChildItem -LiteralPath (Join-Path $env:ProgramFiles 'Adobe') -Directory |
        Where-Object { $_.Name -like 'Adobe After Effects *' -and (Test-Path -LiteralPath (Join-Path $_.FullName 'Support Files\AfterFX.exe')) } |
        Sort-Object @{ Expression = { (Get-Item -LiteralPath (Join-Path $_.FullName 'Support Files\AfterFX.exe')).VersionInfo.FileMajorPart } } -Descending | Select-Object -First 1 -ExpandProperty FullName
}
if (-not $AfterEffectsPath -or -not (Test-Path -LiteralPath (Join-Path $AfterEffectsPath 'Support Files\AfterFX.exe'))) {
    throw 'After Effects was not found. Pass -AfterEffectsPath with its installation folder, or install the JSX manually.'
}
$panelDirectory = Join-Path $AfterEffectsPath 'Support Files\Scripts\ScriptUI Panels'
$panelDestination = Join-Path $panelDirectory 'Chromatic AE.jsx'
try {
    New-Item -ItemType Directory -Path $panelDirectory -Force | Out-Null
    if (Test-Path -LiteralPath $panelDestination) {
        if ((Get-FileHash -LiteralPath $panelSource).Hash -eq (Get-FileHash -LiteralPath $panelDestination).Hash) {
            Write-Host 'Chromatic AE is already installed. Open it from the Window menu.'
            exit 0
        }
        Copy-Item -LiteralPath $panelDestination -Destination ($panelDestination + '.backup-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    }
    Copy-Item -LiteralPath $panelSource -Destination $panelDestination -Force
    Write-Host "Installed: $panelDestination"
    Write-Host 'Restart After Effects, then choose Window > Chromatic AE.jsx.'
} catch [System.UnauthorizedAccessException] {
    $isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    if ($isAdmin) { throw }
    $installerArgs = '-NoProfile -ExecutionPolicy Bypass -File "' + $PSCommandPath + '" -AfterEffectsPath "' + $AfterEffectsPath + '"'
    $installProcess = Start-Process -FilePath 'powershell.exe' -ArgumentList $installerArgs -Verb RunAs -WindowStyle Hidden -Wait -PassThru
    if ($installProcess.ExitCode -ne 0) { throw 'Installation did not complete. You can still use File > Scripts > Run Script File.' }
    Write-Host 'Installation complete. Restart After Effects and open Window > Chromatic AE.jsx.'
}
