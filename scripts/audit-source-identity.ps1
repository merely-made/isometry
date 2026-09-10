#Requires -Version 7.0
[CmdletBinding()]
param(
    [ValidateSet('isometry', 'mesocosm', 'paredros')]
    [string[]] $Scope,

    # Use this with one explicit workspace-root manifest instead of -Scope. It is
    # useful for a leaf workspace or a worktree whose layout differs here.
    [string] $ManifestPath,

    # A metadata file is preferred for reproducible receipts.  With no file,
    # cargo metadata is run read-only for each selected manifest.
    [string[]] $MetadataPath,

    # Make this an enforcing gate when desired. Cross-product differences
    # remain informational unless -FailOnMismatch is supplied.
    [switch] $FailOnDuplicate,
    [switch] $FailOnMismatch,

    # Extra Cargo configuration files for paired-development closures. They
    # complement Cargo's normal per-workspace config discovery.
    [string[]] $CargoConfig,

    # Include optional dependencies when resolving a fresh metadata graph.
    [switch] $AllFeatures,

    # Metadata is locked by default so an audit cannot refresh resolution.
    [switch] $Unlocked
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$family = @(
    'netrender',
    'netrender_text',
    'paint_list_api',
    'netrender_device',
    'layout-dom-api',
    'genet-scripted-dom',
    'paint_list_render',
    'cambium',
    'cambium-rootstock',
    'cambium-genet-winit-host',
    'sprigging',
    'sceno',
    'scenomise',
    'scenotime',
    'muniment',
    'conatus',
    'modulus',
    'nisus'
)
$scriptRoot = Split-Path -Parent $PSScriptRoot
$manifestByScope = [ordered]@{
    isometry = Join-Path $scriptRoot 'Cargo.toml'
    mesocosm = Join-Path $scriptRoot 'mesocosm/Cargo.toml'
    paredros = Join-Path $scriptRoot 'paredros/Cargo.toml'
}

if ($ManifestPath) {
    if ($Scope) {
        throw '-ManifestPath cannot be combined with an explicit -Scope.'
    }
    $manifests = [ordered]@{ explicit = (Resolve-Path -LiteralPath $ManifestPath).Path }
} else {
    if (-not $Scope) { $Scope = @('isometry', 'mesocosm', 'paredros') }
    $manifests = [ordered]@{}
    foreach ($name in $Scope) {
        $manifests[$name] = (Resolve-Path -LiteralPath $manifestByScope[$name]).Path
    }
}

if ($MetadataPath -and $MetadataPath.Count -ne $manifests.Count) {
    throw "Provide one -MetadataPath per selected consumer graph ($($manifests.Count) expected)."
}

function Read-Metadata {
    param([string] $Manifest, [string] $JsonPath)
    if ($JsonPath) {
        # Cargo feature maps can contain case-distinct keys (for example USB
        # and usb). Preserve JSON key identity instead of building PSObjects.
        return (Get-Content -Raw -LiteralPath (Resolve-Path -LiteralPath $JsonPath).Path | ConvertFrom-Json -AsHashtable)
    }

    # Cargo metadata only resolves and describes the graph. It does not
    # compile or modify the selected workspace source; resolution may fetch
    # missing registry or git sources.
    $cargoArguments = @()
    if ($CargoConfig) {
        foreach ($config in $CargoConfig) {
            $cargoArguments += @('--config', (Resolve-Path -LiteralPath $config).Path)
        }
    }
    $cargoArguments += @('metadata', '--format-version', '1')
    if (-not $Unlocked) { $cargoArguments += '--locked' }
    if ($AllFeatures) { $cargoArguments += '--all-features' }
    $cargoArguments += @('--manifest-path', $Manifest)
    $manifestDirectory = Split-Path -Parent $Manifest
    Push-Location -LiteralPath $manifestDirectory
    try {
        $json = & cargo @cargoArguments | Out-String
        $cargoExit = $LASTEXITCODE
    } finally {
        Pop-Location
    }
    if ($cargoExit -ne 0) {
        throw "cargo metadata failed for $Manifest (exit $cargoExit)."
    }
    return ($json | ConvertFrom-Json -AsHashtable)
}

function Assert-MetadataIntegrity {
    param($Metadata, [string] $Manifest)

    $expectedRoot = (Resolve-Path -LiteralPath (Split-Path -Parent $Manifest)).Path
    if (-not $Metadata.Contains('workspace_root') -or -not $Metadata.workspace_root) {
        throw "cargo metadata for $Manifest has no workspace_root. Regenerate the saved metadata from this workspace."
    }
    try {
        $actualRoot = (Resolve-Path -LiteralPath $Metadata.workspace_root).Path
    } catch {
        throw "cargo metadata workspace_root '$($Metadata.workspace_root)' cannot be resolved for $Manifest."
    }
    $pathComparison = if ([IO.Path]::DirectorySeparatorChar -eq '\') {
        [StringComparison]::OrdinalIgnoreCase
    } else {
        [StringComparison]::Ordinal
    }
    if (-not [string]::Equals($actualRoot, $expectedRoot, $pathComparison)) {
        throw "cargo metadata workspace_root '$actualRoot' does not match selected manifest root '$expectedRoot'."
    }

    if (-not $Metadata.Contains('resolve') -or $null -eq $Metadata.resolve) {
        throw "cargo metadata for $Manifest has no resolved graph. Regenerate it without --no-deps."
    }
    if (-not $Metadata.resolve.Contains('nodes') -or $null -eq $Metadata.resolve.nodes) {
        throw "cargo metadata for $Manifest has no resolve.nodes graph."
    }

    if (@($Metadata.workspace_members).Count -eq 0) {
        throw "cargo metadata for $Manifest has no workspace members."
    }
    $packageIds = [Collections.Generic.HashSet[string]]::new()
    foreach ($package in @($Metadata.packages)) {
        if (-not $package.id) { throw "cargo metadata for $Manifest contains a package without an id." }
        [void]$packageIds.Add([string]$package.id)
    }
    $nodeIds = [Collections.Generic.HashSet[string]]::new()
    foreach ($node in @($Metadata.resolve.nodes)) {
        if (-not $node.id -or -not $packageIds.Contains([string]$node.id)) {
            throw "cargo metadata for $Manifest has a resolve node without a matching package id."
        }
        [void]$nodeIds.Add([string]$node.id)
    }
    foreach ($member in @($Metadata.workspace_members)) {
        if (-not $nodeIds.Contains([string]$member)) {
            throw "cargo metadata for $Manifest has no resolve node for workspace member '$member'."
        }
    }
    foreach ($node in @($Metadata.resolve.nodes)) {
        foreach ($dependency in @($node.deps)) {
            if (-not $dependency.pkg -or -not $packageIds.Contains([string]$dependency.pkg) -or -not $nodeIds.Contains([string]$dependency.pkg)) {
                throw "cargo metadata for $Manifest has a dependency edge to an unknown package or resolve node."
            }
        }
    }
}

function Get-ReachablePackageIds {
    param($Metadata)
    $byId = @{}
    foreach ($package in @($Metadata.packages)) { $byId[$package.id] = $package }

    $edges = @{}
    foreach ($node in @($Metadata.resolve.nodes)) {
        $edges[$node.id] = @($node.deps | ForEach-Object { $_.pkg })
    }

    $seen = [Collections.Generic.HashSet[string]]::new()
    $pending = [Collections.Generic.Queue[string]]::new()
    foreach ($id in @($Metadata.workspace_members)) { $pending.Enqueue([string]$id) }
    while ($pending.Count -gt 0) {
        $id = $pending.Dequeue()
        if (-not $seen.Add($id)) { continue }
        foreach ($next in @($edges[$id])) {
            if ($next) { $pending.Enqueue([string]$next) }
        }
    }
    return @($seen)
}

$metadataByConsumer = [ordered]@{}
$familyNames = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
foreach ($name in $family) { [void]$familyNames.Add($name) }
$index = 0
foreach ($entry in $manifests.GetEnumerator()) {
    $jsonPath = if ($MetadataPath) { $MetadataPath[$index] } else { $null }
    $metadata = Read-Metadata -Manifest $entry.Value -JsonPath $jsonPath
    Assert-MetadataIntegrity -Metadata $metadata -Manifest $entry.Value
    $metadataByConsumer[$entry.Key] = $metadata
    # Include every resolved package from the three platform owners, even when
    # it is not one of the critical names above. The fixed names also cover
    # paired-development path overrides and registry paint/DOM patches.
    foreach ($package in @($metadata.packages)) {
        if ($package.source -match '^git\+https://github\.com/(merely-made|mark-ik)/(mere|genet|netrender)\.git(?:[?#]|$)') {
            [void]$familyNames.Add([string]$package.name)
        }
    }
    $index++
}
$family = @($familyNames | Sort-Object)
$reports = [ordered]@{}
$mismatches = 0
foreach ($entry in $manifests.GetEnumerator()) {
    $metadata = $metadataByConsumer[$entry.Key]
    $packageById = @{}
    foreach ($package in @($metadata.packages)) { $packageById[$package.id] = $package }
    $reachable = Get-ReachablePackageIds $metadata
    $rows = @()
    foreach ($name in $family) {
        $matches = @($reachable | ForEach-Object { $packageById[$_] } | Where-Object { $_.name -eq $name } |
            Sort-Object @{Expression='source'; Descending=$false}, version, id)
        $identities = @($matches | ForEach-Object {
            [pscustomobject]@{
                name = $_.name
                version = $_.version
                source = if ($_.source) { $_.source } else { 'path' }
                id = $_.id
            }
        })
        $rows += [pscustomobject]@{ name = $name; packages = $identities; duplicate = ($identities.Count -gt 1) }
    }
    $reports[$entry.Key] = [pscustomobject]@{ manifest = $entry.Value; rows = $rows }
}

Write-Output 'SOURCE IDENTITY AUDIT'
Write-Output "Families: $($family -join ', ')"
foreach ($report in $reports.GetEnumerator()) {
    Write-Output "`n[$($report.Key)] $($report.Value.manifest)"
    foreach ($row in $report.Value.rows) {
        if ($row.packages.Count -eq 0) {
            Write-Output "  $($row.name): ABSENT"
            continue
        }
        $mark = if ($row.duplicate) { ' DUPLICATE' } else { '' }
        Write-Output "  $($row.name): $($row.packages.Count) identity(ies)$mark"
        foreach ($package in $row.packages) {
            Write-Output "    $($package.version) | $($package.source) | $($package.id)"
        }
    }
}

if ($reports.Count -gt 1) {
    Write-Output "`nCROSS-PRODUCT DIFFERENCES (includes legitimate absences)"
    foreach ($name in $family) {
        $shapeRows = @($reports.GetEnumerator() | ForEach-Object {
            $items = @($_.Value.rows | Where-Object name -eq $name | Select-Object -ExpandProperty packages)
            $key = (($items | ForEach-Object { "$($_.version)|$($_.source)|$($_.id)" }) -join ';')
            [pscustomobject]@{ product = $_.Key; key = $key }
        })
        if (@($shapeRows | ForEach-Object { $_.key } | Select-Object -Unique).Count -gt 1) {
            $shapes = $shapeRows | ForEach-Object { "$($_.product)=$($_.key)" }
            Write-Output "  ${name}: $($shapes -join ' ; ')"
            $present = @($shapeRows | Where-Object { $_.key })
            if ($present.Count -gt 1 -and @($present | ForEach-Object { $_.key } | Select-Object -Unique).Count -gt 1) {
                $mismatches++
            }
        }
    }
}

$duplicates = @($reports.Values | ForEach-Object { $_.rows } | Where-Object duplicate)
Write-Output "`nSUMMARY: $($reports.Count) consumers; $($duplicates.Count) duplicate package names; $mismatches cross-product identity mismatches."
if ($FailOnDuplicate -and $duplicates.Count -gt 0) { exit 1 }
if ($FailOnMismatch -and $mismatches -gt 0) { exit 1 }
exit 0
