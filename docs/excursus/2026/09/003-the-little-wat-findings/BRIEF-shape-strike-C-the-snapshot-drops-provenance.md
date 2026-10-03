# BRIEF — shape strike C: an error's value snapshot drops `provenance`

Excursus 003. This implements `AUDIT-the-shape-of-an-error.md` F5 and its § RULING 2026-09-27
item 4 ("`provenance` goes"). It builds on `021fa9a0d`.

**Read `wat-rs/CLAUDE.md` in full first.** Every citation below is a claim. Verify each one before
relying on it, and STOP on a contradiction. Where the call is small and the intent is clear, make it
and name it.

## Why

`:wat::runtime::ValueSnapshot` is the `got` value carried by `TypeMismatch`, `NotCallable`,
`BadCondition`, `NoMatchingClause.called-args` and `PostconditionFailed.returned-value`. It carries
`{type-name rendered provenance}`.

The audit measured `provenance` as unknown in **503 of 506** construction sites:
- `ValueSnapshot::of` always yields `Unknown`;
- `of_tracked` has 3 callers;
- arc 233 retired `Value::Tracked`, which fed it.

In goldens, "unknown" is written two ways, `:provenance nil` (22) and `Option.None` (19), and it is
real in only 5. The builder ruled it goes, leaning towards removal: *"we'll add stuff back in later
if we choose to."*

## Scope: the snapshot field, and a measurement of what is left behind

`Provenance` the type is wider than the snapshot. Measured at `a7749e5a1`, it appears in 18 Rust
files: environment bindings (`src/value/environment.rs:~203`), intrinsics (`keyword`, `edn`, `ast`,
`holon/atom`), the `wat_intrinsic` macro, render, and others. The ruling covers **the error
snapshot's field**. So:

1. **Remove `provenance` from `ValueSnapshot`**, on both sides:
   - the Rust struct (`src/value/observe.rs:~94`);
   - its writer (`value_snapshot_to_edn`, `src/edn/error.rs:~126`);
   - `to_record`'s builder (`src/value/runtime_records.rs:~95`);
   - the wat declaration (`:wat::runtime::ValueSnapshot`, `wat/runtime-errors.wat`).
   - `ValueSnapshot::of_tracked` collapses into `of`, if nothing else needs it.
2. **Measure, do not delete, everything else.** After (1), find which remaining uses of `Provenance`
   have a **consumer**: something that reads it and changes an outcome or an output. Write the census
   in the report, one row per site: kept (with its consumer), or dead (no consumer).
   - If the declared `:wat::kernel::Provenance` enum (`wat/runtime-errors.wat` or
     `wat/kernel/diagnostics.wat`) loses its last holder, retire the declaration.
   - **Do not remove the Rust tracking machinery** (`TrackedValue`, environment provenance, intrinsic
     provenance), even if the census shows it dead. That is a separate decision for the builder;
     report it.

## Gate (mutation-proven in RELEASE)

- **GC1, no snapshot carries provenance.** A lint over every tracked `.edn` golden: no
  `#wat.runtime/ValueSnapshot` map carries a `:provenance` key. It must first be RED on today's
  goldens. Report the file count.
  - Mutation: re-add the key in the writer. RED.
- The existing gates (3a G1/G2/G3, T's typed decode) stay green. Since T, decode refuses an
  undeclared key, so a stale writer can't hide. Say whether GC1 is still needed beside that, and
  keep it either way.

## Goldens

Recapture with `UPDATE_EDN=1` and **read every diff**. The only allowed change is the `:provenance …`
line leaving `ValueSnapshot` maps. Report anything else.

## Scope fence

- **IN:** items 1 and 2 (the measurement only) and GC1.
- **OUT:**
  - deleting `Provenance` tracking beyond the snapshot;
  - D (`Frame`), E (`EvalError`), F (the domain `Fault`s);
  - the open builder calls (`char`, `LoadFetchError`'s tag rename, the owner-only admin panics);
  - the stdlib-freeze excursus.

## Discipline

- **`cargo nextest run --release` ONLY, never `cargo test`.** One cargo process at a time: check
  with `pgrep -x cargo` and `pgrep -x cargo-nextest`, never `pgrep -f`.
- **Never use a git worktree with a shared `CARGO_TARGET_DIR`.**
- **Never edit files under `tests/`, `wat/` or `wat-scripts/` while any run is in flight.**
- **Never hand back while a build, recapture or floor runs.** Wait in the foreground with
  `until grep -qE '^ *Summary' <log>; do sleep 30; done`, repeating past the tool's 10-minute cap.
- Iterate with targeted runs. Run the full floor at the commit.
- The floor:
  - Run `scripts/floor.sh`: 0 failed, 0 timed out.
  - On any red: surface it before any re-run, verbatim, with the arm named.
  - **Never commit a red floor.**
- Stage by name. Never use `git add -A`. No `cargo fmt`.
- Commit prefix `EXCURSUS(003):`, ending with
  `Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>`. Push to
  `origin/reason/little-wat-findings`.

## Report

- the removal sites;
- the `Provenance` census (kept with its consumer, or dead);
- the declaration's fate;
- GC1's anchor count and mutation RED;
- any golden that changed beyond the allowed change;
- the floor `Summary` line, verbatim;
- the SHA.
