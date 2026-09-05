# EXPECTATIONS — STONE: the first layout rules

Fixed BEFORE the strike.

| # | what | command | expected |
|---|---|---|---|
| 1 | it builds | `cargo build --release` | clean |
| 2 | the verb exists and yields comments | a `.wat` probe reading `wat/io.wat` | forms > 0 AND **comments ≥ 10** (printed) |
| 3 | `read-string` untouched | `git diff src/intrinsic/ast.rs` | the existing handler unchanged; only an ADDED verb |
| 4 | the 13 exemptions are GONE | `grep -c 'expect(dead_code' src/edn/render.rs` | **0** — wiring a caller retires them |
| 5 | R1 fires | run the driver on a file with a multi-arg `defn` | arg-spec on its own line, one arg per line, ret-type own line, body own line |
| 6 | R1 fires on an EMPTY argspec | a `defn` with `[]` | `[]` on its **own line** — it is not an exception |
| 7 | **comments survive the FORMATTER** | run over `wat/io.wat` | every `;;` present, same order, **count printed** |
| 8 | **idempotence** | `fmt(fmt(x))` vs `fmt(x)` | **byte-identical** |
| 9 | ★★ **THE ACCEPTANCE** | add `rules/siblings.wat` (R11) | output changes; `git diff wat/fmt.wat` and `git diff rules/defn.wat` are **EMPTY**; no Rust rebuild needed |
| 10 | R11 does what it claims | a form with one child broken and others not | ALL children break |
| 11 | non-vacuity | every preservation assertion | prints the COUNT it examined; a green over zero proves nothing |
| 12 | reader crate green | `cargo test --release -p wat-reader` | ≥107 pass, 0 fail |
| 13 | wat-scripts still load | `cargo nextest run --release -E 'test(every_wat_scripts_file_loads)'` | 1 passed — the new rule files parse + type-check |
| 14 | the floor (ORCHESTRATOR) | `scripts/floor.sh` | 5179+ run, **0 FAILED** |
| 15 | clippy (ORCHESTRATOR) | `cargo clippy --release --all-targets -- -D warnings` | 0 |

**Runtime prediction:** 60-100 min. Largest stone in this arc — the verb is mechanical, the emitter
is the real work, the rules are small.

## Trap-doors named in advance

- **Row 9 is the whole point.** If R11 needs an engine edit, that is a REPORTABLE FAILURE of the
  design, not a thing to work around. STOP-2.
- **A comment pins a newline.** If a layout decision would join a line carrying a `;;`, the comment
  wins. Everything after it on that line would be commented out.
- **`[]` is not an exception** (row 6). 3,247 zero-arg `defn`s currently keep `[]` on the head line;
  the ruling breaks them all out. That is the tool working, not a regression.
- **Idempotence usually breaks on the SECOND pass**, where an indent computed from already-indented
  text drifts. Row 8 is the one most likely to fail.
- **The vacuous green.** Rows 2, 7 and 11 exist because "all comments preserved" over zero comments
  is indistinguishable from success. This was published once in this session.
- **`\;` is a char literal, not a comment.** The reader stone measured it; any pass that scans text
  rather than using spans re-breaks it.
