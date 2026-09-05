# EXPECTATIONS — STONE: keyword args and aligned values

| # | what | expected |
|---|---|---|
| 1 | it builds | clean |
| 2 | ★ no positional → nothing rides | `(:probe::R :a 1 :b 2)` → head alone, then `:a 1` / `:b 2` |
| 3 | ★ a positional RIDES | `defservice`'s shape — the name stays on the head line |
| 4 | ★★ **values ALIGNED** | `:file` / `:reason` / `:line` / `:col` — every value starts in one column |
| 5 | ★★★ **no rule names a column** | `grep -c 'col' wat-scripts/fmt/rules/*.wat` → **0 in every file** |
| 6 | ★★ idempotent | alignment must not drift on pass 2 — `IDEMPOTENT=true` |
| 7 | a ONE-PAIR call | report the shape; do not carve out unasked |
| 8 | ruled shapes hold | every existing fixture, ruled + idempotent |
| 9 | three walls stand | disagreeing-kind sabotage raises; `ClaimedUnder` 0 |
| 10 | comments survive | `run.wat` on `wat/io.wat` → **COMMENTS=28**, count printed |
| 11 | wat-scripts load | `every_wat_scripts_file_loads` 1 passed |
| 12 | ★★★ **THE REAL GATE — 615 doc examples** | run every `@example` through the formatter; report **how many changed**, **how many still exceed 120**, and **the worst remaining shape** verbatim |
| 13 | floor (ORCHESTRATOR) | 5179+ run, **0 FAILED** |
| 14 | clippy (ORCHESTRATOR) | 0 |

**Runtime prediction:** 50-80 min. The rule is small; the emitter's padding pass is the work.

## Trap-doors named in advance

- **Row 12 is the point.** Every stone in this arc until now was verified against fixtures written to
  match the rule being verified, and the first contact with real input found three defects. **615
  real examples is the corpus that matters for the priority target.**
- **Row 5 guards the previous stone.** Alignment is exactly the moment a rule reaches for a column.
- **Row 6 is where this usually breaks.** Padding computed from already-padded text drifts, the same
  class as indent-from-source. It must come from the emitted first tokens of THIS pass.
- **Row 3's failure looks like success** if every fixture happens to have no positional — `defservice`
  is the one that discriminates.
- **Row 7 is a REPORT, not a change.** A single `:key value` becoming two lines may be right
  (exploded) or may want a carve-out; that is the builder's call, not the strike's.
- **The vacuous green:** row 10 prints the comment COUNT.
