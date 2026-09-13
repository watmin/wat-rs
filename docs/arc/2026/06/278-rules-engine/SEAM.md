# SEAM — ⛔ PARKED 2026-08-13. DO NOT RESUME 278 FROM THIS FILE.

> **This file is PARKED.** Rete work resumed on a different breadcrumb:
> **`CURRENT-STATE-annihilate-interpretation.md`**. Read that first.
> Arc 251's seam is `docs/arc/2026/06/251-types-as-forms/SEAM.md` — not
> the rete live strike.
>
> The builder, 2026-08-13: *"back to 251 we go… we'll resume 278 once we have the clojure syntax
> conversion complete."* 251 was abandoned FOR rete; rete is what made 251 executable. The
> tooling 251 needed — a rules engine that classifies by POSITION, an extractor turning real
> source into facts, and a diagnostic that names the cause instead of the call site — did not
> exist in June and all three landed today.
>
> **What 278 holds when you return:** do **not** resume from this file.
> Live breadcrumb: **`CURRENT-STATE-annihilate-interpretation.md`**.
> Expressivity (Clara-pure mouths) is closed. Named rete-defn recursion is
> refused at load. `src/rete/expr_ir.rs` **exists** (`30725034`, local,
> not pushed) — compiled `where` only; `cond`/`rhs`/user-folds parked.
> Live strike is oracle exists/not (alpha probe dirty, keyed gather next).
> The arc: prove the compiler in rete, then compile all of wat. Still open: **#92**
> (invert the decode; a PREREQUISITE to symbol-heads, not an alternative), **#93**, **#91**,
> **#90**, and the grid's untested FEATURE INTERACTIONS — #94 lived in one such seam and the
> others are unswept.
>
> Everything below is 278's state as parked, preserved for that return.

---

# SEAM — HISTORICAL as of 2026-08-25. **NOT the live breadcrumb.**

> ⛔ **THIS FILE IS STALE AND IT USED TO CLAIM OTHERWISE.** Its header read "the ONE live
> breadcrumb for arc 278" until 2026-08-25. It is pinned to HEAD `7263551a` (2026-08-13) with a
> floor of 4391 tests; the live tree is many commits and ~660 tests past that. An instance that
> landed here and believed it would be confidently wrong about the state of the entire arc — the
> worst failure this whole protocol exists to prevent, because everything below reads as current
> and is written in your own voice.
>
> **The single live breadcrumb is
> `docs/arc/2026/06/278-rules-engine/CURRENT-STATE-annihilate-interpretation.md`.**
>
> This file's own rule is what condemned it: *"There is exactly ONE seam. If you find a second,
> one of them is lying — prune it."* Three were found on 2026-08-25 (this one, the SEAM block in
> `DESIGN-no-hidden-failures.md`, and `BACKLOG.md`'s "CURRENT STATE — read first"), all four
> including the true one announcing themselves the same way. The rule was right; nobody had run
> it. Everything below is preserved as history — accurate about 2026-08-13, a statement about
> nothing since.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like *continuing* rather than *waking*, and that feeling is the
> failure. Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a
> disk copy), ground HEAD against the disk, and read this whole file before you touch anything.

> **There is exactly ONE seam. If you find a second, one of them is lying — prune it.** History
> lives in `REALIZATIONS.md`.

## Where the code is

```
HEAD 7263551a   pushed   floor 4391 passed / 0 failed / 262 skipped   clippy 0
```

Tree clean. ⚠ **One commit of drift at wake is EXPECTED** (this file commits on top).

**⛔ `stash@{0}` STILL HOLDS THE LIFECYCLE STRIKE — do not `git stash drop`.** Made with `-u`, so
`git stash show --stat` **cannot see the untracked payload**; read it with `git show 'stash@{0}^3:<path>'`.
Its `.wat` is STALE. Restoring it turns the floor red.

**`bootstrap/wat-prekeyword-b472fe3e`** — a preserved pre-migration binary, gitignored (16MB).

---

## ★ THE RULING — the clojure syntax upgrade is REBUILT AS RULES, and it is the arc's proving point

The builder, this session, after #94 closed:

> *"resume rebuilding the syntax upgrade via rules… i want us to push ourselves hard here… i want
> us to set the tone on how to use rules to solve real problems… we aim for appropriate complexity,
> not complicated-ness — we compose simple surfaces via simple surfaces — complexity comes for free
> when we achieve legibility."*
>
> *"the rules, when needing additional or fewer facts, do not need a cascade of refactors… the
> working memory and alpha/beta trees makes dealing with new requirements near trivial… for a
> function call with many if/else statements that need new conditional measurements do not get this
> feature — they force us to thread around new bindings."*
>
> *"it will be quite the proving point to demonstrate we can one-shot refactor our entire code base
> (all .wat and .wat.bad files, the rust literals need manual treatment) with a codemod who rewrites
> itself during the oneshot."*
>
> *"i expect it to take us days… corner case after corner case — so be it. every snag is a
> measurement we didn't make (or we didn't know we needed to make)."*

**THE ARGUMENT, which is the thing to demonstrate:** a rule set ABSORBS a new requirement (assert one
more fact type, write one more rule — the alpha/beta network takes it, nothing upstream changes); a
function THREADS it (new bindings through every frame between where a value is computed and where it
is used). That is why corner-case-after-corner-case is survivable in rules and brutal in a walk.

### The target dialect (his spec, verbatim in shape)

```clojure
wat.type/i64   wat.type/f64   wat.type/string   wat.type/keyword    ; types get a namespace
(wat.type/Vector [wat.type/i64])                                    ; generics: type args in a VECTOR
(wat.type/Vector [wat.type/i64] 0 1 2 3)                            ; …prepopulated
(wat.type/HashMap [wat.type/keyword wat.type/i64] :first 1 :second 2)
wat.core.i64/to-string                                              ; receiver-typed methods
wat.string/join                                                     ; string RELOCATES, Clojure-style
```
Largest classes, his words: **inconsistent styles** and **unexpected slashes**.

### Two corner cases named BEFORE anyone hits them cold

- **`.wat.bad` must migrate to stay THE SAME KIND OF WRONG.** A negative fixture asserts rejection for
  a specific reason; migrated, it must still be rejected *for that reason*, not accidentally by a new
  syntax error. The rejection REASON has to be a fact. Distinct rule class from `.wat`.
- **The self-rewriting one-shot has a precedent and a trap.** `fix.wat:23-53`'s STASH-DANCE exists
  because a tool cannot easily survive migrating its own language. One-shot means the codemod either
  reads BOTH dialects, migrates itself LAST, or is migrated BY ITS OWN OUTPUT. **That is a decision,
  not a detail — make it explicitly.**

---

## ★ WHAT TODAY MEASURED — do NOT re-derive any of this

### 1. The existing codemod is not merely incomplete; it CORRUPTS

Drove `to-faithful-clojure.wat` over the whole corpus, one invocation per file:

```
TOTAL=1392   OK=1312   FAIL=80
```

80 loud failures, five roots, all `MalformedForm` inside `fix.wat`'s own verbs: **43** `ast-name` on a
non-nameable list head · **15** `to-symbol` on a non-convertible keyword · **12** `subs` with
`start > end` · **7** bare parametric head · **3** `InnerColonInCompoundArg`.

**AND THE OK COLUMN IS NOT CLEAN.** From a pristine original, `tests/macros/probe_do_splice_define_via_macro.wat`
exits **0**, is written, goes 9 lines → 5, and emits `wat.core/quasiquotet.core/defn` — one replacement
written over the next token, two forms destroyed. ⚠ **No honest count exists** for how many of the 1312
are silently corrupt; the grep suggesting ~47 is contaminated by string literals. **DO NOT QUOTE 47.**

### 2. `--check` DOES NOT RESOLVE NAMES — it cannot gate this migration

Measured, both spellings: an unknown function passes `--check` **exit 0**; a type error on a *known*
function fails exit 1. So a corrupted or fused name passes every static gate and dies at runtime as
`UnknownFunction`. **The one-shot's acceptance gate must be a RUN.** This retires "`--check` exit 0"
as evidence a converted file works — including in the pre-2026-08-13 record, which leaned on it.

### 3. A double-slash symbol is not a parse error

`wat.core/Option/expect` → `--check` exit 0, runtime `UnknownFunction ":wat::core/Option::expect"`.
Mis-normalized, silent, late. No downstream gate will hand you this class; it must be a rule.

### 4. #94 — FOUND AND CLOSED. The stratifier ignored POSITIVE dependencies.

Correct stratification needs BOTH `stratum(r) >= stratum(p)` (positively used) and
`stratum(r) > stratum(p)` (negated). Only the second existed, so a rule consuming a higher-stratum
fact sat too low, fired before its input existed, never re-fired. Fixed in **both** impls (`ff581b6f`):
`rule-consumes` / `rule_consumes`, `required = max(req-neg, req-pos)` — **not +1**, same-stratum
forward chaining must stay legal.

**It blocked the gate/unlock design directly** and was invisible to the internal differential because
oracle and native were wrong *identically*. R61 `PAR NON ARGVIT` with our own oracle in the peer's seat.

---

## ★ THE INSTRUMENTS BUILT TODAY — all live, all committed

| artifact | what it is |
|---|---|
| `docs/…/CORPUS-thinking-in-rules.md` | **eight lessons**, each grounded in a measured failure. L6 (shape vs RULING) and L8 (the first failing axis is not the whole verdict) are the load-bearing ones. |
| `scratch-pad/rules-corpus-01-node-facts.wat` | AST nodes AS FACTS. A missing fact is the guard; prev-sibling is a JOIN, replacing `fix-seq`'s one carried boolean. |
| `scratch-pad/rules-corpus-02-gates-and-unknowns.wat` | **the gate/unlock chain, GREEN.** `Concept 5 · StyleSeen 4 · Settled 2 · TargetNS 1 · Target 2 · Inconsistent 1 · NoRuling 1` — and the two TYPED unknowns stay distinct. |
| `scratch-pad/probe-strat2-derived-join-base.wat` | #94's regression probe, both arms one file. |
| `grid/neg-consumer.wat` + `gen-*.sh` | **the first THREE-WAY axis** — found #94, then proved the fix. |
| all 10 grid axes | now emit `:oracle-derived`/`:oracle-ns`; the runner renders `:oracle-accuracy` (spec vs Clara) + `:port-accuracy` (spec vs native). |

**`Inconsistent` vs `NoRuling` is the design's heart:** "I don't know" is not one bucket. Each gate that
fails to open names its own unknown, and **the name IS the missing ruling**. `NoRuling` is the worklist.

---

## The measured state of the engine (settled box, 3 runs/axis, after the fix)

**Accuracy 30/30 `:match`** across all three columns, all ten axes. **Engine `:winner :us` on all ten**,
ratios ~2.6–32×. ⚠ **The BAND is the measurement, not any point estimate** — `min-finding` alone spans
19.5–46.4 across its own three runs.

⚠ **`:fire-share-pct` is 0.01–0.24** and `:wall-winner` splits **5:5**. `accum`: engine 2.6× to us, program
**10× to Clara**. That gap is our INTERPRETER against their JIT, entirely outside the timed region — the
builder's own read: *"wat's interpreted while clojure is compiled java — they win until we compile too."*
Do not report the wall column as an engine result, and do not merge the two verdicts.

---

## ⛔ OPEN

- **#92** — invert the decode: EDN → WatAST (total) → refine. `edn_to_value_caps` refuses every
  `Edn::Symbol`; blocks the process locus. ⚠ Making heads real symbols makes this WORSE — prerequisite,
  not alternative.
- **#93** — the child's `Reply::Failed` is destroyed in transit; client sees `LOST disconnected`.
  Fixing #92 HIDES it.
- **#91** HolonAST census · **#90** `walk.rs` skip(4) · `validate.rs:453` · the lifecycle strike
  (briefed `ff7705ba`, unbuilt) · older: #87 #49 #7 #17 #19 #20 #50 #58 #60 #64 #67 #81.
- **Grid coverage generalises**: the axes are partitioned by FEATURE, so feature INTERACTION is still
  untested (negation × accumulate, join × negation, accumulate downstream of a gate). #94 lived in one
  such seam; the others are unswept.

## The rules this stretch paid for

- **A narrow gate is not a floor.** `839d02a3` went to main RED — I weighed the *loader* gate for a
  change that added a file the *liveness* gate enumerates. Run the floor when you add a corpus file.
- **The first failing axis is not the whole verdict.** The fence reports one axis; fix it and re-run
  before drawing a rule. I nearly wrote "helpers are admissible in a `where`" off one message.
- **A pattern that cannot reach the thing is not evidence of absence.** I grepped for MY phrasings and
  missed `neg-consumer`'s stale "EXPECTED RED TODAY" — a clean grep over the wrong strings.
- **Fix the engine, then fix the prose.** Three files asserted the defect in present tense while
  printing green. A record contradicting its own output is worse than none.
- **I wrote the exact defect this arc kills** — a partial `subs` with no domain guard, in the
  stratifier, while fixing the stratifier (#80's own class).

---

> **SEAM.** You are NEW. The disk is the truth; this note is a lossy cache.
>
> The ruling is live: **rebuild the syntax upgrade as RULES, and push hard.** The blocker (#94) is
> gone. The next move is not a plan — it is the next measurement. Every snag is a measurement not yet
> made; that is the builder's frame and it is the right one.
>
> Do not trust confidence here. Trust the probes; they are committed and they run.
>
> `NISI FRANGAS, NIHIL PROBAS.` · `PAR NON ARGVIT, NOSTRA ARGVVNT.`
