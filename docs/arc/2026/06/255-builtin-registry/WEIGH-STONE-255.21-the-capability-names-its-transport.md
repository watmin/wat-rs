# WEIGH — STONE 255.21 (C-b1b), re-run: the capability names its transport — ACCEPTED (option A); the kwargs row carried to C-b5

Weighed by the orchestrator against disk on 2026-09-24. The executor stopped under the addendum's
item-3 trigger and left option A **uncommitted** in the tree. It tried to `git reset --hard` so the tree
would be quiescent; the permission classifier denied that, and the executor reported it rather than
retrying. **The orchestrator committed A** after verifying the points below.

## Re-measured

| row | measured | result |
|---|---|---|
| tree = the floored A | `git diff HEAD \| sha256sum` vs `scratchpad/s21/255.21-C-b1b-optionA.patch` | **identical** (`ae2dbcd3807dc7e2`) |
| floor on that exact tree | `.floor/2026-09-24T09-25-44Z/clean.log` | `6055 tests run: 6055 passed (9 slow), 22 skipped` |
| `src/` changes | non-comment `+`/`-` lines under `src/` | **0**: comments only |
| ⭐ the four claims | `--check tests/services/probe_arc255_21_coord_claims_either_transport.wat.bad` | rc=1, 4 errors; each `body produces (Address :- [Op Reply <actual>]); signature declares (… <claimed>)`: thread→Shared vs Wire, process→Wire vs Shared, through both `Dialable` and `TypedCapability`. **Pre-stone: rc=0.** |
| the twins | `…_coord_names_its_transport.wat` | rc=0 |
| the kwargs witness | `wat-scripts/scratch-pad/255-21-kwargs-transport-lost-at-impl.wat` | **rc=0: still open**, carried to C-b5 |

Taken from the report without re-running: clippy 0; census `no STOP-8` (215 → 215, no file's rc
changed); ledger 215; no live 2-argument `Dialable`/`TypedCapability` type position remains (the
survivors are a unit-test string and a fixture's private surface).

⭐ **Delta NEW 3 → 2, a cure, not a forged green.** `wat-scripts/probes/arc-170/probe-kwargs-peer.wat`
(the converted copy) was refused pre-stone. Its faithful spelling `wat.kernel/Peer` passed the old
substring test, but the `"wat::kernel::Peer"` text swap never relocated the head. The structural head
check relocates it. The original copy is rc=0 on both binaries.

## What landed

- **The hole witnessed on `main` since 255.21 is closed.** A thread service's `coord` returns
  `(Address :- [Op Reply :wat::kernel::Shared])` and a process service's returns `… Wire`, so claiming
  the other transport is refused. Together, 255.22 (the declared edge), 255.23 (the one method path) and
  this stone make the transport flow from the handle to the address.
- `Dialable`/`TypedCapability :- [S R T]`; defservice's edges carry the Handle's transport.
- **The kwargs macro stops reading names:**
  - `contains? nm "Peer"` (a substring test, at **9** sites) is replaced by `kwargs-type-slot-peer?`,
    which requires the head to be exactly `:wat::kernel::Peer`;
  - the text swap (`split`/`join`) is replaced by a structural `kwargs-type-slot-rehead`;
  - the generated `kwargs-check`, `GrantHandles`, `grant-worker` and `revoke-worker` declare
    `:- [T0 T2 …]`, one parameter per service field;
  - the pair type moved from the string keyword `:(Coords,GrantHandles)` to a structural `Tuple`.
- Five `.edn` goldens that pin `wat/core.wat` line numbers were recaptured (the diff is `:line` +47 only).

## Carried to C-b5 — the kwargs transport is lost one hop later, at the D2 arm

The per-field `T0` binds at the kwargs constructor (`(Kwargs :- [Shared])`), then becomes `_` at `$impl`.
Minimal case: a generic `u :- [T0] [k <- (OH :- [T0])] -> (OH :- [T0])` returns `(OH :- [String])` for
String, but **`(OH :- [_])` for `Shared`**.

The cause is `assignable`'s `transport_param_instantiates` arm (`check.rs` ~:17479): `is_type_param_letter(Var)`
is true, so a marker is admitted against a unification variable **without binding it**. That is exactly
the D2 arm. Option B (skip the arm when either side is a `Var`) closes the row, but reddens **57
process-child tests**, the same class 255.17 hit, where the generated child main types `Status` with a
free letter. **This is C-b5's ground:** delete the special cases once the child main and defservice's
emitted defns declare their transport (C-b2, C-b4).

## Brief errors, recorded

- The `check.rs` comment line numbers drifted.
- The substring test stood at 9 sites, not 8.
- Delta 3 → 2 (explained above).
- The five `.edn` goldens that pin line numbers were not anticipated.
- The kwargs binding cannot be completed inside the macro.
