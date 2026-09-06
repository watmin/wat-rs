# BRIEF — STONE: atoms may ride if they fit

Add the `Width` fact from the walk, an `AllAtoms` rule, and the emitter's inline-vs-explode decision.
Read `[[DESIGN-STONE-atoms-may-ride-if-they-fit]]` first — it pins the one design decision (a rule
grants, the emitter exercises) and carries the width fact's known hazard.

## READ IN ORDER

1. **`wat-scripts/scratch-pad/277-width-as-a-fold.wat`** — the PROVEN width fold, with its control.
   **Copy it; do not re-derive it.** `leaf = length(ast->source)`, `interior = Σ + n + 1`.
2. **`[[NOTE-width-is-a-fact-not-a-rule]]`** — why rete cannot derive this, isolated. **Do not
   attempt a rete derivation; the engine refuses it.**
3. **`wat/grep.wat:25-28` and the `Written` fact** — a `Named` fact is emitted for a StringLit too,
   and `Written` marks a span that holds its node's own text. ⚠ **The width control must exclude
   reader-synthesized nodes** or it reports a defect that is not one.
4. **`wat/fmt.wat`, `AlignPairs` + its map + the emitter's padding pass** — the shape to copy.
   `AllAtoms` is the same: a rule asserts, the emitter consults.
5. **`wat-scripts/fmt/rules/kwargs.wat` and `rules/table.wat`** — the pair-run rule this modifies,
   and the table rule that must keep winning.

## SKETCH

```wat
(:wat::core::defrecord :wat::fmt::Width    [id <- :wat::core::i64  w <- :wat::core::i64])
(:wat::core::defrecord :wat::fmt::AllAtoms [form <- :wat::core::i64])

;; Width: emitted by the WALK in fmt.wat, beside the other facts. Not a rule. Not an aggregate.

;; rules/atoms.wat  (+ its load-file! line in every driver)
;;   a pair collection (map literal, or a form with a trailing pair run) where EVERY value
;;   is a non-compound child  ->  (AllAtoms :form ?p)
;;   NO width test in the rule — a rule may not name a column.

;; emitter: a form with AllAtoms renders INLINE when  current-indent + Width <= 120,
;;          otherwise one pair per line. The budget lives HERE, once.
```

## BLAST RADIUS

```
wat/fmt.wat                       Width fact from the walk; AllAtoms map; the inline decision
wat-scripts/fmt/rules/atoms.wat   NEW
wat-scripts/fmt/run*.wat          the load-file! line
wat-scripts/fmt/fixtures/         fixtures for rows 1-5
```

**No Rust. `Break`, `Claim`, `AlignPairs`, `BlankBefore`, `TableRow` unchanged. Three walls
unchanged. `wat/grep.wat` UNTOUCHED — width is the formatter's, and every grep user would otherwise
pay for a fold they do not use.**

## STOP TRIGGERS

- **STOP-1 — do NOT derive width in rete.** The engine refuses it and the refusal is measured. It
  comes from the walk.
- **STOP-2 — no rule may name a column or a budget.** `grep -c 'col' rules/*.wat` stays 0, and `120`
  appears in the EMITTER, once — not in a rule.
- **STOP-3 — the width control must EXCLUDE reader-synthesized nodes.** Otherwise it reports
  mismatches that are the span's fault, not the fold's. `wat/service.wat:2949` is the worked example.
- **STOP-4 — the sibling TABLE still wins.** A table row must not be re-inlined pair-by-pair, nor
  exploded by the width path. If they conflict, STOP and report which.
- **STOP-5 — if any existing test goes red, STOP.** Capture the whole block; do not re-run.

## ⚠ TRAPS

- A file in `rules/` is **not loaded** until a driver `load-file!`s it.
- `filter` returns a **lazy stream**; `length` raises — `into` a Vector first.
- **`:where` admits only PURE ops.** `string::starts-with` and `rete::or` are both refused; use
  `Node.kind` and separate rules. This cost two probes this session.
- **Every fixture must `wat --check` clean BEFORE the floor.**

## THE FLOOR IS MINE

`scripts/floor.sh` and `cargo clippy --all-targets -D warnings` are the orchestrator's.
