# Cleromancy generator selection

## Decision

Use Cleromancy only to choose one already-loaded Isometry generator before
the normal host-side preview call. This is a local GM action, entered as:

`>choose <seed> <domain> <prompt>`

The three values are public to the local receipt. The host constructs a
uniform field from the loaded `GeneratorChoice` declarations, derives a
Cleromancy reading, then routes its selected choice through the existing
`GenerationRequest::Generate` path.

## Request and result

`GeneratorSelectionRequest` carries the seed, domain, and prompt. The
host-only `GeneratorSelection` returns the selected choice index together
with its `ContextSnapshot`, `Field`, and `Reading`. Those values replay the
exact choice without running an Isometry pack.

The candidate declaration includes the generator id, name, default arguments,
and lock presets. Changing a loaded generator declaration therefore changes
the receipt field digest rather than silently replaying against a different
choice set.

## Ownership

- Cleromancy owns qualification, derived selection, and the sealed receipt.
- Isometry owns its explicit command, loaded generator declarations, preview
  execution, host entropy tape, and campaign commit.
- The receipt is host-local. It is not a `GameEvent`, does not enter a campaign
  checkpoint, and is not sent to joined players.
- The selected generator still runs through Isometry's ordinary sandbox and
  preview/commit gates. Cleromancy neither runs a pack nor mutates campaign
  state.

## Acceptance

- The same request and declared choices produce a replayable Cleromancy
  reading.
- Its selected id builds an ordinary `GeneratorRequest` and produces a real
  Isometry preview record through `GeneratorCatalog::generate`.
- Empty choices or an empty prompt fail before a preview is queued.
- A joined player cannot request the action because the existing host authoring
  gate rejects it.

## Stop rule

This does not add a game-facing Cleromancy intent, shared challenge protocol,
campaign persistence for readings, player-visible oracle mechanics, automatic
generation, or sync. Publishing the paired source revisions is a separate
release/commit step.
