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
3. **no recursion** — **SHIPPED 2026-08-17** (`ReteDefnRecursive` at
   `apply_rete_defn_contracts`; probes `tests/rete/probe_arc278_rete_defn_recurse*`).
   The depth/nodes/`fold_nesting` *numbers* remain NOT IN SCOPE (builder sets them;
   do not derive from the corpus). See `CURRENT-STATE-annihilate-interpretation.md`.

The four checks already exist as Rust walks in `purity.rs` (`is_pure_expr` / `is_deterministic_expr`
/ `is_total_expr` / `is_rete_primitive_expr`). **This stone runs them one phase earlier, at the
declaration, instead of at whatever rule happens to call it.** No new analysis is invented.

## The mechanism — one marker, one door

> ⚠ **CORRECTED 2026-08-06 by grounding, before the strike.** The paragraph below originally read
> *"`Function` has no metadata field today … So the declaration needs somewhere to live"* — which is
> true of `Function` and **false of the substrate**. There IS already a per-binding metadata store:
> `SymbolTable.binding_metadata` (`src/value/symbol_table.rs:142`, type at `:16` —
> `HashMap<String, HashMap<String, WatAST>>`, FQDN → key → AST), populated at registration
> (`runtime.rs:818/929/1293`) and reachable from `head_ok` with **zero** new plumbing, since `head_ok`
> already takes `sym: &SymbolTable` (`purity.rs:961`). It would cost no cascade at all.
>
> **It is still the wrong home, and the reason is the ruling's real justification:
> `binding_metadata` is USER-WRITABLE.** `check.rs:4690` instructs users outright —
> *"use `:wat::core::defn` with metadata-map: `(defn :name {:restricted-to […]} [<args>] -> :<Ret> body)`"* —
> and `wat/spawn.wat:303` is a live corpus example. So a marker stored there could be **forged**:
> `(:wat::core::defn :usr::f {:rete-declared true} …)` would set the key with the rete body check
> never running. That is a lie with a form — R28 `SOLVIMVS NE MENTIRETVR` violated exactly, and the
> same shape as the `None`-means-skip defect R66 just killed.
>
> **A typed field cannot be forged and cannot be misspelled.** The ruling stands; it now stands on a
> disconfirmed alternative rather than on an absence nobody checked.

**`Function` (`src/value/environment.rs:35`) has exactly nine fields** — `name`, `params`,
`type_params`, `param_types`, `ret_type`, `rest_param`, `rest_param_type`, `body`, `closed_env` —
none of them metadata. So the declaration needs a typed home. That is the entire mechanism cost
beyond the form:

- a new field on `Function` recording that this fn was rete-declared *(and later, its bound)*
- `head_ok`'s `sym.functions` branch (`src/rete/purity.rs`) changes from **walk the body** to
  **consult the marker** — that is the membrane, and it is one branch

**This is NOT a `RETE_OPS` row, and that distinction is load-bearing.** `RETE_OPS` is *what may appear
inside a predicate*; `fn`/`let`/`match`/`cond` are EXPRESSIONS and belong there. `defn` is a
**top-level declaration** that never appears inside a `where`. Its `params`/`ret`/`meta` columns would
be meaningless. Do not pattern-match it into the #56 mirror family.

## ★★ WHAT THE FIRST STRIKE LEARNED — 2026-08-06/07, all four proven by a run

The first attempt built the form, the marker and the membrane, and **the membrane never fired at
runtime.** Four corrections, kept here because every one of them is a place the next hand would
land in the same hole. The design is unchanged; the MECHANISM is where it was wrong.

### 1. The membrane lives in `classify_fn`, and it is SCOPED TO LAW A

Not at `head_ok`'s `sym.functions` branch (`purity.rs:997`) — that branch RETURNS `classify_fn`,
whose `FunctionBody::Native` arm is what admits the native HOF combinators (`foldl`/`map`/…) via
`intrinsic_meta`. Patching there denies all of them.

And inside `classify_fn`'s `Wat` arm, the refusal must fire **only on `Axis::RetePrimitive`**:

```rust
if func.rete.is_some()            { Ok(()) }                       // declared: all four proven
else if matches!(axis, Axis::RetePrimitive) { Err(...) }           // undeclared: law A refuses
else { classify_expr(body_ast.as_ref(), axis, sym, seen) }         // the other three: unchanged
```

`Some(_)` may be admitted on any axis — all four WERE proven at the declaration. `None` means only
*undeclared*, which says nothing about purity. Denying all four made `:wat::rete::pure?` — a
GENERAL predicate over any expression — answer **false for an ordinary, genuinely pure fn**, and
broke nine tests in unrelated files. `Axis::RetePrimitive`'s own doc names the rule: it exists as a
separate variant because reusing Pure/Deterministic/Total "would make the refusal LIE."

### 2. Seed `seen` with the WHOLE declared set, not just the name being checked

`find_axis_violation` starts from an empty `HashSet`, so a self-call reaches `classify_fn` while the
fn is still `rete: None` and is denied **for calling itself** (`:head` = its own fqdn). Worse, the
check iterates `declared: &HashSet<String>` in **arbitrary order**, so a MUTUAL reference
(`where-nesting`'s `c1`/`c2`) passed or failed by hash order — a check that answers differently per
run is not a check.

Seeding the whole group is PARITY, not leniency: `classify_fn`'s own guard is already
`if seen.contains(fqdn) { Ok(()) } // back-edge`. Every member is proven independently against its
own body; only the ORDER of proving stops mattering. Bounding recursion is #87's ruling — #88 must
not forbid by accident what the stone deferred on purpose.

### 3. ★ THE ONE THAT MATTERS — THE CHECK BELONGS TO REGISTRATION, NOT TO FREEZE

The first cut ran the check in `build_env` (step 6.975) and stamped `Function.rete` there. But
`FrozenWorld::freeze` calls **`register_runtime_defs` at `freeze.rs:564`, AFTER `build_env`
returns**, and that pass re-registers every `defn` — **dropping the stamp.** So the file LOADS (the
check ran, and correctly refuses a bad body) while the runtime fence still refuses every helper,
because the `Function` it reads is a fresh, unstamped one. Every re-headed site behaved identically
for this reason; the whole corpus waterfall was chasing this one symptom.

**`register_runtime_defs` is the ONE DOOR.** Both paths call it — `freeze.rs:564` (boot) and
`runtime.rs:24475` (a live session). Putting the check there covers both, and the `build_env` call
is **deleted**, not kept alongside: two checks would be two implementations of one law.

### 4. The corpus codemod must key on (FILE, NAME), never NAME alone

`:test::big?` is declared in TWO unrelated fixtures with DIFFERENT bodies — one rete-clean, one
core-spelled. A name-keyed rewrite re-headed both and broke the second. Only files the CHECKER
named may move. And law A is transitive, so re-heading a helper makes its undeclared callees scream
in the next round — `where-nesting`'s `c1..c10` chain moves as a unit. Expect a waterfall; let the
checker name each round.

## ★ THE DEPLOYMENT MODEL — the builder's ruling, 2026-08-07, and it settles the failure shape

> *"i fully intend to build a defservice who accepts user-forms, compiles them into a rete jump
> table and, if that passes, then allow the users to provide facts and then fire upon those facts…
> the forms to be compiled are not from the machine that does the compilation… they will come over
> a wire… we must never assume they validated their input… just like mysql or whatever else."*

Rule compilation is **runtime-only, from a foreign host**. Process-local today; the wire is the
deployment model, and the shape must not need a redesign to get there.

**Therefore a refused declaration is a RESPONSE, never a raise.** MySQL answers *"your query isn't
shaped correctly"* and keeps serving. A raise crossing that boundary kills the service for every
other client — which this arc already paid for: `28701476`→`b9d61bd6`, a wrong-typed body under a
correct tag took a service down for all clients, cured by a named `RequestMalformed` reply.

**The shape is settled substrate convention — ONE GOOD RESULT, N BAD** (the builder: *"we are full
ADT so an enum to match on is the only viable path"*). The nearest precedent is the op that already
takes user rules:

```
SiftRulesResponse   Deductions        ← the one good result
                    Fatal             ← 500-class: we broke
                    RequestTooLarge   ← 400-class: carries bytes + cap
                    RequestMalformed  ← 400-class: carries path + expected + got
QueryLogsResponse   Success | RequestTooLarge | RequestMalformed
```

Each failure variant carries **located structured fields**, never prose standing in for structure.
A rules-compilation refusal is the 400-class sibling of `RequestMalformed` — a missing VARIANT in
that family, not a new error channel. Task **#17** ("a variant per failure kind, 400 vs 500,
exhaustive") owns the contract; this is a row in it.

`ReteDefnAxisViolation { name, axis, head }` + its span already carries exactly that payload, proven
by the gate. It moves from raise-at-freeze to value-at-registration.

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

- **The numeric bound** (`depth`/`nodes`/`fold_nesting`). A declaration is its natural home —
  computed once at the definition instead of re-derived per call site — but the *numbers* are
  the builder's. Do not take them from the corpus. Recursion (the back-edge) is no longer
  in this list: it shipped 2026-08-17 as a load refusal, without waiting on the numbers.

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
