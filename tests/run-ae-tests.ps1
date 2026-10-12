param([string]$AfterFX = 'C:\Program Files\Adobe\Adobe After Effects 2026\Support Files\AfterFX.exe')
$ErrorActionPreference = 'Stop'
$workspacePath = Split-Path $PSScriptRoot -Parent
$outputDirectory = Join-Path $workspacePath 'test-output'
New-Item -ItemType Directory -Path $outputDirectory -Force | Out-Null
$reportPath = Join-Path $outputDirectory 'results.txt'
$runStarted = Get-Date
$testProcess = Start-Process -FilePath $AfterFX -ArgumentList ('-noui -m -r "' + (Join-Path $PSScriptRoot 'integration.jsx') + '"') -WindowStyle Hidden -PassThru
if (-not $testProcess.WaitForExit(180000)) { throw 'AE did not finish within 3 minutes. Inspect the disposable test process.' }
if (-not (Test-Path -LiteralPath $reportPath) -or (Get-Item -LiteralPath $reportPath).LastWriteTime -lt $runStarted) { throw 'AE did not produce a fresh report.' }
$report = Get-Content -LiteralPath $reportPath -Raw
Write-Output $report
if ($report -notmatch 'RESULT \d+ passed, 0 failed') { throw 'AE integration checks failed.' }
