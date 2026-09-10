#Requires -Version 7.0
# Run with pwsh -File scripts/tests/audit-source-identity.ps1. No Cargo fetches.
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$audit = Join-Path $repo 'scripts/audit-source-identity.ps1'
$fixture = [IO.Path]::GetTempFileName()
$secondFixture = [IO.Path]::GetTempFileName()
$valid = @{
    workspace_root = $repo
    workspace_members = @('root')
    packages = @(@{ id = 'root'; name = 'fixture'; version = '0.1.0'; source = $null })
    resolve = @{ nodes = @(@{ id = 'root'; deps = @() }) }
}
# Real all-features Cargo metadata can carry both names in one feature map.
$features = [Collections.Generic.Dictionary[string, object]]::new([StringComparer]::Ordinal)
$features.Add('USB', @())
$features.Add('usb', @())
$valid.packages[0].features = $features

function Assert-Audit {
    param($Metadata, [bool] $Accept, [string] $Name, $SecondMetadata = $null)
    $Metadata | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $fixture
    $arguments = @{ Scope = @('isometry'); MetadataPath = @($fixture) }
    if ($null -ne $SecondMetadata) {
        $SecondMetadata | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $secondFixture
        $arguments.Scope += 'mesocosm'
        $arguments.MetadataPath += $secondFixture
    }
    $global:LASTEXITCODE = 0
    try {
        & $audit @arguments -FailOnDuplicate -FailOnMismatch *> $null
        $accepted = $LASTEXITCODE -eq 0
    } catch {
        $accepted = $false
    }
    if ($accepted -ne $Accept) { throw "Unexpected audit result: $Name (accepted=$accepted)" }
    Write-Output "PASS: $Name"
}

try {
    Assert-Audit $valid $true 'case-distinct feature names'
    $renderer = $valid | ConvertTo-Json -Depth 20 | ConvertFrom-Json -AsHashtable
    $renderer.packages += @(
        @{ id = 'renderer-a'; name = 'netrender'; version = '0.1.0'; source = 'git+https://example.invalid/netrender?rev=a' },
        @{ id = 'renderer-b'; name = 'netrender'; version = '0.1.0'; source = 'git+https://example.invalid/netrender?rev=b' }
    )
    $renderer.resolve.nodes += @(
        @{ id = 'renderer-a'; deps = @() },
        @{ id = 'renderer-b'; deps = @() }
    )
    $renderer.resolve.nodes[0].deps = @(@{ pkg = 'renderer-a' })
    Assert-Audit $renderer $true 'unreachable duplicate ignored'
    $renderer.resolve.nodes[0].deps += @{ pkg = 'renderer-b' }
    Assert-Audit $renderer $false 'reachable duplicate refused'
    $unlisted = $renderer | ConvertTo-Json -Depth 20 | ConvertFrom-Json -AsHashtable
    foreach ($package in $unlisted.packages | Where-Object name -eq 'netrender') {
        $package.name = 'unlisted-platform-fixture'
        $package.source = "git+https://github.com/merely-made/mere.git?rev=$($package.id)"
    }
    Assert-Audit $unlisted $false 'unlisted platform package duplicate refused'
    $renderer.resolve.nodes[0].deps = @(@{ pkg = 'renderer-a' })
    $other = $renderer | ConvertTo-Json -Depth 20 | ConvertFrom-Json -AsHashtable
    $other.workspace_root = Join-Path $repo 'mesocosm'
    $other.resolve.nodes[0].deps = @(@{ pkg = 'renderer-b' })
    Assert-Audit $renderer $false 'cross-product identity mismatch refused' $other
    $other.resolve.nodes[0].deps = @()
    Assert-Audit $renderer $true 'legitimate cross-product absence allowed' $other
    foreach ($case in @('wrong-root', 'null-resolve', 'missing-member', 'unknown-package', 'unknown-edge')) {
        $metadata = $valid | ConvertTo-Json -Depth 20 | ConvertFrom-Json -AsHashtable
        switch ($case) {
            'wrong-root' { $metadata.workspace_root = Split-Path -Parent $repo }
            'null-resolve' { $metadata.resolve = $null }
            'missing-member' { $metadata.workspace_members = @('missing') }
            'unknown-package' { $metadata.resolve.nodes[0].id = 'unknown' }
            'unknown-edge' { $metadata.resolve.nodes[0].deps = @(@{ pkg = 'unknown' }) }
        }
        Assert-Audit $metadata $false $case
    }
} finally {
    Remove-Item -LiteralPath $fixture, $secondFixture -Force
}
