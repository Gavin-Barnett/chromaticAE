param([string]$AfterFX='C:\Program Files\Adobe\Adobe After Effects 2026\Support Files\AfterFX.exe')
$ErrorActionPreference='Stop'
$root=Split-Path -Parent $PSScriptRoot
$report=Join-Path $root 'test-output\native-bounds-results.txt'
$regression=Join-Path $root 'test-output\native-results.txt'
New-Item -ItemType Directory -Force (Split-Path $report -Parent) | Out-Null
$expectedMajor=(Get-Item -LiteralPath $AfterFX).VersionInfo.FileMajorPart
$started=Get-Date
$script=Join-Path $PSScriptRoot 'native-suite.jsx'
# AE's script parser treats unnecessary quotes as part of the filename.
$scriptArgument=if($script -match '\s'){ '"'+$script+'"' }else{$script}
$proc=Start-Process -FilePath $AfterFX -ArgumentList @('-noui','-m','-r',$scriptArgument) -WindowStyle Hidden -PassThru
Write-Host "Running native integration tests in AE process $($proc.Id)."
$deadline=(Get-Date).AddMinutes(3)
while((Get-Date) -lt $deadline){
    if((Test-Path -LiteralPath $report) -and (Get-Item -LiteralPath $report).LastWriteTime -ge $started){
        $text=Get-Content -LiteralPath $report -Raw
        if($text -match 'RESULT \d+ passed, \d+ failed'){
            if($text -notmatch ('After Effects '+$expectedMajor+'\.')){throw 'Report came from a different AE version. Run suites sequentially.'}
            Write-Output $text
            $centre=Get-Content -LiteralPath (Join-Path $root 'test-output\native-centre-results.txt') -Raw
            Write-Output $centre
            $original=Get-Content -LiteralPath $regression -Raw
            Write-Output $original
            if($centre -notmatch 'RESULT 15 passed, 0 failed' -or $text -notmatch 'RESULT (18|21) passed, 0 failed' -or $original -notmatch 'RESULT 27 passed, 0 failed'){throw 'AE integration failed. See test-output reports.'}
            exit 0
        }
    }
    if($proc.HasExited){throw ('AE exited before producing a fresh suite report. Exit code: '+$proc.ExitCode)}
    Start-Sleep -Milliseconds 500
}
throw 'AE test timed out. Inspect the test process and startup dialogs; an existing user project was not closed.'
