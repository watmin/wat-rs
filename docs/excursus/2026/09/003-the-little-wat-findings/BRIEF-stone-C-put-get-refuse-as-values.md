# BRIEF — STONE C (redrawn): `Lru/put` returns a Result, `Lru/get` misses

Read `DESIGN-stone-C-put-get-refuse-as-values.md` first.

## The work

1. `src/rust_deps/cache.rs`: `put` returns `Result<Option<(Value, Value)>, RawFault>`, `Err` on an
   unhashable key with `diagnostic = ":wat::cache::Lru/put"`; `get` returns `None` on an unhashable
   key. Remove both `panic!`s. Rewrite the module doc's failure-surface section.
2. `wat/cache.wat`: `:wat::cache::Lru/put`'s signature returns the `Result`; `HolographicLru/put`
   propagates it; service handlers use `Result/expect` with the verb name in the message if and
   only if the surface forces it (stone A's disposition).
3. Move the 31 `put` call sites **with a recorded wat-fix codemod** (`CLAUDE.md` — never hand-edit
   a multi-site `.wat` change). Model: `wat-scripts/fixes/wrap-cache-new-in-result-expect.wat` and
   its replay fixture `wat-scripts/fixes/replay/wrap-cache-new-in-result-expect/`.
4. Record what shipped in `docs/arc/2026/04/109-kill-std/NOTE-the-cache-lru-panics-…md`.

## Rooms

- `src/rust_deps/cache.rs:40-75` (module doc) and `:140-172` (`put`/`get`); `Lru::new` above them
  (`:~120`) is stone A's shipped `Result<Self, RawFault>` — copy it.
- `wat/cache.wat:127-144` (`Lru/put`), `HolographicLru/put`, and the two services' handlers.
- `src/collection/eval.rs:203-207` — HashMap `contains-key?`'s miss, the precedent for `get`.
- The stone-A codemod and replay fixture above.

## STOP triggers

1. A service handler that cannot express the `Err` AND cannot `Result/expect` it — report.
2. The F-083 row can no longer pin its defect after the fixture moves — report; do not delete it.
3. A `put` caller the DESIGN's table lacks — derive the count yourself and report the delta.
4. Any gate reddens that you did not add — capture whole, name the arm. ⛔ Do not re-run first.

## Prove it — a probe, four cases

`put` direct · `put` via a generic `K` · `get` direct · `get` via a generic `K`, each keyed on an
`Lru` handle. `put` → an `Err` value naming `:wat::cache::Lru/put`; `get` → a miss. No
`panicked at`, no `RUST_BACKTRACE`. **Mutation:** restore each `panic!` in turn (keeping the new
signatures) — its cases go RED. Restore.

## Mechanics — ⛔ read

- The Bash tool caps at 600s. Floor: `nohup scripts/floor.sh > <file> 2>&1 &`, then repeated
  foreground `until grep -qE '^ *Summary' <that file>; do sleep 30; done` blocks. Do not end your
  turn while it runs. Never `pgrep -f 'cargo …'`. Read `Summary` lines, never piped exit codes.
- `cargo fmt` reformats the whole workspace — `rustfmt <file>` or neither.
- `git add` BEFORE running `git ls-files`-based gates.
- A new file under `tests/` or `wat-scripts/fixes/` answers to: `no_loose_string_assert`,
  `no_inlined_edn`, `no_inlined_wat_in_tests`, `every_tracked_wat_parses`,
  `every_wat_bad_fixture_actually_fails`, `every_recorded_migration_replays` (fixture-or-rune +
  `;; SCOPE:`), and the sharded `every_wat_scripts_file_loads…` gate.
