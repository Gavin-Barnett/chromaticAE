param([string]$AfterEffectsPath)
$ErrorActionPreference = 'Stop'
$panelPath = Join-Path $PSScriptRoot 'Chromatic AE.jsx'
if (-not (Test-Path -LiteralPath $panelPath)) { throw 'Keep this launcher beside Chromatic AE.jsx.' }
$aeExecutable = $null
if ($AfterEffectsPath) { $aeExecutable = Join-Path $AfterEffectsPath 'Support Files\AfterFX.exe' }
if (-not $aeExecutable) {
    $aeExecutable = Get-Process AfterFX -ErrorAction SilentlyContinue | Where-Object { $_.Path -and $_.MainWindowHandle -ne 0 } | Select-Object -First 1 -ExpandProperty Path
}
if (-not $aeExecutable) {
    $aeFolder = Get-ChildItem -LiteralPath (Join-Path $env:ProgramFiles 'Adobe') -Directory |
        Where-Object { $_.Name -like 'Adobe After Effects *' -and (Test-Path -LiteralPath (Join-Path $_.FullName 'Support Files\AfterFX.exe')) } |
        Sort-Object @{ Expression = { (Get-Item -LiteralPath (Join-Path $_.FullName 'Support Files\AfterFX.exe')).VersionInfo.FileMajorPart } } -Descending | Select-Object -First 1 -ExpandProperty FullName
    if ($aeFolder) { $aeExecutable = Join-Path $aeFolder 'Support Files\AfterFX.exe' }
}
if (-not $aeExecutable -or -not (Test-Path -LiteralPath $aeExecutable)) { throw 'After Effects was not found. Run the JSX from File > Scripts > Run Script File.' }
Start-Process -FilePath $aeExecutable -ArgumentList ('-r "' + $panelPath + '"') -WindowStyle Hidden
