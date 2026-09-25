# BRIEF — ENVELOPE STEP 1 (D1): one location shape — `:wat::kernel::Location` retires into `:wat::core::Span`

**Drawn 2026-09-25.** First of four steps of `DESIGN-the-error-envelope-and-its-frames.md` (RULED
2026-09-24, D1-D4 accepted). Read that design's D1 section first.

## The work

Three records mean "a location" today. This step removes one of them:

| record | fields | end |
|---|---|---|
| `:wat::core::Span` (`wat/core.wat:2233`) — KEEP | `file line col end` | `(Option :- [Pos])` |
| `:wat::kernel::Location` (`wat/core.wat:2161`) — RETIRE | `file line col` | none |

Every use of `:wat::kernel::Location` becomes `:wat::core::Span`. **Retire it outright — no alias.** One
way to say "where" is the point of D1. A Rust-originated location becomes a `Span` with `end` `None`; a
wat-originated one carries `Some`.

(`:wat::kernel::Frame` is the third shape; it changes in step 2, not here — leave it.)

## Measured scope at `2c4c06a05` — re-derive it

- `.wat` (7 lines, 5 files): `wat/core.wat:2161` (the declaration), `:2185` (the `:wat::core::Error`
  surface's `location`), `:2201` (`Fault.location`); `wat/grep.wat:333-334` (`Location/line`,
  `Location/col` accessors → `Span/line`, `Span/col`); `wat/kernel/diagnostics.wat:157`
  (`(Option :- [Location])`); `wat/rete/compile.wat:266`.
- Rust (~10): `src/types.rs:2286` (`wat_record_from!(… ":wat::kernel::Location")`), `src/runtime.rs:~11979`
  and `~11992` (the builder and its names), `src/host/test_runner.rs:908, 911, 1045`, plus whatever
  `(:wat::kernel::here)` returns — find it.
- Tests/goldens: 10 files name `wat.kernel/Location` or `kernel::Location`. Recapture goldens with
  `UPDATE_EDN=1` and check each new one is a `#wat.core/Span`.

## How

- The `.wat` changes are a structural rename across several files: per `CLAUDE.md`, a **recorded wat-fix
  codemod** under `wat-scripts/fixes/`, dry-run on a copy and diffed, with a replay fixture (stone A's
  `wrap-cache-new-in-result-expect` is a model). `wat/` is the stdlib, `include_str!`'d — it needs a
  rebuild to take effect.
- Where Rust BUILDS a Location value from a `Span` it has in hand, build the `Span` directly — keep the
  `end` if one exists, never drop it to match the old shape.

## Prove it

- `grep` for `kernel::Location` / `wat.kernel/Location` over `wat/ src/ crates/ tests/ wat-tests/` → 0
  (arc/excursus docs excepted — history).
- `(:wat::kernel::here)` returns a `:wat::core::Span`.
- A `Fault/of` error and an assertion failure show `:location #wat.core/Span {… :end …}`.
- Floor 0 failed; clippy clean. A rename — the compiler and the load gates are the gate; say so rather
  than stage a fake mutation.

## STOP triggers

1. A consumer relies on `Location` having NO `end` field (e.g. positional destructuring, an EDN reader
   expecting exactly three keys) — report it.
2. The wire decoder for a `Span` cannot read what used to be a `Location` from an older peer — report;
   do not add a compatibility path unasked.
3. Any gate reddens that you did not add (other than the goldens you were told to recapture) — capture
   whole, name the arm. ⛔ Do not re-run first.

## Mechanics — ⛔ read

- **`cargo nextest run --release`, never `cargo test`** — focused runs too.
- **One cargo process at a time.** Never start a second build or test run while one is going; if you
  start one by mistake, let it FINISH rather than killing it just before the floor — stone R's floor
  went red with 83 timeouts from exactly that.
- Floor: `nohup scripts/floor.sh > <file> 2>&1 &`, then repeated foreground
  `until grep -qE '^ *Summary' <that file>; do sleep 30; done` blocks (each under the 600s cap). Do not
  end your turn while it runs. Never `pgrep -f 'cargo …'`. Read `Summary` lines.
- No `cargo fmt` / `rustfmt`. Stage explicit paths, never `git add -A`.
- Test lints: EDN string literals trip `no_inlined_edn` (use `.edn` goldens); `contains`/`ends_with` in
  an assert trips `no_loose_string_assert`.
