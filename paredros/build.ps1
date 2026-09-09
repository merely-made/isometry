# Resolve the former-checkout junction before Cargo evaluates relative paths.
$productDirectory = Get-Item -LiteralPath $PSScriptRoot
$resolved = $productDirectory.ResolveLinkTarget($true)
$canonical = if ($resolved) { $resolved.FullName } else { $productDirectory.FullName }
& (Join-Path (Split-Path $canonical -Parent) 'scripts/wing.ps1') paredros @args
exit $LASTEXITCODE
