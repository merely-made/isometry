# Licensing

Mesocosm deliberately separates the game, reusable libraries, and game
assets. A license applies according to the material's owning boundary, not
merely because its text is present in the repository.

## Game and repository

Game code, application and host crates, tests, examples, and repository
documentation are licensed under the Mozilla Public License 2.0
(`MPL-2.0`). See `LICENSE-MPL-2.0`.

The `0.0.1` name-reservation package was published under `MIT OR
Apache-2.0`; those existing grants remain in force. The MPL game-code grant
begins with repository version `0.0.2`.

## Reusable libraries

**Retired 2026-09-03.** From 2026-07-31 to 2026-09-03 a separately
identified reusable library crate could be licensed `MIT OR Apache-2.0`
once its reusable boundary was real. Mark ruled 2026-09-03 that a promoted
library stays MPL-2.0 like the rest of the wing instead — the license
posture brief's platform default
(`mere/design_docs/2026-08-22_license_posture_brief.md`) leaves no boundary
exception for a library's own permissive grant, only the fork/vendor
criterion in its §4. No crate in this repository ever held the retired
status; `mesocosm-phenotype` was deliberately **not** one of these even
under the old clause, since pack loaders, validators, definitions and
game-specific schemas are game code (processdef plan §5), and it remains
`MPL-2.0` in its own manifest under the current rule too.

## Assets

Original game assets under `assets/` are licensed under Creative Commons
Attribution-ShareAlike 4.0 International (`CC-BY-SA-4.0`) unless an adjacent
notice says otherwise. See `LICENSE-CC-BY-SA-4.0` and
`assets/ATTRIBUTION.md`.

Imported or third-party assets retain their own licenses. Add their creator,
source, license, and modification history to `assets/ATTRIBUTION.md`; never
silently relicense them as project originals. Content Mark expects later —
body templates and the data-type core-plus-frontier (a defined core of types
plus an extensible frontier where new types are made and core ones combined)
— follows this same asset grant.
