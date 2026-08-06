# DESIGN-STONE — the rete `defn`: the rete language gets a DECLARED UNIT

> **RULED by the builder 2026-08-06:** *"i think we need a rete-defn -- i've been wanting it for days
> without a good reason, i think its arrived..."* — and then, cutting my over-design down to its
> actual shape: *"we just need it create functions who bind to symbols and those functions are
> tighter in expressions than core-defn?"*
>
> **That is the whole thing.** Same registration, same symbol binding as `defn`. The only difference
> is that the BODY is checked tighter, AT THE DEFINITION SITE.

## The why, demonstrated — not argued

An ordinary `defn` used by a rule is rete-admissible **by accident of its current body**. Nobody
declared it, so there is no contract, so nothing can be *broken*, so nothing warns. Reproduced live:

```
BEFORE   helper `:ct::risky?` written with :wat::rete::core::i64::<   -> rule fires, 1 flag
EDIT     one word INSIDE THE HELPER:  :wat::rete::core::i64::<  ->  :wat::core::i64::<
--check  CLEAN. The edit looks completely fine.
RULE     "compile-condition: where expr is not a rete primitive"
FRAMES   compile-condition · compile-rule · compile · user::main
```

**Not one frame names `:ct::risky?`.** The person who made the edit is nowhere in the failure. With a
declaration, that same edit is refused **at the fn**, immediately.

## ★ THE FOUR QUESTIONS — ruled 2026-08-06

The fork posed was *accept-only* (the form exists; a `where` still admits any law-A-clean fn) vs
*require* (a `where` admits rete primitives + rete-defns, full stop).

**First, a category correction: accept-only is not a design alternative, it is a MIGRATION PHASE.**
The four questions judge an END STATE; judging a way-station as one is a category error.

### The end state — does the rete predicate language have a declared boundary?

| | |
|---|---|
| **Obvious?** | **YES** — a reader sees `(:wat::rete::where (:usr::big? ?n))` and knows `:usr::big?` must be declared, checked, bounded. One rule. And at the definition, the two forms are visibly different. |
| **Simple?** | **YES** — one admission rule at the door: *is it declared?* The transitive walk still exists but runs ONCE at a declaration instead of being chased per use. Today the door does two jobs (admit + infer); this splits them. |
| **Honest?** | **YES** — **law A** (#57: *a rete expression may contain ONLY `:wat::rete::` ops*) claims *"the entire rete query language may only be composed from rete primitives"* while enforcing it by chasing into undeclared code, which is exactly why **#82** and **#60** exist (walkers that miss). A declaration makes the claim structural: the language IS what is declared. And it states its own cost rather than hiding it. |
| **Good UX?** | **YES** — one extra decision at declaration time, and it is the decision that was already implicit and unenforced. The payoff is the error landing where you wrote it. Postgres `IMMUTABLE`/`STABLE` is the same ask, and nobody finds it onerous. |

**4 × YES.**

### Accept-only, judged as an end state

- **Obvious? NO** — two ways to spell the same capability, differing only in whether a guarantee
  exists, and the difference is **invisible at the use site** — which is the defect being fixed,
  half-fixed.
- **Simple? NO** — two admission paths into the rete language, maintained forever: declared, and
  inferred-by-walking.

Any NO disqualifies. **Accept-only is not the design.**

### And it is not needed as a phase either

A phase exists to make an expensive migration affordable. **This migration is mechanical.** Every fn
currently admitted into a `where` is *already* law-A clean — that is WHY it is admitted; `head_ok`
walked its body and found it so. Re-declaring is a **pure re-heading**, no body changes:

```clojure
(:wat::core::defn :usr::big? …)   ->   (:wat::rete::core::defn :usr::big? …)
```

**27 sites** (26 helpers called from `where` predicates + the 1 user aggregator). Codemod-able.
⇒ **Go straight to the membrane.** The phase buys nothing and costs a permanently-dual door.

## What the form IS

Everything `defn` does — parse the signature, register in `sym.functions`, bind the symbol — **plus**
the body checked at the definition against:

1. **Law A** — composes only rete primitives and other rete-defns
2. **Pure ∧ deterministic ∧ total** — the other three axes the fence already measures
3. *(later, #87)* **no recursion** and **the bound** — see NOT IN SCOPE

The four checks already exist as Rust walks in `purity.rs` (`is_pure_expr` / `is_deterministic_expr`
/ `is_total_expr` / `is_rete_primitive_expr`). **This stone runs them one phase earlier, at the
declaration, instead of at whatever rule happens to call it.** No new analysis is invented.

## The mechanism — one marker, one door

**`Function` (`src/value/environment.rs`) has no metadata field today** — `type_params`,
`param_types`, `ret_type`, `rest_param`, `rest_param_type`, `body`, `closed_env` and nothing else. So
the declaration needs somewhere to live. That is the entire mechanism cost beyond the form:

- a new field on `Function` recording that this fn was rete-declared *(and later, its bound)*
- `head_ok`'s `sym.functions` branch (`src/rete/purity.rs`) changes from **walk the body** to
  **consult the marker** — that is the membrane, and it is one branch

**This is NOT a `RETE_OPS` row, and that distinction is load-bearing.** `RETE_OPS` is *what may appear
inside a predicate*; `fn`/`let`/`match`/`cond` are EXPRESSIONS and belong there. `defn` is a
**top-level declaration** that never appears inside a `where`. Its `params`/`ret`/`meta` columns would
be meaningless. Do not pattern-match it into the #56 mirror family.

## ⛔ STOPs

- **STOP-1 — the body check must REUSE `purity.rs`'s walks, never re-implement them.** Four axes, one
  implementation, called from a second phase. A second copy is the stone's own law violated
  (*"a rete op is NEVER a second implementation"*) and the exact drift `[[feedback_an_adjacent_implementation_is_not_the_subject]]` names.
- **STOP-2 — a rete-defn stays callable from ORDINARY wat.** It is a fn carrying extra guarantees,
  like Postgres `IMMUTABLE` — still callable anywhere. Nothing is lost. If the design starts making
  it a separate callable namespace, that is scope creep.
- **STOP-3 — the fence on the INLINE `where` expression does not go away.** A predicate written
  directly in a rule is still checked in place; this stone only adds the declared-callee path.
- **STOP-4 — the migration is a RE-HEADING, and must be proven so.** If any of the 27 sites needs a
  BODY change, that fn was not actually law-A clean and the "already admitted ⇒ already clean"
  reasoning has a hole. Surface it; do not quietly edit a body.

## NOT IN SCOPE — deliberately

- **The bound** (`depth`/`nodes`/`fold_nesting`, #87). A declaration is its natural home — computed
  once at the definition instead of re-derived per call site — but it hangs on the marker AFTER the
  marker exists. Land the declaration, then hang the bound. Adjacent, then flip.
- **The no-recursion rule** (#86/#87). Same reasoning: it belongs at the declaration, and it lands
  with the bound it enables.

---

## ▶ THE NAMING TARGET — materialized for an intueri cast

*(R17 **self-prompt-injection** — the arc's own discipline for a design with no disk to ground
 against: WRITE THE CONCRETE ARTIFACT INTO THE SESSION, then judge THAT. The ward judges real forms,
 never a description of them.)*

**The slot:** a top-level form that declares a function whose body is restricted to the rete
vocabulary, checked at the definition site. It binds a symbol exactly as `defn` does.

```clojure
;; ── what it replaces at 27 sites ────────────────────────────────────────────
(:wat::core::defn :usr::big? [n <- :wat::core::i64] -> :wat::core::bool
  (:wat::rete::core::i64::> n 100))

;; ── CANDIDATE A — the family-consistent mirror
(:wat::rete::core::defn :usr::big? [n <- :wat::core::i64] -> :wat::core::bool
  (:wat::rete::core::i64::> n 100))

;; ── CANDIDATE B — no `core::`, grouping it with the top-level rule forms (where/defrule/make-rule)
;;    ⚠ MY ORIGINAL COMMENT HERE READ "since this is not a core mirror but a rete-only construct" —
;;    which CONTRADICTED this stone's own opening ruling ("Same registration, same symbol binding as
;;    `defn`") three paragraphs above. The cast caught it. B's justification denied the very mirror
;;    relationship the stone asserts; kept visible rather than silently rewritten.
(:wat::rete::defn :usr::big? [n <- :wat::core::i64] -> :wat::core::bool
  (:wat::rete::core::i64::> n 100))

;; ── CANDIDATE C — names what it produces rather than mirroring `defn`
(:wat::rete::defpredicate :usr::big? …)     ;; but it is NOT always a predicate —
                                            ;; a user AGGREGATOR is (PV<T>) -> R, not -> bool

;; ── CANDIDATE D — names the guarantee
(:wat::rete::defpure :usr::big? …)          ;; but purity is only ONE of the four axes it asserts
```

**Anchors already settled in this namespace, for consistency:** `:wat::rete::core::fn` ·
`:wat::rete::core::let` · `:wat::rete::core::match` · `:wat::rete::core::cond` (the #56 expression
mirrors) · `:wat::rete::acc::*` (the builtin accumulators) · `:wat::rete::where` ·
`:wat::rete::defrule` · `:wat::rete::make-rule`.

### ★★ RULED — `:wat::rete::core::defn` (CANDIDATE A). intueri cast 2026-08-06, weighed against the disk.

**And the tension as I first framed it was WRONG — I confounded two variables.** I wrote that the
`core::` segment marks a core mirror, but that *top-level* forms (`where`/`defrule`/`make-rule`) carry
no `core::` — presenting those as competing predictors. **They are not competing; they are
confounded**, and `cond` is the one example that separates them. Verified on the disk by my own read:

| form | top-level? | `core::`? | has a `:wat::core::X` twin? |
|---|---|---|---|
| `:wat::rete::core::cond` (`rete.wat:2327`, a `defmacro`) | **YES** | **YES** | **YES** |
| `:wat::rete::make-rule` (`:2364`, a `defn`) | YES | no | no |
| `:wat::rete::defrule` (`:2406`, a `defmacro`) | YES | no | no |
| `:wat::rete::query` (`:2248`) · `:wat::rete::where` | YES | no | no |

**`cond` is top-level AND carries `core::` — so "top-level" predicts nothing.** The single predictor
is *does it mirror a `:wat::core::X`*. The other four lack `core::` because they are rete inventions
with no core twin, not because they are top-level. `defn` mirrors `:wat::core::defn`, so it takes
`core::`, exactly as `cond` does.

**Two more reasons the cast grounded, both checked:**

- **`vocabulary.rs` documents ONE mechanical naming rule** (`rete_name` = `core_name` with `rete::`
  inserted after `wat::`), and its own module doc is a cautionary tale about that rule acquiring ad-hoc
  variants (*"three different rules at once… silently missed 17 of 57 rows"*). B would be the first
  deliberate hand-authored exception for a form with a clean, collision-free derivation.
- **No `NAMING_RULE_EXCEPTIONS` entry is needed.** That list exists only for **collisions** — several
  rows deriving one name (the `{enum,string,bool,keyword}::{=,not=}` and `first` groups, 11 entries,
  count-pinned). `:wat::core::defn` is shared by no other row, so the derivation is unique.
  ⚠ **And since this is NOT a `RETE_OPS` row** (see above), the naming-rule tests iterate `RETE_OPS`
  only and will neither check nor need extension for it — **the FQDN is a convention honored BY HAND**,
  exactly as `cond`'s is in its own `defmacro`. Say so in the brief; a convention no test enforces is
  the kind that rots.

**C and D rejected with reasons kept:** `defpredicate` is a **Level 1 lie** — a user aggregator is
`(PV<T>) -> R`, not `-> bool`, so it is false for half the form's uses. `defpure` mumbles — purity is
one of four axes, and not even the novel one; **law A is the axis this form actually adds**.
A fifth the cast considered and rejected: `:wat::rete::admit` — names the contract but drops
"this binds a symbol like `defn`", which is worse than B on Obvious.
