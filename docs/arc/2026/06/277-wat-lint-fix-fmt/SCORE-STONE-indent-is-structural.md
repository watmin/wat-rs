# SCORE — STONE: indent is STRUCTURAL

No commit. `Claim` / `ClaimedUnder` untouched (STOP-4). Floor and clippy left to the orchestrator.

## The arm, after

Four rules, one nested form. Pass 1 = pass 2. `IDEMPOTENT=true`.

```
(:wat::core::defn :fix::all
  [x <- :wat::core::i64
   y <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::let
    [a (:wat::core::+ x 1)
     b (:wat::core::+ y 2)]
    (:wat::core::match a
      (n n)
      (_ (:wat::core::+ a b)))))
```

R4's arms sit under the `match` (emitted indent 6), not at source column 67. Every other fixture kept its ruled shape and is idempotent.

## The wall

```
grep -c 'col'      wat-scripts/fmt/rules/*.wat   →  0 0 0 0
grep -c ':indent'  wat-scripts/fmt/rules/*.wat   →  0 0 0 0
```

No rule reads a source column. No rule names an indent. `Break {id, kind}` with `"block"` | `"align"`; the emitter computes every column from what it has already written (`Acc.col`).

## Finding — rete RHS refuses a keyword literal

The DESIGN sketched `:kind :block`. The engine:

```
RhsUnresolvableOperand: operand `:block` … must be a ?var bound by this rule's :when,
or an integer / float / boolean / string literal
```

No Rust change (BRIEF). Kind is therefore a **String** `"block"` / `"align"`. Same two disciplines; the names survived; the type did not. Named, not hidden.

## Finding — the composition fixture was not a legal program

`all-four.wat` copied the DESIGN's `(0 b)` arm. The checker: `pattern must be keyword, symbol, or list; got int`. Same class as the earlier half-broken rewrite. Pattern is now `(n n)`; two arms, same nesting. Layout is what this stone tests.

## STOP-4 held

`Claim` / `ClaimedUnder` / R11's gate are the previous stone's. Not edited. `[[REFUTE-claim-the-forms-you-position-not-the-subtree]]` is unmeasured against this contract; that is the next probe, not this strike.

## Commands

| command | result |
|---|---|
| `cargo build --release` | clean |
| `run-all.wat` on `all-four.wat` | ruled layout, arms under `match`, **IDEMPOTENT=true** |
| `run.wat` on `defn-multi.wat` / `defn-empty.wat` | ruled, idempotent |
| `run-let.wat` on `let-two.wat` | ruled, idempotent |
| `run-r4.wat` on `half-broken.wat` / `unruled-top.wat` | ruled, idempotent |
| `run.wat` on `wat/io.wat` | **COMMENTS=28**, IDEMPOTENT=true |
| `every_wat_scripts_file_loads` | **1 passed** |

---

## ORCHESTRATOR VERDICT — 2026-09-05, weighed against my own re-run

**ACCEPTED. Nothing narrowed, nothing added.** The first strike this session I had no edit to make.

| what | command | result |
|---|---|---|
| ★★ **THE WALL** | `grep -c 'col'` / `':indent'` over `rules/*.wat` | **`0 0 0 0` and `0 0 0 0`** |
| the four-rule composition | `run-all.wat` on `all-four.wat` | ruled shape for all three forms nested, **`IDEMPOTENT=true`** |
| floor | `scripts/floor.sh` | **5179 run, 5179 passed, 0 FAILED, 18 skipped** |
| clippy | `--all-targets -D warnings` | **0** |

```
(:wat::core::defn :fix::all
  [x <- :wat::core::i64
   y <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::let
    [a (:wat::core::+ x 1)
     b (:wat::core::+ y 2)]
    (:wat::core::match a
      (n n)
      (_ (:wat::core::+ a b)))))
```

R4's arms sit under the `match` at emitted indent 6 — **not at source column 67**, which was the
whole defect. The collision class is now unrepresentable: a rule cannot name a column, so two rules
cannot name different ones.

### ⛔ AND A PREDICTION OF MINE THAT THE RE-RUN REFUTED

`[[REFUTE-claim-the-forms-you-position-not-the-subtree]]` said, of the `Claim` granularity defect:
*"with columns gone some collisions may simply cease to exist, so that is re-measured AFTER."*

**Re-measured. It did not cease to exist.** The controlled pair is byte-for-byte unchanged under the
new contract:

```
do INSIDE a defn    (:wat::core::do (println "a") (println "b") (+ x 1)))    ← 90 cols, R11 INERT
do at TOP LEVEL     (:wat::core::do
                      (:wat::kernel::println "a")   …                        ← R11 works
```

**The two defects are independent.** `ClaimedUnder` blocks the default rule by ANCESTRY, and ancestry
has nothing to do with columns. The Claim-granularity finding stands unchanged and needs its own
stone — and STOP-4 was right to forbid fixing both at once, because had they been fixed together
this refutation of my own hypothesis would have been invisible.

### Both of the strike's own findings are real, and one is better than briefed

**1 — the rete RHS refuses a keyword literal.** The DESIGN sketched `:kind :block`;
`RhsUnresolvableOperand` allows only a bound `?var` or an integer / float / boolean / string. So
`kind` is a `String`. **That is a genuine engine constraint, reported rather than worked around**,
and the BRIEF forbade a Rust change.

⭐ **And the strike walled the weakness it introduced**, unasked: an unknown kind raises
`assertion-failed! "fmt: Break.kind must be block or align"` rather than silently defaulting
(`wat/fmt.wat:146-154`). Stringly-typed by necessity; loud by construction. Better than the brief.

**2 — my composition fixture was not a legal program.** `all-four.wat` used `(0 b)` as a match arm;
the checker refuses it (*"pattern must be keyword, symbol, or list; got int"*). **I wrote that
fixture in the DESIGN.** Rewritten to `(n n)`, same nesting, same layout question.

### Not disputed

`Claim`/`ClaimedUnder`/R11's gate untouched (STOP-4 held). Every prior fixture keeps its ruled shape
and is idempotent. `wat/io.wat`: **COMMENTS=28**, count printed. `every_wat_scripts_file_loads` 1/1.
