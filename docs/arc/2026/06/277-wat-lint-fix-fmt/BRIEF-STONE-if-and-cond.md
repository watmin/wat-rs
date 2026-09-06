# BRIEF — STONE: `if`'s measurement rides, and `cond`'s clauses align

Two rules the builder ruled and nobody built. **Two new rule files and nothing else.** Read
`[[DESIGN-STONE-if-and-cond]]` first — it carries the before/after, the clause-width measurement
behind the fit test, and the honest note that this is corpus work rather than the 255 unlock.

## READ IN ORDER

1. **`wat-scripts/fmt/rules/match.wat`** — 2 rules, the smallest complete rule file in the arc, and
   the closest shape: a head keyword, a scrutinee that RIDES, children that break. **Copy this.**
2. **`wat-scripts/fmt/rules/table.wat`** — the **fit test** and the structural `?p != 0` guard. Row
   6 needs the fit test's shape; do not re-invent it.
3. **`wat-scripts/fmt/rules/atoms.wat`** — *inline iff every value is an atom AND it fits*. It is why
   `(:else "many")` already rides today while `((:wat::i64::= k 0) "zero")` explodes; the cond rule
   must make both consistent (row 13).
4. **`wat/core.wat:1455`** — `cond`'s macro. **Clauses only, no subject**, terminal `(:else body)`
   required. A rule for `(cond <subject> …)` is a rule for `match`.
5. **`wat-scripts/fmt/run-all.wat`** — the driver. ⚠ **A rule file is inert until a driver
   `load-file!`s it.**

## SKETCH

```wat
;; if.wat — three children; the FIRST rides, the other two break one level in.
;;   (:wat::core::if <test>            <- rides the head line
;;     <then>                          <- Break block
;;     <else>)                         <- Break block

;; cond.wat — each CLAUSE breaks; inside a clause the body RIDES its test; and the
;; bodies ALIGN across the clauses of one cond (AlignPairs is the existing fact for
;; "pad a broken child so the next one rides at a shared column").
;;   (:wat::core::cond
;;     ((:wat::i64::= k 0) "zero")
;;     ((:wat::i64::= k 1) "one")
;;     (:else              "many"))
;;
;; subject to the FIT TEST: a cond whose aligned form exceeds 120 does NOT align.
```

## BLAST RADIUS

```
wat-scripts/fmt/rules/if.wat      NEW
wat-scripts/fmt/rules/cond.wat    NEW
wat-scripts/fmt/run-all.wat       + two load-file! lines
wat-scripts/fmt/fixtures/         the shapes + a cond whose ALIGNED form overflows 120
```

**No Rust. No `wat/fmt.wat`. No existing rule file. No new fact record.**

## STOP TRIGGERS

- **STOP-1 — if either rule needs `wat/fmt.wat` edited, STOP AND REPORT WHICH.** "A new style rule is
  a new file and nothing else" is this arc's acceptance criterion and has held five times. A sixth
  rule that needs the emitter is a real finding about the rule language — **surface it, do not
  quietly widen the emitter.**
- **STOP-2 — build row 6's overflowing cond fixture FIRST and confirm it aligns BADLY before the fit
  test exists.** Rows 4 and 5 pass with a rule that always aligns; only the overflow case can tell a
  working fit test from one that never fires.
- **STOP-3 — do NOT chase `spawn 9/174` or `fmt 2/147`.** Those residues are measured, named and
  owned by an earlier stone (long comments, string literals, unruled `defsurface`, 17-arg calls).
  Row 8 is a non-regression, not a target.
- **STOP-4 — `cond` has NO subject.** Clauses only. If the rule you are writing binds a term after
  the head, you are writing `match`, which is built.
- **STOP-5 — do NOT touch `match`, defect E, or anything in `crates/wat-doc/`.**
- **STOP-6 — if any existing test goes red, STOP.** Capture the whole block verbatim; do not re-run.

## ⚠ TRAPS

- **A rule file is not loaded until a driver `load-file!`s it.** Both new files need a line in
  `run-all.wat`, or every measurement silently reports the OLD behaviour and every row passes for the
  wrong reason.
- **This stone MOVES formatter output on purpose** — 118 `if` sites in the five real files. Do not
  expect byte-identity and do not aim for it; row 8 is a width non-regression against a captured
  baseline.
- **`(:else "many")` rides today by accident**, via the atoms rule. Row 13 says both clause shapes
  must end up the same; a cond rule that only handles the call-test case leaves the inconsistency in
  place with new paint.
- **Row 5 wants a COLUMN.** Report the column the bodies start at and show two clauses of different
  test width sharing it — "aligned" as prose is satisfiable by output that merely looks tidy.
- Every fixture must `wat --check` clean before the floor.

## THE FLOOR IS MINE

`scripts/floor.sh` and `cargo clippy --all-targets -D warnings` are the orchestrator's. Run the cheap
targeted checks — `--check`, the census on the five real files, the fixtures, the 614 — and report the
numbers.
