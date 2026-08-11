# EXPECTATIONS — closure extraction carries `def`-bound names

Written **before** the strike, so the result cannot move the goalposts.
Pairs with `BRIEF-closure-extract-def-arm.md`.

## The scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | the RED gate turns green | `cargo test --release --test function t22_toplevel_defn_references_def_bound_value` | `1 passed; 0 failed` |
| 2 | ★ the package STANDS ALONE | (inside T22) `re_freeze(prologue)` then invoke → `520` | the def resolves in a **fresh** world, not the parent's |
| 3 | ★ the def keeps its ORIGINAL name | (inside T22) `collect_def_names(&prologue)` contains `:my::LIMIT` | present, unrenamed |
| 4 | no closure-extraction regression | `cargo test --release --test function` | whole target green; T1–T21 unchanged |
| 5 | the spawn/closure neighbours hold | `cargo test --release --test kernel`, `--test comms` | green |
| 6 | the build is honest | `cargo build --release` | exit 0 |
| 7 | no lint debt | `cargo clippy --release --all-targets -- -D warnings` | 0 warnings |
| 8 | the whole floor (orchestrator's own re-run) | `scripts/floor.sh` → read the **Summary line** | **4386 passed / 0 failed** or better; never a piped exit code |
| 9 | the arc's real consumer moves | `printf '' \| ./target/release/wat wat-scripts/scratch-pad/probe-arc278-fnforms-reaches-program-types.wat` | no longer raises at `closure_extract.rs:769` |

**Rows 2 and 3 are the load-bearing ones.** Row 1 alone can be satisfied by a package that merely
*mentions* the def; row 2 is the one a wrong fix fails, because it re-freezes the prologue into a
brand-new world and invokes there. Row 3 is the one a "clever" fix fails, by renaming the def to a
synthetic capture name the way captured *locals* are renamed — which cannot work, because
`rewrite_captures` never rewrites a Keyword reference.

**Row 9 is the reason the stone exists**, and it is deliberately NOT a pass/fail on the whole probe:
that probe may surface a *second*, different gap once `def` is carried. A different error at a
different line is a **finding**, not a failure of this stone. The same line raising again is a
failure.

## Independent prediction

- **Runtime:** 20–40 minutes. The arm is localized, the emission helper already exists to copy, and
  the type-dependency question is answered by the unconditional sweep at `closure_extract.rs:299-303`.
- **Diff size:** ~25–40 lines added in `src/closure_extract.rs`, zero deleted beyond the raise
  block. Anything materially larger means the rider has built machinery the rooms already provide —
  weigh that before crediting it.
- **Test delta:** +1 test (T22, already written), +1 fixture (already written). If the rider adds
  more tests, read them: they are welcome only if they assert something rows 1–3 do not.

## Trap doors, named in advance

1. **The unencodable-kind wall.** `Value::Vector` is among the kinds `encode_value_to_ast` refuses
   (`closure_extract.rs:2044-2078`). A user `def` holding a Vector will STOP-1 rather than extract.
   That is **correct behaviour for this stone** and must not be bridged. If it fires on the real
   `defservice` path (row 9), that is the next stone, cleanly named.

2. **A `defn` is a `def` bound to a fn.** `wat/core.wat:1175` expands `defn` to
   `(:wat::core::def ~name (:wat::core::fn ~@rest))`. Function resolution at `:735` wins for these,
   so the def arm should never see them — but if the resolution order is different from what the
   reading suggests, the rider will hit `wat__core__fn` in the encoder and STOP-1. **That STOP is
   the finding**, and it means the resolution chain needs a ruling (STOP-3), not a widened encoder.

3. **Emission ordering.** A def emitted *after* the dep defines will pass rows 1 and 3 and fail
   row 2 only when a dep body reads the def — which T22 does not exercise, because its def is read
   by the *entry*, not by a dep. If the rider places it wrong, the scorecard will not catch it.
   **The orchestrator must read the diff for the emission slot** and confirm it sits between step 2
   (captured bindings) and step 3 (dep defines). This row is deliberately called out because the
   test cannot see it.

4. **A green build over a red corpus.** `cargo build --release` bakes the stdlib; it does not run
   the corpus. Row 6 passing means nothing about rows 4, 5, 8. This trap has bitten this arc before
   (R65) and is listed so it does not again.

## Scoring method

Written **after** the orchestrator's own independent re-run — never from the rider's report. Each
row gets its real result, the honest deltas get named, and the diff gets read for trap-door 3.
A rider's "all green" is a hypothesis until `scripts/floor.sh`'s Summary line says so in my hands.
