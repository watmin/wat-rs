# DESIGN — STONE 251.8: implement SYMBOL proper

**Status: DRAWN 2026-08-13. Stone 251.8a is strike-ready; 251.8b–d are the campaign shape, not yet briefed.**

The builder's ruling, 2026-08-13:

> *"i think we need to implement symbol proper... i think that's been missing since the
> beginning as we've been using 'colon-quoted symbols'...."*
>
> *"a symbol is either namespaced or it isn't... non-namespaced symbols are only legal in
> binder positions — all other symbols must be namespaced.... this is already enforced
> everywhere.. but the rust side needs a firm enforcement....."*
>
> *"we could maybe have an explicit-hidden 'binder' namespace to make them all fit the same
> underlying shape... a symbol with a 'binder' namespace is legal and nothing in the user
> forms change... we can query its namespace and get back something like $binder... we could
> reserve that namespace internally like we do with wat.\* and rust.\* now.... yea?"*

---

## WHY — the defect, stated at its own definition site

`crates/wat-reader/src/ast.rs:99-107`, verbatim:

> *"Keyword token, as in `:foo`, `:wat::holon::Atom` … Used both as keyword literals (payloads
> for wat keyword atoms) **AND** as keyword-path references (heads of calls, type annotations).
> **Distinguished by context at later passes.**"*

One node, two unrelated jobs — a **value** and a **reference** — with the node itself unable to
say which. Every downstream pass re-decides, independently, and they do not agree. This is R28's
fusion (`SOLVIMVS NE MENTIRETVR`), un-decomplected, at the AST.

**The cost, measured 2026-08-13** (probes in `$CLAUDE_JOB_DIR`, results reproduced in task #95):
a call head spelled as a dotted symbol is invisible to the type checker — args, arity and return
are all unchecked, because `infer_list` (`src/check.rs:2518`) gates its **entire** call-inference
universe on `if let WatAST::Keyword(k, head_span) = head` at `:2542`, closing at `:5568`. Past
that only `Some`/`Ok`/`Err` bare-symbol cases survive; a namespaced `Symbol` head falls to a fresh
type var, which unifies with anything.

```
d3  (user/f "boom")   where f is [n :- wat.type/i64] :- wat.type/i64   exit 0 — RAN, printed "boom"
d4  (:user::f "boom") control                                          exit 3 — COMPILE TypeMismatch
d5  (wat.core/+ 1 2 3 4 5)                                             exit 0 — printed 15
```

**This is arc 251's real blocker.** Flipping the corpus to the dotted surface would not merely
weaken return-type enforcement — it would switch off the type checker at every call site, in one
commit, with the floor staying green because the runtime still dispatches correctly.
R59 `NISI FRANGAS NIHIL PROBAS` at the scale of the language.

---

## ★ THE DISCRIMINATOR — proven by run, and it overturns the AST's own claim

The AST says the two roles are separable only "by context." **They are not: they already have two
distinct spellings.** Durable probe, green, committed beside this design:
`wat-scripts/scratch-pad/probe-251-keyword-vs-colon-quoted-symbol.wat`

```
:foo             →  :foo                          keyword, VALUE
:my.app/status   →  :my.app/status                keyword, VALUE
:wat.core/+      →  :wat.core/+                   keyword, VALUE   ← spelled to collide with a real
                                                                     definition; does NOT resolve
:wat::core::+    →  #wat-edn.opaque/clauses nil   REFERENCE        ← resolves to the fn, in VALUE position
```

Clojure 1.12.4 agrees on the classification: `(type :wat.core/+)` and `(type :foo)` are both
`clojure.lang.Keyword`; `(type 'wat.core/+)` is `clojure.lang.Symbol`.

> **A `::` inside a keyword token means it was never a keyword.**

Mechanically decidable at the reader. That is what makes the correction **enumerable** rather
than a survey — and it is why the sentence in `ast.rs` is not a description of the language but a
description of the implementation's confusion.

**Consequence for the arc:** 251 is not "flip 1392 files to a new syntax." It is *make symbols
real; the dotted surface is what falls out.* The flip becomes a **reader/printer** concern — two
spellings of one node — instead of a semantic migration. Which is why the flip kept feeling like
it stood on nothing.

---

## THE SHAPE RULED

```
every symbol is (namespace, name, scopes) — a namespace is NEVER absent

  $bound/x       a LOCAL binder    — reserved namespace, un-spellable by user source
  wat.core/+     a reference       — the substrate's
  user/f         a reference       — the program's
  :foo           a KEYWORD         — and only ever a value
  :my.app/status a KEYWORD         — namespaced, still only ever a value
```

Nothing in user forms changes. `(let [x 1] … x)` still reads `x`; the reader gives it the binder
namespace. "Is this a reference?" becomes a namespace comparison instead of the four hand-rolled
string tests that decide it today, each independently:

```
src/resolve/normalize.rs:81   ident.as_str().contains('/')
src/macros/expand.rs:537      ident.as_str().contains('/')
src/runtime.rs:3048           s.as_str().contains('/')
src/runtime.rs:3307           s.as_str().contains('/')
```

That is #75's ONE DOOR class, and `contains('/')` is arc 278's own recurring bug species — *a
string comparison with one side normalized and the other not*, three instances in that arc alone.

### Why a NAMED binder namespace and not `Option<Namespace>`

Cut on precedent one day old (arc 278 R66): an `Option` whose `None` meant *"skip the check"*
collapsed two unrelated situations and passed **both** — the builder's own *"'none means skip'
feels like a catastrophic bug."* A named namespace makes the binder case a **match arm that
cannot be forgotten**; `None` is what gets `unwrap_or_default`'d. Uniform shape beats optional
field. R65 `SCVTVM IDEM INDEX` — the shield is the ledger.

### The vocabulary (intueri-cast, `93971169`; target at `wat-scripts/intueri/symbol-namespace-vocabulary.wat.intueri`)

| slot | name | what decided it |
|---|---|---|
| binder namespace | **`$bound`** | the namespace is read at USE sites; a use of `x` is a *bound occurrence*, not a binder. Clojure-legal (`(ns $bound)` verified live against 1.12.4). Measured free: 0 corpus hits. |
| accessor | **`namespace`** | Clojure-faithful (`(namespace 'wat.core/+)`). `ast-namespace` loses because `ast-name` is PARTIAL — joining that family imports a false partiality onto a verb total by construction. |
| predicate | **`reference?`** | polarity settled by the four sites it replaces: all four test for reference-ness and act on `true`, so `binder?`/`bound?` would force a negation at every one. |
| diagnostic term | **`colon-quoted symbol`** | the builder's own phrase; `keyword-form symbol` re-teaches the fusion in the sentence meant to dismantle it. |

`WatAST::Keyword` **keeps its name** — the fusion was never in the name (Clojure's `Keyword` is
value-only too). Its **doc comment** is the defect and gets cut to the value role when the
correction lands; left standing it becomes a stale comment actively re-teaching a bug that no
longer exists.

Verified against the disk before drawing: `:wat::core::namespace`, `reference?` and `bound?` are
all unclaimed; `is_reserved_prefix` (`src/resolve/reserved.rs:34`) strips a leading `:` then
`starts_with`, so the reservation entry is the literal **`":$bound::"`** in doubled-colon shape —
a concrete edit, not passive absorption.

---

## ★ THE ONE CONTRACT DECISION, PINNED

> **`Identifier` gains a `namespace` that is NEVER absent. A binder carries the reserved `$bound`
> namespace; `Identifier::bare` becomes the constructor for a `$bound` symbol and no other. There
> is no un-namespaced `Identifier`, no `Option<Namespace>`, and no second constructor that admits
> one.**

Everything else in this design follows from that sentence. If it is not held, the campaign
degrades to the `contains('/')` status quo with extra ceremony.

---

## THE CAMPAIGN — four stones, only the first briefed

**251.8a — THE VOCABULARY, AT ZERO OFFENDERS (this stone; strike-ready).**
Additive and green throughout. `$bound` is reserved; the `namespace` accessor and `reference?`
predicate ship as the ONE DOOR; the four `contains('/')` hand-rolls collapse onto it. **No
reference moves. No node changes role.** The corpus does not notice. This is #41's proven
pattern — *turn the wall on at zero offenders* — and it installs the vocabulary every later stone
spends.

> ⚠ **8a DOES NOT YET HOLD THE PINNED CONTRACT, and says so rather than implying it.** In 8a the
> namespace is **DERIVED** from the spelling by one implementation, not **STORED** as a field.
> Grounded reason: `Identifier::bare` (`crates/wat-reader/src/identifier.rs:94`) is the sole
> constructor and `as_str()` (`:126`) returns the full spelling to a large caller set — splitting
> the stored name into `(namespace, name)` moves every one of them. That cascade belongs in
> **8b**, where the normalizer inverts and those callers are being touched anyway.
>
> So 8a's honest claim is narrow: *four hand-rolled string tests become one door with a name.*
> The door's **signature is the contract** (`namespace()` is total and never returns an absence),
> so 8b swaps derived for stored behind it without moving a caller. Claiming 8a implements
> symbols would be the overclaim this arc keeps catching.

**251.8a-ii — THE BINDER NAMESPACE IS UNFORGEABLE. RULED 2026-08-13. Its own strike.**

The builder:

> *"i think my stance is no one but rust's handling of binders can declare `$bound/*` symbols..."*

**The defect this closes, measured on the post-8a binary** (task #99): `$bound/x` written in user
source is accepted, treated as a binder, and dies at RUNTIME as `UnboundSymbol`. Before 8a it was a
freeze-time `UnresolvedReference`. Meanwhile `(def :$bound::x 1)` IS refused at freeze with
`ReservedPrefix`. So the reservation bites at the *definition* door and not the *reference* door,
and 8a moved a freeze error to runtime for the one namespace it invents.

**FOUR QUESTIONS, flat, on every option — and the shared premise checked first.**

⚠ **B and C rest on a premise that expires.** Both assume the binder namespace is part of the NAME
STRING. True in 8a (`namespace()` is derived from spelling); **false after 8b** (stored field). They
are artifacts of a temporary implementation, not durable options.

| option | Obvious | Simple | Honest | Good UX | |
|---|---|---|---|---|---|
| **A** — refuse at freeze (resolve layer) | YES | YES | YES | YES | 4/4 |
| **D** — refuse at the READER; the namespace is never constructed | YES | YES | YES | YES | **4/4 — CHOSEN** |
| **B** — treat as a binder *(what 8a ships)* | **NO** | YES | **NO** | **NO** | disqualified |
| **C** — normalize, let the reserved-prefix gate refuse it *(pre-8a)* | YES | **NO** | **NO** | — | disqualified |

**B fails Obvious** — `$bound/x` silently means `x`; the runtime error names a symbol the user
believed they had qualified. **Fails Honest** — user source can forge what must be substrate-only.
**C fails Simple on BRAIDING, not size** — it routes a *binder* namespace through the *keyword
reference* pipeline so the refusal falls out as a side effect of pretending it is a reference; and
**fails Honest** because the diagnostic tells the story of someone who tried to DEFINE into a
reserved prefix, when they used it in REFERENCE position. (C is what shipped before 8a: 8a did not
regress from a good state, it moved off a differently-broken one.)

**★ WHY D OVER A, and it is the ladder.** A is a CHECK — every pass between the reader and that
check can still be handed a forged binder. D is NO-FORM — the namespace is never constructed, so no
downstream pass can hold one. Same reason `$bound` is a named namespace rather than an `Option`.

**And D lands at its FINAL ADDRESS today.** 8b changes what `Identifier` STORES; it does not change
where user text is first read. The rule goes in once and never moves — whereas A would put a
refusal in the resolve layer that 8b relocates. This kills the "land it now, relocate later" hedge:
there is nothing to relocate.

**Scope, stated because the orchestrator got this wrong once and it must not survive into the
brief:** the rule is about the **namespace `$bound`**, NOT about the character `$`. `$` is an
ordinary identifier character (`is_symbol_break`, `lexer.rs:519`, does not list it) and is in live
use as `:<name>$impl`, a macro-minted NAME SUFFIX inside a keyword — never a namespace. `$impl` is
untouched by this stone.

### ⚠ WHAT 8a-ii's FIRST CONSUMER FOUND ABOUT 8a's DOOR — 8b MUST KNOW THIS

Striking 8a-ii surfaced a limitation of the 8a door that 8a's own design did not anticipate, and it
was found the way these always are — by the first real consumer (`PRIMVS VSVS ANGVLOS PANDIT`).

**`namespace()` collapses two distinct cases to the same string:**

```
bare `x`          no slash        → "$bound"   (the DEFAULT arm)
explicit `$bound/x`   slash present, segment == BOUND_NAMESPACE
                                  → "$bound"   (the SEGMENT arm)
```

So the door **cannot express "was this namespace explicitly written?"** A guard of
`namespace() == BOUND_NAMESPACE` would have refused **every bare binder in the corpus**. The rider
verified both return the same value by hand, and Expectations row 3 (the positive control) is what
would have caught it had the guard shipped that way — which is precisely the row that exists because
the orchestrator had already confused `$bound` with `$` once.

This is why 8a-ii's guard is a **literal-spelling check** (`strip_prefix(BOUND_NAMESPACE)` +
`starts_with('/')`), not a use of the door. That is NOT a fifth hand-rolled classifier and does not
trip STOP-1: the four sites 8a collapsed all answer *"is this a reference?"* — a general question,
reused everywhere; this answers *"is this literally the one reserved spelling?"* — one constant, one
site, never reused.

**★ AND THE COLLAPSE IS HARMLESS AFTER 8a-ii, WHICH VINDICATES CHOOSING D OVER A.** Once the reader
refuses `$bound/…`, the only way a symbol can carry the `$bound` namespace is that the substrate
minted it. The distinction `namespace()` cannot express becomes a distinction that **cannot arise**.
Option A (refuse at freeze) would have left the forgeable spelling constructible and the door's
lossiness live in every pass between the reader and the check. D did not merely close a hole — it
made the accessor's ambiguity unreachable.

**FOR 8b:** when the namespace becomes STORED rather than derived, keep this property. A stored
namespace must be set by the substrate for binders and be unforgeable from source; if 8b ever
reintroduces a path where user text can produce the `$bound` namespace, the collapse becomes a live
ambiguity again and this note is the reason it matters.

**251.8a-bis — THE ANGLE-BRACKET RETIREMENT. RULED 2026-08-13; PREREQUISITE OF 8b.**

The builder:

> *"i want essentially zero tolerance for `HashMap<K,V>` .... it must be `(HashMap [K V])` .... we
> must support the angle brackets mid migration... but post migration these are completely
> illegal... you may never declare a parametric type using angle brackets going forward....."*

So: dual-read during, **hard illegal after** — the four-move de-prime pattern this project has
already proven end to end (add the form → migrate callers → delete the old → wall at zero
offenders, #41's shape).

**★ WHY IT IS A PREREQUISITE OF 8b AND NOT A PARALLEL TRACK — measured against Clojure's own EDN
reader, 2026-08-13:**

```
:wat::core::HashMap<wat::core::keyword,wat::core::string>
  → REFUSED  "Invalid token: :wat::core::HashMap<wat::core::keyword"   ← the read stopped at the COMMA
(:wat::core::defn :f [m <- …] -> :wat::core::nil nil)
  → REFUSED  "Invalid token: :wat::core::defn"                          ← every `::` keyword is non-EDN
(wat.type/HashMap [wat.type/keyword wat.type/string] :first "foo")  → OK, arity 6
(wat.type/Vector [wat.type/i64])                                     → OK, arity 2
(wat.core/defn some.ns/incr [n :- wat.type/i64] :- wat.type/i64 …)   → OK, arity 6
```

The builder's claim — *the language must become Clojure-compliant EDN; it is not now* — measured
and confirmed. But the **ordering** argument is sharper than non-compliance, and it is about what
happens AFTER 8b turns `::` references into symbols:

```
HashMap<K,V>       as an EDN symbol → reads as   HashMap<K          ← SILENTLY TRUNCATED
(f HashMap<K,V>)   as EDN           → reads as   (f HashMap<K V>)   ARITY 3, was 2. NO ERROR.
Vector<i64>        as an EDN symbol → reads as   Vector<i64>        arity intact
```

As a **keyword**, `HashMap<K,V>` survives only because `lex_keyword` (`src/lexer.rs:605`) hand-rolls
a bracket-balancer EDN does not have. As a **symbol** it is accepted and **wrong** — the comma is
EDN whitespace, so the form silently changes arity instead of failing. `<` and `>` are legal
Clojure symbol characters (`->`, `>=`), so the comma is the *only* fatal one.

**Measured population:** 3543 angle sites across 599 files; **965 of them carry a comma.** Those 965
are the ones that would stop erroring and start lying. Top shapes: `<K,V>` 134, `<wat::core::i64,wat::core::i64>`
121, `<S,R>` 102, `<probe::Echo::Op,probe::Echo::Reply>` 59.

⚠ **The exact coupling depends on 8b's scope, which this design drew loosely.** If 8b converts
references in TYPE-ANNOTATION positions (not only call heads), the 965 become mis-aritied symbols
and this stone is a hard prerequisite. If 8b is call-heads-only, they remain keywords and the
reckoning lands at 8d. **Either way they must die before the corpus is EDN** — the ruling is not
conditional, only its position is. Pin 8b's scope before sequencing.

**⊘ SUPERSEDED — `109/NOTE-generic-bracket-syntax-edn.md`.** That note (2026-06-04) proposes
**pipe-separated** `<First|Second|Third>` to keep a parametric type atomic to Clojure's reader, and
flags itself *"a POINTER, not a decision"* with no four-questions verdict locked. It is overtaken by
measurement, not by preference: pipe keeps the type one *token*; the ruled form abandons tokenhood
and makes it a *list*, which is natively EDN and arity-stable. The note's own stated goal is better
served by the option it did not consider. Do not implement the pipe.

**251.8b — INVERT THE NORMALIZER.** Today `resolve/normalize.rs` rewrites `Symbol("wat.core/+")`
→ `Keyword(":wat::core::+")` so that, in its own words, *"the UNTOUCHED downstream dispatch
resolves it."* Under the correction that is backwards: the `::` keyword is the one that should
become a Symbol. Inverting it is what lights the corpus, and 251.8a's door is what makes the fire
readable rather than 647 undifferentiated sites.

**251.8c — CLOSE THE CHECK HOLE.** With references arriving as Symbols, `infer_list`'s
Keyword-gated dispatch must route them. This is the stone that actually fixes #95; it cannot be
done honestly before 8b, because patching the Keyword gate alone would treat the symptom while
leaving `Symbol` un-implemented — the *check* rung where the ladder offers *no-form*.

**251.8d — THE READER/PRINTER FLIP.** `::` retires as a reference spelling; a colon means keyword,
full stop. The 1392-file corpus flip lands here, as a spelling change over a substrate that
already type-checks both.

---

## OUT OF SCOPE — affirmatively cut, not deferred

- **The dotted-head type hole (#95) is NOT fixed by 251.8a.** It is 251.8c. Closing it earlier
  would mean patching `infer_list`'s Keyword gate to also accept Symbols — the *check* rung, on a
  substrate where the *no-form* rung is reachable. Cut.
- **The corpus flip is NOT in this stone.** It is 251.8d, and it must not begin while a dotted
  call head is unchecked.
- **`ast-name`'s partiality is NOT in this stone.** It is a real defect (it raised on 43 of 1392
  files and killed the old codemod) and the intueri cast surfaced it, but it is `ast-name`'s
  problem, not the namespace accessor's. Tracked separately.
- **The opaque-clause-table leak is NOT in this stone.** `(println :wat::core::+)` printing
  `#wat-edn.opaque/clauses nil` is a genuine finding from this session's probe — a function's
  internal clause table reaching user-visible output. Its own defect, filed, not smuggled in here.

---

## STOP TRIGGERS (rejection criteria — ship nothing, surface the gap)

**STOP-1 — the scope cascade.** If adding the namespace to `Identifier` cannot be done without
threading a new parameter through call sites beyond the ones named in the brief, STOP and report
the count. The 24m/PRIMVS-VSVS lesson is explicit: *never thread a substrate flag as a new param
through the world — put it on the reference already threaded and set it at the boundary.* A
constructor-signature change is the honest shape; a threaded parameter is not.

**STOP-2 — hygiene collision.** `Identifier` already carries `scopes: BTreeSet<ScopeId>` for macro
hygiene. If the namespace turns out to interact with scope remapping — in particular if
`Identifier::bare`'s debug assert (which exists precisely to forbid a scope baked into a name)
fires, or if `hash_canonical_program` changes — STOP. Those are load-bearing for the execve boot
wire (arc 170 `NON EXEMPLAR, SED ORTVS`), and a change there is a different stone.

**STOP-3 — the four sites do not agree.** If collapsing the four `contains('/')` tests onto
`reference?` changes the behaviour of ANY of them, STOP and report which and how. They are
supposed to be four spellings of one question; if they are not, that divergence is a finding
larger than this stone.

---

## THE GATE

251.8a is **observationally inert by construction** — no reference moves, no node changes role, no
user form changes. So the gate is a floor that has not moved plus a wall that is real:

1. `cargo nextest run --release` — Summary line read by hand, weighed against the floor at strike
   time. Zero new failures. A *changed* count in either direction is a finding, not a pass.
2. `cargo clippy --release --all-targets` — zero warnings (`-D warnings` is armed).
3. The four `contains('/')` sites are **gone**, replaced by `reference?`. Grep-verified at zero.
4. A RED probe proving the door is not vacuous: a symbol constructed as a binder answers
   `$bound` from `namespace`, and `reference?` is false for it and true for `wat.core/+` — and
   the probe must be **mutation-tested** (break the predicate, watch the gate go red) before it
   is believed. R59: a pass is a claim; only a break earns one.
5. `wat-scripts/scratch-pad/probe-251-keyword-vs-colon-quoted-symbol.wat` still exits 0 and prints
   the same three values.

---

## PROVENANCE

Every measurement in this design was run by the orchestrator this session, exit codes taken
without a pipe. The intueri verdict was **weighed against the disk**, not relayed: the accessor
and predicate names were confirmed unclaimed, `is_reserved_prefix`'s shape was read, and the
ward's correction to the reservation mechanics was checked and adopted. The ward also caught a
bias in the naming target — the Clojure REPL transcript for `$bound` was front-loaded above the
candidate list — which is recorded here rather than quietly fixed.
