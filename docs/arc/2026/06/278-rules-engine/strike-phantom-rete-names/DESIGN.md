# DESIGN-STONE — `wat-scripts/` proves a file PARSES, not that its names EXIST

> **Origin (2026-09-01).** Class **F2**, `cernere`'s row. Driven at HEAD `51b851c91`. **The row said
> the mandated codemod rewrites into retired forms. It does — and the drive found the larger thing
> underneath.**

## Why — the doctrine's claim is false, and I proved it with an invented name

`wat-rs/CLAUDE.md` states the scratch-pad convention's whole justification:

> *"the `every_wat_scripts_file_loads` gate parses + type-checks **every** `.wat` under
> `wat-scripts/` … so a scratch program that rots goes RED and cannot become a graveyard that reads
> like live code. **All wat stays correct, always.**"*

Driven — a head that has never existed in any registry, in a `def` body:

```
(def :probe-nonsense (… (:wat::rete::core::THIS-HEAD-NEVER-EXISTED …)))
(defn :user::main [] -> nil (println "ran"))
        →  "ran"
```

It type-checks and the program **runs**. A `def` nothing forces is never resolved, so a file under
`wat-scripts/` may name anything. **The gate proves parse + freeze. It does not prove that the names
in the file exist**, which is precisely the rot the convention was written to prevent.

## The live instances

`:wat::rete::core::map` and `:wat::rete::core::filter` — **no `RETE_OPS` rows**
(`grep -cE 'rete_name: ":wat::rete::core::(map|filter)"'` → **0**). Rete's whole
map/filter/fold family is `mapv`, `filterv`, `foldl`, `reduce`. Each phantom appears twice:

| where | as |
|---|---|
| `wat-scripts/fixes/rete-where-per-type-spelling.wat` | **live rename TARGETS** in the mandated codemod |
| `wat-scripts/scratch-pad/probe-arc278-57-round1b-parametric-and-hof.wat:56,62` | **heads in `def` bodies** that type-check clean |

## ⛔ THE CODEMOD'S ROWS CANNOT BE CORRECTED — ONLY DELETED

The obvious fix is to re-point the targets at `mapv`/`filterv`. **That would be worse than the
defect.** `vocabulary.rs:965` quotes `wat/seq.wat`: *"mapv / filterv — the eager forms: force
`map`/`filter`'s **lazy Stream** result to a Vector"*, and `expr_ir/eval.rs:586` says a `Stream` is
*"deliberately absent: the compiled executor has no stream machinery."*

So `:wat::core::map` → `:wat::rete::core::mapv` is **not a spelling change** — it swaps a lazy
`Stream` for an eager `Vector`. A rename table cannot express that, and a codemod that did it would
silently change what programs compute. **There is no valid rename for this pair. The rows go.**

## ★ THE ONE CONTRACT DECISION

**A `:wat::rete::` name written in CODE under `wat-scripts/` resolves — to a `RETE_OPS` row or to a
known form. Prose may name a retired form; code may not.** That distinction is the whole design:
`foldr` and `nth` appear under `wat-scripts/` today **only in comments**, correctly recording that
those rows do not exist. A lint that flagged them would demand the deletion of accurate history.

## ⚠ WHAT THE INSTRUMENT ACTUALLY REPORTS — and why a naive scan is not the gate

A first pass over `wat-scripts/` found **136 distinct `:wat::rete::` tokens, 70 of them
`core::`, 9 not in `RETE_OPS`.** Of those 9, **most are noise**:

- `:wat::rete::core::defn` — a **FORM**, not a row. 15 files use it correctly.
- `:wat::rete::core::` and `…::X` — a bare prefix and a placeholder, both from the codemod's own
  table-building prose.
- `…::enum::`, `…::f64::` — tokenizer fragments; the regex stopped before `=`.
- `foldr`, `nth` — **comments**, both accurate.

**Two are real.** The count is not the finding; the classifier is. Build the instrument to
distinguish code from comment and rows from forms, and let it report — F0.

## Blast radius

`wat-scripts/fixes/rete-where-per-type-spelling.wat` (two rows out), the scratch probe (two dead
`def`s), one new gate under `tests/lint/`, and **`wat-rs/CLAUDE.md`'s claim** — which the gate makes
true for rete names rather than merely narrowing.

## Out of scope — AFFIRMATIVELY CUT

- **Making the gate resolve EVERY head in every `def`.** That needs forcing or a full static resolve
  pass, and forcing a `def` may have effects. `:wat::rete::` names are a closed, enumerable set (79
  rows) and are the family that actually rots — retirements. Textual resolution of those is
  proportionate; the general problem is not this strike.
- **Re-pointing the codemod at `mapv`/`filterv`.** Rejected above, on the semantics.
- **The other four `wat-scripts/fixes/*.wat` rename tables.** Measured: their `:wat::rete::` targets
  all resolve. The gate will cover them going forward.
