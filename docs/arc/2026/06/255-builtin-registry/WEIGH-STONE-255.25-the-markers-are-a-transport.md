# WEIGH — STONE 255.25 (C-b4): the markers are a `Transport`, and the child main says `Wire` — ACCEPTED

**Executor commit `5ea41100f`**, resumed after 255.25a. Weighed by the orchestrator against disk on
2026-09-24.

## Re-measured

| row | measured | result |
|---|---|---|
| tree | `git status` | clean |
| floor runs, mapped by stamp | `.floor/2026-09-24T22-13-26Z` · `22-20-36Z` · `22-29-28Z` | 2 red (both this stone's: an inlined-wat lint on the new test, and the ledger shrinking) → **`6069 tests run: 6069 passed`** → the D2 measurement run (`6088 run: 6087 passed, 1 failed`), reverted |
| the family | `wat/spawn.wat:292` | `(:wat::core::defenum :wat::kernel::Transport :wat::enum::Pure …)`; no `defstruct` marker left |
| ⭐ the child main says Wire, end to end | `wat wat-scripts/scratch-pad/255-25-child-main-says-wire.wat` | prints `"5"`, rc=0. The same change on the old struct `Wire` was rc=2 at the self-peer purity wall |
| ledger | `LEDGER_TOTAL` | 215 → **211**: the marker helpers compare through the denotation door |

Taken from the report without re-running:

- the codemod was re-run fresh rather than splicing a stale patch; its output equals the saved patch's
  `.wat` hunks, and it is idempotent;
- its replay fixture carries 5 must-not-move controls and was shown able to go red;
- clippy 0; census `no STOP-8` with no file's rc changed; delta NEW 2 / RECOVERY 0;
- **all 57 process-child tests green, checked by name.**

## What landed

- **The markers are a closed `Pure` family,** and a variant's purity comes from its enum's declaration.
  The control, a variant of an `Impure` enum, is refused as a field. `(Address :- [.. Transport.Shared])`
  is **still impure** (`ImpureFieldInPureAggregate`); `.. Transport.Wire` is pure; the markers alone are
  pure. The old struct `Shared` alone had been refused as a field. That was the lie, and it is corrected.
- **The Rust spelling lives in one place:** `SHARED_MARKER`/`WIRE_MARKER`, compared by denotation. A
  faithful spelling is the same marker. No consumer bypasses the helpers.
- ⭐ **The generated child main spells its transport `Transport.Wire`.** The free, undeclared letter that
  255.17 traced every red back to is gone from the child.

## ⭐ The D2 measurement — the ground for C-b5 is ready

With the parked D2 patch applied on top:

- **the 57 process-child tests stay green**, where 255.17 turned all 57 red;
- all 19 `probe_arc255_17_last_type_argument` rows pass;
- the **only** red is `every_wat_scripts_file_loads`, on
  `wat-scripts/scratch-pad/255-21-kwargs-transport-lost-at-impl.wat`, which now **fails to type-check**:
  `expected (Address :- [Op Reply Transport.Wire]); got (… Transport.Shared)`.

That file is 255.21's **witness** of the kwargs-transport hole. Its own header says: *when it stops
type-checking, the hole is closed — move it to a `.wat.bad` row.* **So D2 closes the kwargs hole too.**
The patch was reverted, and D2 lands in C-b5.

## Surviving old spellings, by design

The two recorded migrations (`address-transport-arity.wat`, `unstamp-transport-wire.wat`) are survivors,
because a tool is never its own input. There is also one comment in the kwargs witness.
