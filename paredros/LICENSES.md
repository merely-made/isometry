# Licensing

Paredros deliberately separates the game, reusable libraries, and game
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
once its reusable boundary was real; `crates/paredros-identity` held that
status, promoted to the wing's identity crate by the R4 extraction review
(`design_docs/2026-08-10_r4_extraction_review.md`). Mark ruled 2026-09-03
that a promoted library stays MPL-2.0 like the rest of the wing instead —
the license posture brief's platform default
(`mere/design_docs/2026-08-22_license_posture_brief.md`) leaves no boundary
exception for a library's own permissive grant, only the fork/vendor
criterion in its §4. `paredros-identity` was relicensed `MPL-2.0` the same
day; no crate in this repository currently holds a reusable-library
exception.

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

## Retained licenses

The path below is **not** covered by this repository's MPL-2.0 grant. It is
the machine-readable form of the boundary stated above, and the house header
tool (`mere/scripts/relicense_headers.py`) reads this table's first column as
its skip list, so no file under this path receives the MPL Exhibit A header.
Nothing here is third-party code: it is Mark's own work held under a
different grant by a recorded ruling.

| Path | License | Upstream / origin | Notice file |
|---|---|---|---|
| `assets` | CC BY-SA 4.0 | own work; the games-wing asset grant of 2026-07-31 | `LICENSE-CC-BY-SA-4.0`, `assets/ATTRIBUTION.md` |

`crates/paredros-identity` held a retained row here from 2026-08-10 to
2026-09-03 (MIT OR Apache-2.0, promoted library grant); it was removed when
Mark relicensed the crate MPL-2.0, per the "Reusable libraries" section
above. `assets/` tracks only Markdown today, which the tool cannot reach in
any case; the row is listed so a shader or script placed there later is
still skipped.
