# FINDING — the corpus gate LOADS 477 files and COMPILES zero rules. 11 cannot compile.

**Measured 2026-09-10** at `6f1d93451`, instrument at `tests/lint/rete-compile-census.sh`.

> ## ⭐ CLOSED 2026-09-10 at `2a9de244a` — `strike-no-rule-that-cannot-compile`
>
> **Every number below is HISTORICAL and no longer reproducible, by design:** the nine rotted
> files are deleted, the two recorded migrations repaired, and `tests/lint/rete_compile_gate.rs`
> now requires every rule-declaring corpus file to COMPILE — with **zero exemption categories**,
> and a test asserting no exemption rune was ever added. The corpus reads **128 compile / 0 cannot**.
>
> ⛔ **The repair found FIVE fence call sites, not the three this finding implied** — and the
> reason is a limitation of the instrument below that anyone re-using it must know: `compile-all`
> **fails fast per file**, so the census reports only each file's FIRST refusal. A second bad fence
> in the same file is invisible until the first is cured. **The census bounds files correctly and
> UNDERCOUNTS sites.**
>
> ⛔ **And the instrument's own anchor was pinned to a mutable fact.** It required
> `to-faithful-clojure-net.wat` to fail — the very file the strike repaired — so curing the corpus
> disarmed the instrument that measured it, permanently. Now fixed: the anchor is a **synthesized**
> fence the script constructs itself. *An anchor must be something you build, never something you
> found.*

## The gap

`every_wat_scripts_file_loads` walks every `.wat` under `wat-scripts/` and calls
`startup_from_file`. That runs **validate**. It never runs rete **compile** — and the four-axis
gate lives there, in wat, not in Rust:

```wat
;; wat/rete/compile.wat:463
(:wat::core::and is-pure is-det is-total is-rete)
```

**pure · deterministic · total · rete-primitive**, with `first-failing-axis` naming which broke. A
rule violating any axis **loads green forever** and dies the instant anything compiles it.

⭐ `wat-rs/CLAUDE.md` already concedes the shape one level in: *"Type-checking is not resolution. A
`def` body nothing forces is never resolved."* This is the next verse: **loading is not compiling.**

## The measurement

| population | count |
|---|---|
| `.wat` under `wat-scripts/` | 477 |
| …matching `defrule`/`defquery` | 144 |
| …of those, codemods matching the token only inside **string literals** they rewrite — no rules of their own | 8 |
| **files that actually declare rete rules** | **136** |
| — compile | **125** |
| — ⛔ **cannot compile** | **11** |

**All 11 fail the same four-axis gate**, at a `where` or a `:then`:

| axis | n | offending heads |
|---|---|---|
| not **total** | 7 | `:wat::core::string::contains?` ×2, `:wat::core::>` ×3, `:wat::core::i64::+`, `:wat::core::i64::-` |
| not a **rete primitive** | 4 | `:wat::core::=` ×2, `:wat::core::not`, `:probe-term::countdown` |

## Which of the 11 are deliberate, and which are rot

**3 declare refusal intent in their headers** — `experiri-then/then-law-a-core-head.wat`,
`then-law-a-core-not.wat` (Law A probes), `probe-stop-a-where-arith-path.wat` (a STOP probe).
Legitimate negatives with no way to say so, because no gate reads this property.

**8 do not**, and three are named here because they are not merely undeclared:

- ⛔ **`wat-scripts/fixes/to-faithful-clojure-net.wat`** and **`to-faithful-clojure-rete.wat`** —
  **RECORDED MIGRATIONS**, the "how to write a codemod" exemplars `CLAUDE.md` mandates copying.
  Neither can execute a single rule. `:fix::convert` dies at `compile-all` on `g4-namespaced`'s
  `(:wat::rete::where (:fix::has-ns? ?name))`, whose body calls partial
  `:wat::core::string::contains?`. A codemod that cannot run is not a recorded migration.
- ⛔ **`scratch-pad/probe-rules-rich.wat`** — its own header asserts *"the expr is fenced
  pure∧det"*. The runtime refutes it: `:wat::core::>` is **not total**. A claim of legality that
  nothing checks, sitting in the file it describes.

The remaining 5 (`rete-truth-maintenance-probes/neg.wat`, `probe-arena-rich-graph.wat`,
`probe-rete-predicate-termination-routes.wat`, `probe-where-cond-fence-execution-split.wat`,
`probe-where-shape-spread.wat`) are unread as to intent. ⚠ `probe-where-shape-spread.wat` opens
*"the EXPRESSIVITY SURFACE, not our corpus"*, so it may be deliberate exploration.

## ⛔ How this was found, and why that is the real lesson

**By accident, for the second time in two days.** The fence type-checker made
`to-faithful-clojure-net.wat` red at load; fixing it prompted a sanity-check drive of the codemod;
that drive hit the totality wall. Nothing was *looking*. The same sentence closes
`FINDING-the-fence-is-a-hole-in-the-type-system.md`: *"nothing has ever looked."*

## The cure, and it is shaped by something that already exists

A compile census as a **gate** over the 136, with a declaration for deliberate negatives. The
mechanism is already built and proven twice in this repo — `rune:lint(red-by-design)` in
`tests/lint/docs_wat_loads_or_declares_why_not.rs`, and `rune:lint(bad-is-banked)` in
`every_wat_bad_fixture_actually_fails.rs`. Both close a category set, demand a sentence, and pin it
to something the gate re-reads every run.

⚠ **Do not start it mid-merge**, and do not fix the two codemods without deciding the expressivity
question first: `g4-namespaced`'s fence is a **bare user-fn call**, one of the exact three forms
`expr_is_provably_boolean` refuses inline and Clara admits — the parked
`DESIGN-widen-the-clause-then-refuse-the-fence.md`. These two codemods are evidence for that
design, not merely bugs to patch.
