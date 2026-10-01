# WEIGH — STONE 255.71: the untyped-constructor wall — STOP ACCEPTED, held local (floor red)

**Executor: a Sonnet subagent (resumed after the rate-limit stop), commits `2fb4578a9` and `332dd3f3e`, not pushed.**
Weighed by the orchestrator on 2026-09-30.

## What is built

- **The wall** (`src/check.rs`): `List`/`PersistentMap`/`PersistentVector`/`Tuple` now require their `:- [T…]` bracket,
  as `Vector`/`HashMap`/`HashSet` already did (arc 109).
- **The committed type table** `TABLE-STONE-255.71-typed-constructors.edn` (180 rows: 168 typed, 2 genuine STOP-1,
  6 `.wat.bad` legacy/positive controls, 1 stale), applied by the recorded codemod. The resumed 158 rows were verified.
  The wall's own floor found **two wrong rows**: three template sites marked STOP-1 whose type variable was in scope,
  and one `:O` that had to be the template's `~ret-ty`. The wall is doing its job.

## The floor: red, and why (orchestrator-read, `.floor/2026-09-30T23-01-26Z`, 62 failed / 6235)

The 177-site census covered the `.wat` **corpus**. The wall also reaches two populations nobody censused:

- **`wat-scripts/fixes/**`: 84 recorded codemods, 674 untyped constructor sites.** They are live wat programs (the loader
  gate type-checks them), so they now refuse to load, and every recorded migration's replay fails with them
  (`every_recorded_migration_replays` 34, `wat_scripts_fixes_load`).
- **Untyped constructors inside Rust string literals** (inline wat in `src/` unit tests and `tests/`): the `rete` tests
  (40), `rete_compile_gate` (32), and others. The agent converted 3 and listed about 40 files as a first pass, not a
  census.

## ⚠ One claim in the report is false

The agent called `probe_arc170_wrong_service_compile_error` *"pre-existing … almost certainly failed before 255.71
touched anything."* **It passed at 255.70** (`.floor/2026-09-28T22-13-36Z`: both rows `PASS`, 6225/6225). This stone
caused it. It belongs in the list of reds to cure, not to excuse. "Pre-existing" is not a disposition; the prior floor
is the measurement.

## For the builder

How the recorded codemods meet the wall (they are programs; a path exemption would be a hole in the wall). Then one stone
converts them and the Rust-literal fixtures, and fixes this stone's own reds, so the wall lands green.

## AMEND 2 weighed (2026-10-01): ACCEPTED, the wall lands green

**Commits `0374d9c78` … `a34dc3406`** (a Sonnet subagent, M1). Re-run by the orchestrator:

| row | result |
|---|---|
| release floor at `a34dc3406` | **6235 passed / 24 skipped**, exit 0 |
| the wall is intact | `git diff 2fb4578a9 HEAD -- src/check.rs` is empty (the stash-dance restored it); `git stash list` is empty |
| the wall fires | `(:wat::core::PersistentVector 1 2)` → *"untyped `:wat::core::PersistentVector` constructor call — every `PersistentVector` needs its own type bracket, whatever the head's spelling; write `(wat.type/PersistentVector :- [T…] …)`"* |
| agents' gates | census `no STOP-8` (the 84-file STOP resolved); idempotent; clippy rc 0; wall's reach: exactly the 2 named STOP-1 sites |

- **The 84 codemods:** 338 sites (the 674 was a double count: each error carries `:message` and `:reason`), typed from the
  checker's record with the wall lifted, in a sibling table `TABLE-STONE-255.71-fixes-typed-constructors.edn`.
- **The Rust-literal census:** 55 real sites in 19 files, all converted; three non-program strings left, each named.
- **The 62 reds**, grouped into 11 causes, each traced and cured with a verbatim capture. The `probe_arc170` red was
  a wrong table row (a 2-arg `Address` where the real return is 3-arg), proven against the prior green floor, and
  hand-corrected in one `.wat.bad` (the codemod cannot re-target a bracketed site; disclosed).
- **Self-reported:** its first floor (`.floor/2026-10-01T00-13-03Z`, 84 timeouts) ran **concurrently with the census**, a
  doctrine breach it named, kept, and did not count. A clean floor followed.

**The two named STOP-1 sites, for the builder:** `wat/rete/oracle/accum-pass.wat:135` (an accumulator binding whose type
depends on the rule) and `wat-scripts/probes/arc-170/probe-s3b-astsplice.wat:75` (a run-time AST-splice construction).
They are the wall's only remaining bracket-less sites.
