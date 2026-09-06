# BRIEF — STONE: the fence refuses what it cannot prove, and the enum gets its name

Two halves, and neither ships alone. **C:** the `:then` fence stops admitting
`:wat::rete::core::match`, because its totality axis is head-level and a match's partiality is
form-level. **A:** mint `:wat::rete::core::enum::name` so the three callers C strands have a road,
and restore their captures. Read
`[[DESIGN-STONE-the-fence-refuses-what-it-cannot-prove]]` first — it carries the completeness census
and the four-questions table that ruled out doing either half alone.

## READ IN ORDER

1. **`wat/rete/compile.wat:742-766`, `then-item-fence`** — all four axes, totality included. **This
   is C's site.** The fence is not broken; it is being asked to vouch for something it cannot see.
2. **`src/intrinsic/rete.rs:191-215`** — `total?`'s contract: *"whether every **HEAD** in `expr`'s
   transitive walk is defined on all its inputs."* **Head-level. Read this before touching anything**
   — it is why C is a refusal rather than a smarter check.
3. **`wat/core.wat:1455`** — `cond`'s macro. It raises `cond: non-exhaustive — needs a terminal :else
   arm` at EXPAND. **This is why `cond` is NOT in C's scope** and row 4 guards that.
4. **`src/intrinsic/record.rs:400-410`** — `:wat::core::variant`, the enum CONSTRUCTOR. A's new
   intrinsic is its reader and belongs beside it.
5. **`src/rete/vocabulary.rs:1050-1090`** — the `core::enum::=` / `::not=` rows and the `Form`-not-
   `Alias` reasoning. **A's row copies this shape.**
6. **`src/rete/vocabulary.rs:1115`, `1133`** — `core::bool::to-string`, and the `i64`/`f64` siblings.
   The family A joins.
7. **`wat-scripts/scratch-pad/277-head-kind-census.wat`** — a caller whose name is a promise it can
   no longer keep. Row 9.

## SKETCH

```wat
;; C — in then-item-fence, beside the four axes. The refusal names the AXIS:
;;   "a :then admits only what the fence can prove TOTAL. `match` is total as a HEAD,
;;    but a match's exhaustiveness is a property of ITS ARMS — form-level, which a
;;    head-level axis cannot see. Use :wat::rete::core::enum::name for a variant's
;;    name, or bind the value in :when."
;; It refuses the EXHAUSTIVE match too: the axis cannot tell them apart, so admitting
;; the good one means admitting both.
```

```rust
// A — the core reader, beside `variant` (the constructor). No accessor exists today;
// `variant` builds, nothing reads.
//   (:wat::core::variant-name <enum value>)  ->  "List"    no leading colon
// then the rete exposure, the shape core::enum::= already uses (Form, arm in
// infer_rete_form), so it is callable inside a :then.
```

## BLAST RADIUS

```
wat/rete/compile.wat      then-item-fence + the diagnostic
src/intrinsic/record.rs   the core reader
src/rete/vocabulary.rs    the enum::name row
src/check.rs              its infer_rete_form arm
3 caller .wat files       the captures restored
tests/rete/…              negative controls for BOTH halves
```

## STOP TRIGGERS

- **STOP-1 — C refuses `match` OUTRIGHT, exhaustive or not.** A fix that inspects the arms to refuse
  only the partial ones **is option B**, it is a second exhaustiveness checker beside `infer_match`,
  and the four questions already ruled it out. If that feels wrong, STOP and report — do not build it.
- **STOP-2 — do NOT change `total?`.** Its head-level contract is correct and documented. Widening it
  to peer inside forms is a different and larger question.
- **STOP-3 — `cond` stays admitted.** It is already guarded at expand. A wall that takes `cond` with
  it is too wide; row 4 is the gate.
- **STOP-4 — the core reader must stand alone.** If `enum::name` only works inside a rule, the
  layering is wrong: the rete row is an EXPOSURE of a core op, not a thing unto itself.
- **STOP-5 — restore only the three sites C strands.** `g7_rule`, `probe-grep-cli` and
  `probe-grep-driver` were correctly resolved with the literal `"symbol"` and are NOT to be churned.
- **STOP-6 — if either half cannot land, ship NEITHER and report.** C alone strands three callers; A
  alone routes around an unnamed hole. That is the whole finding of the four-questions table.
- **STOP-7 — if any existing test goes red, STOP.** Capture the whole block verbatim; do not re-run.

## ⚠ TRAPS

- **`wat/rete/compile.wat` is FROZEN into the release binary.** `cargo build --release` between the
  fence edit and any measurement, or you are testing the old fence. Measured the hard way this
  session: three sabotages read as evidence before that was established.
- **The captures will read `"List"` / `"Keyword"`, not `"list"` / `"keyword"`.** `enum::name` returns
  the wat VARIANT name; the pre-enum captures held `ast-kind` strings. Expected, and the more honest
  of the two — say so in the SCORE rather than letting it surface as a surprise.
- **The census is complete and `match` is the only head in scope.** `first` (head-level partial) is
  already refused by the existing axes; `cond` is already refused at expand. Both were measured.
- **`enum::=` is a `Form`, not an `Alias`, and the reason is in its own comment** — a user enum can
  never have a pre-minted row. `enum::name` inherits that reasoning.

## THE FLOOR IS MINE

`scripts/floor.sh` and `cargo clippy --all-targets -D warnings` are the orchestrator's. Run the cheap
targeted checks — the probes, `-E 'test(rete)'`, `wat_grep`, the three restored callers — and report
the numbers.
