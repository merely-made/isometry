# Parse the positional product ourselves so Cargo flags such as -p cannot
# bind to a PowerShell parameter abbreviation.
$Product = if ($args.Count -gt 0) { $args[0] } else { '' }
if ($Product -notin @('isometry', 'mesocosm', 'paredros')) {
    throw 'Usage: wing.ps1 <isometry|mesocosm|paredros> [cargo arguments]'
}
$CargoArguments = @($args | Select-Object -Skip 1)

# Enter the selected workspace so Cargo discovers that product's config,
# assets and lockfile. Tabletop-only local patches must not leak into children.
$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path $PSScriptRoot -Parent
$workspacePath = if ($Product -eq 'isometry') { $repoRoot } else { Join-Path $repoRoot $Product }
$localConfig = Join-Path $repoRoot '.cargo/tabletop-local.toml'
$cargoPrefix = @()
if ($Product -eq 'isometry' -and (Test-Path -LiteralPath $localConfig)) {
    $cargoPrefix = @('--config', $localConfig)
}
if (-not $CargoArguments) { $CargoArguments = @('check') }
Push-Location -LiteralPath $workspacePath
try {
    & cargo @cargoPrefix @CargoArguments
    $result = $LASTEXITCODE
} finally {
    Pop-Location
}
exit $result
