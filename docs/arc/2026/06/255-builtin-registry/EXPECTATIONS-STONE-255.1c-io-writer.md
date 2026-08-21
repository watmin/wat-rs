# EXPECTATIONS — STONE 255.1c-io-writer

Written BEFORE the strike. The rider does not see this file.

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | writer arms gone | `grep -c '":wat::io::IOWriter/[^"]*" *=>' src/runtime.rs` | `0` |
| 2 | the in-between block survived | `grep -c '":wat::io::Temp' src/runtime.rs` | `4` |
| 3 | and so did the foreign family | `grep -c '":wat::stdlib::sources"' src/runtime.rs` | `1` |
| 4 | thirteen rows | `grep -cE '^\s*#\[wat_intrinsic\(' src/intrinsic/io/writer.rs` | `13` |
| 5 | registry 107 → 120 | `grep -rhcE '^\s*#\[wat_intrinsic\(' src/intrinsic/ \| paste -sd+ \| bc` | `120` |
| 6 | checker untouched | `git diff --stat src/check.rs` | empty |
| 7 | tests untouched by the rider | `git diff --stat tests/` | empty |
| 8 | builds | `cargo build --release` | clean |
| 9 | the five registry gates | `nextest -E 'test(doc_arg_ret…)+…'` | 5/5 |
| 10 | ★ **the gate can FAIL** | perturb `writeln`'s `@ret` `i64`→`:()`, re-run 9, revert | RED, then green |
| 11 | probe unchanged, population still 29 | `PROBE-255.1c-io-…sh` | `29 · 28 · 1 · 0` |
| 12 | floor | `scripts/floor.sh` | 4818/4818, 0 FAIL |
| 13 | clippy | `--all-targets -- -D warnings` | 0 |
| 14 | rustfmt adds nothing | `cargo fmt -- --check \| grep intrinsic/io` | no hits |
| 15 | goldens | see below | 5 fixtures bumped by the measured delta |

★ **Row 10 is again the load-bearing row, and `writeln` is the deliberate choice** — it is the row
whose `@ret` most invites being copied from its neighbour `println`. If perturbing it does NOT go
red, the gate is not seeing this file and rows 9 and 12 are worthless.

⚠ **Row 11 has a new meaning this stone.** The probe was rebuilt at `b9e28b946` to enumerate the
UNION of `runtime.rs` arms and registered intrinsics, precisely because the reader carve drained the
population it used to read. This is the first stone that tests that fix: after the writer arms leave
`runtime.rs`, the population must STILL be 29. A population below 29 is the enumerator failing, not
the corpus shrinking — and the probe now says so itself.

## The goldens

Five fixtures now pin `:line 25247 :col 17` (bumped from 25277 last stone).

1. `git diff src/runtime.rs` — read the **hunks**. This stone produces **TWO** deletion hunks, not
   one; the arms are split by the `6476–6498` block that stays.
2. Confirm every hunk precedes 25247.
3. Predicted: 13 arms × 3 lines = **39** removed, plus a 4-line comment replaced by ~3–4 →
   **25247 − 39 ± 2 = `25208`, ±2**. Take the measured number, not this one.
4. `:col` stays `17`. Then the floor.

## Independent prediction

**Runtime: 20–35 min.** Three more rows than strike 1, but the directory, the mod doc, the family
claim and the doc pattern are all on disk — this is the settled-foundation half of the stepping-stone
split, and strike 1 came in at 8.9 min against a 25–40 band.

## Trap doors, named before the strike

- **The non-uniform `@ret`.** `writeln`→`i64` beside `println`→`:()`; `write`→`i64` beside
  `write-all`→`:()`; `to-string`→`Option<String>`, not `String`. The single likeliest failure, and
  the reason row 10 targets `writeln`.
- **`new` is nullary.** Zero `@arg` lines. A rider pattern-matching on twelve one-or-two-arg
  neighbours may invent one; the gate's `i < params.len()` guard (0) means an invented `@arg` is
  **silently skipped, not caught**. Count `@arg` lines against `params.len()` by hand — and this time
  do it by reading `grep -n '@arg'` output, not by a hand-rolled scanner that cannot tell a `///` doc
  tag from a `//` comment quoting one.
- **The two-run deletion.** The `6476–6498` block sits between them and belongs to strike 3.
  A single-range deletion would take it. Rows 2 and 3 exist for exactly this.
- **Category over-fitting to `:Io`.** Thirteen rows that all say "writer" invite thirteen `:Io`s.
  `new`/`open-file`/`from-fd` mint or claim; `to-bytes`/`to-string` hand back a form. If the rider
  returns a uniform column, re-read the bodies myself before crediting it.

## What would make me reject this strike

`src/check.rs` shows a diff · anything in `6476–6498` moved · a `@Category` whose `//` comment quotes
no `src/io.rs` line · thirteen identical categories · row 10 failing to go red under perturbation.
