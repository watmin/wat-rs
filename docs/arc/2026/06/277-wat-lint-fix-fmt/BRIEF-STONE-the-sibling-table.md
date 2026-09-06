# BRIEF — STONE: the sibling table

Detect adjacent same-typed instances and render them as an aligned table, overriding the exploded
pair-run. Read `[[DESIGN-STONE-the-sibling-table]]` first — it carries the three rulings, why the key
SEQUENCE is part of the definition, and why width is deliberately not consulted.

## READ IN ORDER

1. **`wat-scripts/scratch-pad/rules-corpus-02-gates-and-unknowns.wat:176-180`** — the builder's
   approved shape, **and it has drifted**: 4 pad on one row, 8 plus a stray space on another. The
   formatter must make it exact.
2. **`wat-scripts/scratch-pad/rules-corpus-01-node-facts.wat:121-125`** — a group needing NO padding
   (all values the same width). It must still be a group and must be left alone.
3. **`wat-scripts/fixes/reclaim-service-fixture-names.wat:40-44`** — the POSITIONAL table (D3).
4. **`wat-tests/format.wat:16`** — `(:wat::core::format "{a} {b}" :b 5 :a "x")`, keys deliberately
   reversed. ⚠ **Row 3's counterexample. Grouping on head alone destroys it.**
5. **`wat/fmt.wat`, `AlignPairs` + its map + the emitter's padding pass** — the shape to extend.
   `TableRow` needs the same, **group-scoped instead of form-scoped**.
6. **`wat-scripts/fmt/rules/kwargs.wat`** — the pair-run rule the table must override.
7. **`wat-scripts/fmt/fixtures/kw-table.wat`** — the reproducer, on disk, currently 15 lines.

## SKETCH

```wat
(:wat::core::defrecord :wat::fmt::TableRow
  [form  <- :wat::core::i64      ;; a member
   group <- :wat::core::i64])    ;; the first member's node id

;; wat-scripts/fmt/rules/table.wat  (+ its load-file! line in every driver)
;;   adjacent siblings (consecutive :index under one parent) sharing
;;     - the same HEAD name
;;     - the same KEY SEQUENCE in order      (positional: the same ARITY)
;;   2 or more  ->  assert TableRow for each, group = the first member's id
;;   a member with a TableRow gets NO pair-run Breaks — it stays one line

;; emitter: for each group, compute per-column widths across ALL members from THIS pass,
;; then pad. No rule names a column.
```

## BLAST RADIUS

```
wat/fmt.wat                       TableRow record + map + group-scoped padding
wat-scripts/fmt/rules/table.wat   NEW
wat-scripts/fmt/rules/kwargs.wat  withhold when the form carries a TableRow
wat-scripts/fmt/run*.wat          the load-file! line
wat-scripts/fmt/fixtures/         fixtures for rows 1-5
```

**No Rust. `Break`, `Claim`, `AlignPairs`, `BlankBefore` unchanged. The three walls unchanged.**

## STOP TRIGGERS

- **STOP-1 — the group test MUST include the key sequence.** Head alone destroys `format.wat:16`.
  Row 3 exists for this and it is the row a wrong rule passes everything else on.
- **STOP-2 — no rule may name a column.** `grep -c 'col' rules/*.wat` stays 0; the emitter pads.
- **STOP-3 — do NOT consult line width.** A table stays a table however long a row is. Width driving
  layout is unruled and would be the first such case since the exploded ruling. If a row seems to
  demand it, STOP and report.
- **STOP-4 — if padding drifts across passes, STOP.** Widths come from THIS pass's rendered members,
  never from the input's existing spaces.
- **STOP-5 — if any existing test goes red, STOP.** Capture the whole block; do not re-run.

## ⚠ TRAPS

- A file in `rules/` is **not loaded** until a driver `load-file!`s it.
- `filter` returns a **lazy stream**; `length` raises — `into` a Vector first.
- **Every fixture must `wat --check` clean BEFORE the floor.**
- **Validate a probe FIRES before reading its silence as a result.**

## THE FLOOR IS MINE

`scripts/floor.sh` and `cargo clippy --all-targets -D warnings` are the orchestrator's.
