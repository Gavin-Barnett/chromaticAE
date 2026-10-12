$ErrorActionPreference='Stop'
$root=Split-Path -Parent $PSScriptRoot
$rustc='rustc'
if(Test-Path -LiteralPath (Join-Path $root '.toolchain\cargo\bin\rustc.exe')){
    $env:CARGO_HOME=Join-Path $root '.toolchain\cargo'
    $env:RUSTUP_HOME=Join-Path $root '.toolchain\rustup'
    $rustc=Join-Path $env:CARGO_HOME 'bin\rustc.exe'
}
New-Item -ItemType Directory -Force (Join-Path $PSScriptRoot 'target') | Out-Null
$exe=Join-Path $PSScriptRoot 'target\render-bench.exe'
& $rustc --edition 2024 -O (Join-Path $PSScriptRoot 'benches\render.rs') -o $exe
if($LASTEXITCODE -ne 0){throw 'Benchmark build failed.'}
& $exe
if($LASTEXITCODE -ne 0){throw 'Benchmark failed.'}
