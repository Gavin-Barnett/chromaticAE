param()
$ErrorActionPreference='Stop'
$root=Split-Path -Parent $PSScriptRoot
# Use a workspace-local toolchain if present; otherwise use the user's Rust install.
$localCargo=Join-Path $root '.toolchain\cargo\bin\cargo.exe'
if(Test-Path -LiteralPath $localCargo){
    $env:CARGO_HOME=Join-Path $root '.toolchain\cargo'
    $env:RUSTUP_HOME=Join-Path $root '.toolchain\rustup'
    $env:PATH="$env:CARGO_HOME\bin;$env:PATH"
    $cargo=$localCargo
}else{$cargo='cargo'}
& $cargo test --locked --manifest-path (Join-Path $PSScriptRoot 'Cargo.toml')
if($LASTEXITCODE -ne 0){throw 'Kernel tests failed.'}
& $cargo build --locked --release --manifest-path (Join-Path $PSScriptRoot 'Cargo.toml')
if($LASTEXITCODE -ne 0){throw 'Native build failed.'}
$dest=Join-Path $root 'plugin\Chromatic AE.aex'
New-Item -ItemType Directory -Path (Split-Path -Parent $dest) -Force | Out-Null
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'target\release\chromatic_ae.dll') -Destination $dest -Force
Get-FileHash -LiteralPath $dest -Algorithm SHA256
Write-Host 'Built plugin\Chromatic AE.aex. Close AE and run Install Chromatic AE.cmd to install.'
