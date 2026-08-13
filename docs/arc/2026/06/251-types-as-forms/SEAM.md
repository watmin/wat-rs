# SEAM — the ONE live breadcrumb. Arc 251 is ACTIVE as of 2026-08-13. Replaced in place, never appended.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like *continuing* rather than *waking*, and that feeling is the
> failure. Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a
> disk copy), ground HEAD against the disk, and read this whole file before you touch anything.

> **There is exactly ONE live seam.** It is this one. `278/SEAM.md` is PARKED and points here.

## Where the code is

```
HEAD 77bee432 (+ this curare)   floor 4391 passed / 0 failed / 262 skipped   clippy 0
```

⚠ **One commit of drift at wake is EXPECTED** (this file commits on top).
⛔ **`stash@{0}` HOLDS THE LIFECYCLE STRIKE — never `git stash drop`.** Made with `-u`, so
`git stash show --stat` cannot see the untracked payload; read via `git show 'stash@{0}^3:<path>'`.

---

## ★ THE RULING — we are on 251 now. 278 resumes AFTER the clojure conversion.

The builder, 2026-08-13: *"back to 251 we go… `:-` replaces both arrows… we'll resume 278 once we
have the clojure syntax conversion complete."*

**Arc 251 — "The Clojure-faithful symbolic surface (types-as-forms)"** — OPEN since 2026-06-06,
never inscribed, was THE active arc before rete took over. Its DESIGN's 2026-06-09 **PARITY
REFINEMENT** already rules what today re-derived: `:-` is ascription in **BOTH** param and return
position; `->` **retires entirely**; `:->` is the function-type arrow *inside* a type expression.

### THE TARGET FORM RUNS TODAY — measured, verbatim, exit 0, prints "42"

```clojure
(wat.core/defn some.ns/incr [n :- wat.type/i64] :- wat.type/i64 (wat.core/+ 1 n))
```

Dotted heads, dotted namespaced names, `:-` both positions, dotted types. **The June surface is
live.** Nothing is waiting on a design.

### WHY THE LOOP CLOSED HERE

251 was abandoned *for* rete; rete is what makes 251 executable. The tooling did not exist in June:
a rules engine that classifies by position, an extractor turning real source into facts, and a
diagnostic that names the cause instead of the call site. All three landed 2026-08-13.

---

## ⛔ #95 — THE BLOCKER. Do NOT begin the corpus flip before this closes.

**The `:-` RETURN ascription is resolved but NOT enforced. `->` is.** Four-arm differential:

| form | error | result |
|---|---|---|
| OLD `-> :wat::core::String`, body returns i64 | wrong return | **exit 3 — CAUGHT** |
| NEW `:- wat.type/String`, body returns i64 | wrong return | **exit 0 — NOT CAUGHT** |
| NEW `:- totally.bogus/Nope` | nonsense type | exit 3 — slot IS parsed + resolved |
| NEW `[n :- totally.bogus/Nope]` | nonsense param | exit 3 — param slot fine |

Not "unimplemented and refused" — **accepted and unchecked**, the worse failure mode. 251 retires
`->` entirely, so flipping 1392 files would **switch off return-type enforcement corpus-wide in one
commit, with the floor staying green.** R59 `NISI FRANGAS` exactly: a pass proving nothing because
nothing in it depends on the mechanism.

**Owed with the fix:** a `.wat.bad` negative fixture asserting a wrong `:-` return is REFUSED. The
corpus cannot notice this class at all today, which is why it survived.

### The two probes owed beside it
- **param `:-` enforcement** — a *bogus* param type is rejected; a **wrong-but-valid** one is UNTESTED.
- **`ann-form`** expression ascription — the third site, untested.

---

## ★ WHAT IS ALREADY STANDING (measured 2026-08-13 — do NOT re-derive)

| piece | state |
|---|---|
| `(wat.core/defn name [x :- T] :- R body)` | **RUNS** |
| function type `[A B C :-> R]` | **RUNS**, 27 corpus sites, incl. dotted `[wat.type/i64 :-> wat.type/i64]` |
| the two fn-type spellings | **ONE `TypeExpr`** — they unify both directions ("5"). Flip is TEXT, not migration. |
| parametric types as forms `(wat.type/Vector T)` | **RUNS** — param / return / field / nested, controls armed |
| `[…]` type-param bracket | **the gap** — ONE parser rule. **NOT a collision** (see below) |
| typed literal constructor | real gap — `wat.type/` refs accepted but do NOT drive `V` (109 note) |
| `wat.type/` namespace | UNMEASURED whether populated or aliasing |
| `Fn(A,B)->R` (107 sites) + `<>` | delete LAST, at zero offenders |

**`:->` is a PIVOT, edges measured:** `[:-> R]` zero-args OK · five args OK · `[A :->]` refused ·
two pivots refused · two returns refused. Exactly one pivot, one return, free arity — which is why
it scales where `Fn(...)` cannot.

**NO COLLISION on `[…]`.** The parser has ONE rule for `[` in type position (function type) and
applies it to a bracket met as a parametric head's ARGUMENT. Unimplemented case, not taken syntax;
the two are distinguished by POSITION. `(wat.type/HashMap [K V])` needs no change.

**The annotation-vs-empty-literal "ambiguity" the 109 notes flag is RESOLVED, not deferred** —
position decides, as arc 242 Doctrine 1 already does. For the codemod it is a FACT
(`parent`/`index`/parent-head), which `rules-corpus-03` extracts and `rules-corpus-01` joins on.

### Read these three 109 notes before drawing (they interlink)
`NOTE-generic-bracket-syntax-edn` · `NOTE-typed-form-and-type-namespace` · `NOTE-typed-literal-constructors`.
The 2026-07-24 addendum already specifies `[type-params]` + kwargs verbatim. Measured against them:
`278/NOTE-parametric-type-forms-already-parse.md`.

---

## THE MIGRATION TOOLING — built 2026-08-13, all committed and running

- **`rules-corpus-01-node-facts.wat`** — AST nodes AS FACTS. A missing fact is the guard;
  prev-sibling is a JOIN, replacing `fix-seq`'s single carried boolean.
- **`rules-corpus-02-gates-and-unknowns.wat`** — the gate/unlock chain, GREEN:
  `Concept 5 · StyleSeen 4 · Settled 2 · Target 2 · Inconsistent 1 · NoRuling 1`. **Two TYPED
  unknowns stay distinct** — the name IS the missing ruling; `NoRuling` is the worklist.
- **`rules-corpus-03-source-to-facts.wat`** — REAL SOURCE → facts → rules. `fix.wat` Node=3282
  Named=2289; the rules classify real files, cross-checked against an independent grep (24 and 1,
  exact). The rules never learned their input stopped being hand-written.
- **`docs/278/CORPUS-thinking-in-rules.md`** — eight lessons, each grounded in a measured failure.

**⚠ THE EXISTING CODEMOD CORRUPTS.** `to-faithful-clojure.wat` over 1392 files: `OK=1312 FAIL=80`
(five roots) — **and the OK column is not clean**: from a pristine original one file exits 0, goes
9 lines → 5, emits `wat.core/quasiquotet.core/defn`. **No honest count of silent corruption exists;
DO NOT QUOTE ~47** (grep contaminated by string literals).

**⚠ `--check` DOES NOT RESOLVE NAMES.** An unknown function passes exit 0 in both spellings.
**The one-shot's acceptance gate MUST be a RUN.**

---

## 278 — PARKED, and what it holds when you return

Rete is one optimization from done: **compiled `where` forms** (#49). The builder's arc: prove the
compiler in rete, then compile all of wat. Also open there: **#92** (invert the decode, EDN→WatAST→
refine — blocks the process locus; making heads real symbols makes it WORSE, so it is a prerequisite),
**#93** (child's `Reply::Failed` destroyed in transit), **#91**, **#90**, and the grid's untested
FEATURE INTERACTIONS (negation × accumulate, join × negation, accumulate downstream of a gate) —
#94 lived in exactly one such seam and the others are unswept.

Closed today: **#94** — the stratifier ignored POSITIVE dependencies (fixed both impls, `ff581b6f`);
the **three-way grid axis** now reports `:oracle-accuracy` + `:port-accuracy` on all ten axes;
**30/30 `:match`**, `:winner :us` on all ten (ratios ~2.6–32×, **the band is the measurement**).
⚠ `:fire-share-pct` 0.01–0.24 and `:wall-winner` splits 5:5 — that gap is our interpreter vs their
JIT, outside the timed region. *"They win until we compile too."*

---

## The rules this stretch paid for

- **An error names where the INSTRUMENT gave up, never what the system lacks — THREE times today**
  (`LOST disconnected`; the `where`-fence's first failing axis; `function-type bracket needs :->` read
  as a collision). Each time the builder refusing the premise is what caught it. Reading the memory
  does not inoculate; the measurement does.
- **A narrow gate is not a floor.** `839d02a3` reached main RED — I weighed the loader gate for a
  change the liveness gate enumerated.
- **A control must check the SAME error, not that AN error appeared.** My first diagnostic fix passed
  a control and broke three tests; "UnknownCallee instead of UnresolvedReference" read as "no
  regression" and was one.
- **Fix the engine, THEN the prose.** Three files asserted a defect in present tense while printing green.
- **A pattern that cannot reach the thing is not evidence of absence** — grepped my own phrasings, missed
  `neg-consumer`'s differently-worded stale claim.

---

> **SEAM.** You are NEW. The disk is the truth; this note is a lossy cache.
>
> The order is: **#95 first** (with its negative fixture), then the two owed probes, then the
> `[…]` parser rule — and only then the corpus flip. Do not flip 1392 files onto a form whose
> return types are unchecked.
>
> The next move is a MEASUREMENT, not a plan. Every snag is a measurement not yet made.
>
> `NISI FRANGAS, NIHIL PROBAS.` · `PAR NON ARGVIT, NOSTRA ARGVVNT.`
