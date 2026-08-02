# DESIGN-STONE — the rete expression language

> **Status: DESIGNED 2026-08-02. S0 RULED by the builder — law A, namespace-based.** Supersedes
> T2/T3 of `DESIGN-STONE-total-the-third-axis.md` (T1 — the axis, unarmed — is LANDED at `f37c54f3`).
> Downstream beneficiary: `DESIGN-STONE-compiled-where.md`.
>
> Two drafts preceded this one. The first scoped the work to *arithmetic only* — too small (see
> **§ Why the corpus is the wrong instrument**). The second left the law as an open fork; that fork is
> now closed (see **§ The fork, and how it was ruled**).

## The thing being built

A **closed expression language for rete conditions** — the vocabulary legal inside a `where` (and the
accumulator fence). Pure, total, deterministic, and closed enough that a compiler for it is finite.

Builder's framing, the design's spine:

> *"think of sql… the ops there are bespoke to sql… in wat everything is edn so our ops LOOK similar
> to regular code… but we can impose exact guardrails on our where clauses to make compilation
> trivial… Clara chose to use regular clojure forms… that means perf takes a hit and there's
> impurities allowed… we can impose a dsl that requires all forms are compilable."*

## ★ THE LOAD-BEARING PROPERTY — the closed set is a BASIS, not a ceiling

The vocabulary is the **alphabet**, not the expressivity budget. Users compose arbitrarily complex
predicates as ordinary `defn`s over it, and each one is admissible *because* it bottoms out in the
closed set.

Builder: *"users can compose as complex of funcs as they want with this limited, but expressive, set
of tools."*

**And the mechanism is already built.** `head_ok` (`purity.rs:545`) consults `sym.functions` **before**
it ever reaches the intrinsic table, handing off to `classify_fn` (`:725`), which walks the fn body on
the same axis with cycle detection (`seen`, back-edge ⇒ `Ok`). A user fn composed of admissible heads
is admissible, transitively, at any depth. Nothing to build — it shipped with stone 6a.

This corrects the second draft, which wrote *"call it 40–60 names"* as though that bounded what a user
could say. It bounds the alphabet. It is SQL's actual shape: a closed operator set, and views on views
on views.

## The fork, and how it was ruled

The open question was whether the law is **namespace-based** (a `where` admits only `:wat::rete::` ops)
or **property-based** (pure ∧ total, any namespace).

**RULED: namespace.** Compilability is the whole argument for a closed set, and only the namespace law
delivers it — property-based leaves the head-space open, so the compiler still faces arbitrary wat and
the admitted set is *whatever happens to have been classified*. Property-based is today's fence with a
third conjunct; it buys totality and nothing else.

Recorded rather than deleted, so it is not re-derived: the property law was a real option, it was
weighed, and it lost on compilability alone.

## What the law actually says — three doors, not one

A naive reading ("the head must be `:wat::rete::`") is **wrong twice**, and both errors are load-bearing.

```
head_ok(head, axis):
  constructor_meta  → declaration-derived   (a record's fields exist by construction of the type)
  accessor_meta     → declaration-derived   (`Client/rep` is total, and will never be rete-namespaced)
  sym.functions     → classify_fn, RECURSE  (the composition door — unchanged)
  else              → is the head in the rete VOCABULARY?
```

**Error 1 — it would kill composition.** `Client/rep` and a user's `:user::risk-score` are not
rete-namespaced and never will be. The accessor/constructor doors stay because *declaration-derived*
is a stronger warrant than a namespace: a field declared on an aggregate exists on every instance by
construction (`purity.rs:509`). The `sym.functions` door stays because it *is* the basis property above.

**Error 2 — `:wat::rete::` is already the engine's own API.** Grounded: `fire-rules`, `insert`,
`compile`, `compile-condition`, `Session`, `AlphaNode`, `activate-fact`, `collect-derived`,
`accumulate-pass` — dozens of names. A bare `starts_with(":wat::rete::")` **admits `fire-rules` inside
a `where`.**

The builder's own sketch already avoids this: every example carries a module segment.

```clojure
:wat::rete::core::if          ; wat.rete.core/if      (post-300 spelling)
:wat::rete::i64::+            ; wat.rete.i64/+
:wat::rete::f64::/            ; wat.rete.f64//
:wat::rete::string::subs      ; wat.rete.string/subs
:wat::rete::holon::cosine     ; wat.rete.holon/cosine
```

**So the admission test is the MODULE SET, not the root prefix** — a small closed list of modules
(`core`, `i64`, `f64`, `string`, `holon`, …) that grows by demand, replacing a 110-verb hand-list. Bare
`:wat::rete::<name>` (no module) is the engine API and stays inadmissible.

### Why the namespace is a certificate and not a convention

`:wat::rete::` sits under `:wat::`, a **reserved prefix** — a user cannot mint there
(`resolve/reserved.rs`). The namespacing wall armed at `b18888f8` is what makes the cheap admission
test sound. Minting-time discipline (nothing enters a rete module unless pure ∧ total ∧ deterministic)
is what the fence-time test is allowed to assume.

## Forms ARE ops here — and they split into two classes

The second draft argued *"forms are not ops; you namespace functions, not syntax."* **That was wrong on
the mechanism.** Grounded:

- **`if` / `let` / `do` / `when` have no structural arm.** They go through `head_ok` and are admitted by
  the plain `matches!` list at `purity.rs:246-249`, exactly like `+`. Under law A they are refused
  unless mirrored. The only alternative is a fence reading *"rete-namespaced **or** one of these four
  core forms"* — a second rule, an exception list, precisely the seam a closed set exists to delete.
  **⇒ `:wat::rete::core::{if,let,do,when}` must exist.**
- **`cond` / `match` / `fn` (and `quote`/`quasiquote`/`holon::literal`) DO have structural arms**
  (`purity.rs:602`, `:608`, `:628`, `:658`), matched against a literal keyword string, and never reach
  `head_ok`. Mirroring these is **widening a match guard**, not registering an op. Small, but a
  different edit, and a rider must not conflate them.

## ★ THE IMPLEMENTATION LAW — shared kernel, two surfaces

Builder: *"the native solutions are using shared tooling to keep impl consistent and coherent… the
surface is where the names disambiguate what you are using for what reason."*

A rete op is **never a second implementation.** It is a second *terminal handler* over the same routine.
The FQDN at the call site is the only place the two contracts differ:

- `:wat::core::i64::+` — *overflow aborts the fire.*
- `:wat::rete::i64::+` — *overflow yields the `:undefined` operand I declared.*

**The factoring already partly exists.** `runtime.rs:9753`:

```rust
":wat::core::i64::+" => Some(arith_i64_i64_inner(impl_name, vals,
    |a, b| a.checked_add(b).ok_or(I64ArithErr::Overflow(a, b)))),
```

The partiality is already a typed domain fact (`I64ArithErr`), and the raise is one handler over it
(`:9871`). The rete surface substitutes the fallback at that same site.

**This dissolves the two-spellings-drift cost** rather than mitigating it: there is no second
implementation to drift from. (It is *not* the `()`/`nil` failure the record names — there the wall was
built on the first spelling and the second was a bypass; here the wall is built on the **rete** spelling
and the core spelling is what a `where` refuses. The second name is the key, not a door around it.)

### Three implementation classes

| class | work | members |
|---|---|---|
| **alias** | rete name → the same routine; zero new logic | comparisons, boolean, `string::length`/`trim`/`to-lowercase`, `length`/`contains?`/`get`, the holon four if S2 confirms them |
| **fallback surface** | a second terminal handler on the shared kernel | `i64::{+ - * / mod rem quot}`, `f64::{+ - * /}`, `first`/`nth`, `string::subs` — T1's 8, plus the NaN re-audit's additions |
| **form mirror** | head-table entry (`if`/`let`/`do`/`when`) · structural-guard widening (`cond`/`match`/`fn`) | checker-side |

Most of the vocabulary is **class 1**. That is why "grow by demand" stays cheap: a new total op is a
name pointing at a routine that already exists.

### ⛔ STOP-A — ground the arithmetic path before hanging the surface

Two sites raise `IntegerOverflow`: `runtime.rs:4829` (inline, from the eval arm) and `:9753` (through
`arith_i64_i64_inner`). One is factored, one is not. A grep cannot tell whether that is a duplicate or
a fast path. **Ground which path a `where` expression actually traverses before deciding where the rete
surface hangs.**

This matters beyond housekeeping: after #49a there will be **two** evaluation paths for a `where` — the
interpreted `eval_test_core` walk (`matcher.rs:1125`) we have today and the compiled executor being
specified. Both must give the same answer for `:wat::rete::i64::+`. The shared kernel is what makes that
true by construction instead of by two implementations that happen to agree.

## Why — three legs, and only two are measured

| leg | claim | status |
|---|---|---|
| **Totality** | partial ops in a `where` are a live hazard: `first`-on-empty compiles, fires until one empty vector, then aborts the whole fire | **MEASURED** — 39/98 corpus rows refused, 8 verbs (T1) |
| **Coherence** | a partial mint leaves a namespace seam mid-expression with no visible logic to which side a verb falls on | structural |
| **Compilability** | a closed head-space turns `where`-compilation from *"compile a large fraction of wat"* into a finite jump table | structurally true; **payoff UNMEASURED — see the bound** |

### ⚠ The perf leg is bounded, by our own prior stone

`DESIGN-STONE-compiled-where.md`'s Step 0 is a STOP gate written against exactly this temptation:

> *"`filter` is the whole filter-pass loop, not the predicate… a per-TestNode `new_tokens = ts.clone()`
> — at `[50 200]` that is 10,000 Token clones per round… **not the predicate at all**. 'filter is
> 89.5%' ≠ 'the predicate is 89.5%'."*

It cites `[[feedback_measure_the_decomposition_never_read_it]]`. Task #50 tracks that clone separately.
**Totality and coherence justify this build on their own. The size of the speedup is Step 0's to
report, and nobody may say "shockingly faster" before it does.**

Grounded: `compiled_cond.rs` and `compiled_rhs.rs` exist; there is **no** `compiled_where.rs`; `where`
runs `eval_test_core` (`matcher.rs:1125`) from four sites in `kernel.rs`.

### The Clara contrast is R18 one layer down

Clara admits arbitrary Clojure in `:test`. That is *why* it cannot compile the predicate and *why*
impurity leaks into conditions. The scope-reduction is not a smaller engine — it is the weapon, the
same shape as the purity advantage at the negation layer, now at the `where` layer.

## Why the corpus is the wrong instrument for sizing this

T1 measured **8** partial verbs across 98 rows. That is a **floor, not a target**:

> `[[feedback_optimize_for_the_expressivity_surface_not_the_corpus]]` — *the corpus is a record of what
> happened to COMPILE, so it is structurally blind to the fence.*

Those rows contain what was expressible under the *old* rules. They cannot say what someone will want
to write once the fence is a real boundary. Sizing this design to them would be designing to
survivorship — the mistake the first draft made.

**And the vocabulary is not enumerated up front.** Builder: *"my example above is obviously not a
complete set… we'll grow them as the expressivity builds itself by demand."* The **wall** must be
complete on day one; the **vocabulary** must not be. What makes that safe is the module-set test: a new
op is a name in an existing module, or a new module in a short list — both checkable, neither a rewrite.

## ★ Why `:undefined` rather than guards — `if` IS lazy, and it does not help

**Measured:** `(if (= 0 0) 42 (:wat::core::i64::/ 1 0))` prints `42`. `if` evaluates only the taken
branch, so a hand-written guard is genuinely safe *at runtime*.

**But the fence is a STATIC WALK.** `classify_expr` visits every sub-expression including the untaken
branch, sees `i64::/`, and refuses. Accepting a guard would require the checker to *prove the guard
sufficient* — theorem-proving, inside a checker deliberately kept magic-free.

**`:undefined` is what makes totality structural instead of provable.** That is the justification; it is
not a workaround for missing laziness.

## What "total" MEANS — builder-ruled, stricter than IEEE

> *"NaN /is/ an undefined here… force the users to deal with what happens when they get 'not-a-number'
> or they hit a positive or negative infinity — all of these are 'undefined' in my mind."*

**Total = produces an ORDINARY VALUE on every input. Not "never raises."** IEEE achieves totality by
sentinel; a sentinel is the substrate's own name for undefined, and NaN is worse than a raise because
*every comparison against it is false* — the rule silently does not fire.

**Forces a re-audit of T1's `total: true` entries** (bounded to those, not the 110-verb list). One
question each, answered by reading the implementation: *can this produce NaN or ±Inf?*

```
f64::+  f64::-  f64::*            <- overflow to ±Inf        => NOT total
bigint::to-f64  rational::to-f64  <- unbounded source → +Inf => GROUND IT, likely NOT total
i64::to-f64                       <- i64 max ≪ f64 max        => keep
bigint::+ -   rational::+ -       <- arbitrary precision      => keep
rational/numerator /denominator   <- accessors                => keep
```

**The closure property this buys:** if every *producer* of NaN/±Inf requires `:undefined`, NaN cannot
arise inside a `where`, so comparisons are safe by construction.

**⚠ The hole, stated not papered:** a *fact field* may already hold NaN. The fence governs expressions,
not data. An ingested NaN reaches a comparison and silently answers false. This design does not close
that.

## The vocabulary — a starting set, growing by demand

| module | shape |
|---|---|
| `rete::core` | `if let do when` (head-table) · `cond match fn` (structural guards) |
| `rete::i64` | `+ - * / mod rem quot` — **`:undefined` required** |
| `rete::f64` | `+ - * /` — **`:undefined` required** (per the NaN ruling) |
| `rete::core` (compare) | `< > <= >= = not=` — plain; safe by the closure property |
| `rete::core` (bool) | `and or not` — plain |
| — | field accessors: NOT mirrored; admitted by the declaration-derived door |
| `rete::core` (coll) | `length`, `contains?`, `get` plain; **`first`/`nth` take `:undefined`** |
| `rete::string` | `length`, `concat`, `starts-with?`, `ends-with?`, `to-lowercase`, `trim` plain; **`subs` takes `:undefined`** |
| `rete::holon` | `cosine`, `dot`, `coincident?`, `presence?` — the VSA seam; see below |
| `rete::time` | *(demand-driven, see below)* |

### Time ops — admitted as operations on VALUES, never as a clock read

Builder: *"some time ops need to be allowed."* No new machinery is required, and the existing wall is
already correct: `compile-condition` gates on `(and is-pure is-det)` (`rete.wat:661`), so
`:wat::time::now` is **already refused by the determinism conjunct** — and must stay refused. A `where`
that reads a clock breaks pure replay (R5: the snapshot is `{facts, rules}`; re-firing must give the
same answer, or the oracle differential means nothing).

So `rete::time` enters as pure/total/deterministic operations on **timestamp values already in working
memory** — compare, subtract, bucket. The clock itself stays outside the fence.

### The holon seam — a CLASSIFICATION, not a mint

Four verbs (`cosine`, `dot`, `coincident?`, `presence?`) were ruled pure **2026-08-01** precisely to
open R4's designed VSA-matched-LHS seam; `purity.rs:945` records that leaving them unclassified had
*"welded shut R4's designed VSA seam."* The canonical form is written into `purity.rs:370`:

```clojure
(:wat::core::f64::> (:wat::holon::cosine ?a ?b) 0.9)
```

**All four are `total: false` today** (T1 default-deny; no corpus row uses them). **Arming without them
re-welds the seam shut, one day after it was opened.**

**And cosine CANNOT produce NaN** — grounded in the sibling repo,
`holon-rs/src/kernel/similarity.rs`:

```rust
let denom = (na * nb).sqrt();
if denom < 1e-10 { 0.0 } else { dot / denom }
```

The degenerate case is already guarded. Designing a `:undefined` for it would be
`[[feedback_an_unreachable_arm_accumulates_lies]]`. So the four are **likely already total** — four
grounded classifications, not four fallback-carrying mints. (`dot` still needs its own check: can a
large-dim f64 vector reach ±Inf?)

**⚠ OPEN, and it follows from the NaN ruling:** that guard returns `0.0`, which in cosine's semantics
means *"orthogonal, unrelated"* — a plausible-looking answer for an undefined comparison, and arguably
worse than NaN because NaN propagates visibly while `0.0` sails through `(f64::> … 0.9)` as a confident
*no match*. Should `rete::holon::cosine` require `:undefined` for the degenerate case? **Builder's call.**

## The call shape — RULED, and probe-verified

```clojure
(:wat::rete::i64::+ a b :undefined -1)
(:wat::rete::f64::/ x y :undefined 0.0)
(:wat::rete::core::first xs :undefined 0)
```

Positional operands in core's order; **`:undefined` is the only kwarg**. Killed full-kwargs with one
form — *"`(+ :1 0 :2 2 :3 massive-int)` …. wtf is a kwarg for `+`?"* The principle: positional
confusion is a defect only where there is **no established order convention**. Division's order is
universally known; naming it buys nothing on a form written constantly.

**Probe-verified, `--check` AND a run, both directions:** mixed positional+kwargs signatures are
admitted; omitting the fallback fails at expansion with `kwargs-lower: missing argument :undefined`.
**Mandatory-ness needs no new machinery.**

## The wall is ONE-DIRECTIONAL

Rete ops are **ordinary wat functions**. Calling one outside a rule is odd, not illegal, and it keeps
working. What is illegal is the reverse: a non-conforming op inside a `where`.

Deliberate and load-bearing — it keeps the ops testable and composable as plain functions rather than a
walled garden reachable only through the engine, and "grow the expressivity" means adding ordinary
functions, not extending a special form.

## Strike order

| | |
|---|---|
| ~~**S0**~~ | ~~Rule the fork~~ — **RULED: law A, namespace-based.** |
| **S1** | Re-audit T1's `total: true` entries against the ruled definition. Bounded, mechanical. |
| **S2** | Classify the 4 holon verbs (grounded per-verb; `dot`'s ±Inf reachability checked). |
| **S3a** | **Ground STOP-A** — which arithmetic path does a `where` traverse? Decides where the surface hangs. |
| **S3b** | Mint the fallback-carrying ops as a second handler on the shared kernel. **Native.** |
| **S4** | Give the already-total families their rete presence (class-1 aliases) + the module-set admission test in `head_ok`. |
| **S5** | The form mirrors — head-table (`if`/`let`/`do`/`when`) and structural-guard widening (`cond`/`match`/`fn`). |
| **S6** | Migrate the 39 refused corpus rows. |
| **S7** | **ARM** — the third conjunct at the fence. Never before S6. |

**S3b must be NATIVE, not wat wrappers.** A wat `defn` wrapping `i64::/` gets walked by `classify_fn`,
which sees the partial core op in its body and classifies the wrapper non-total — it would fail the
very fence it exists to satisfy. Implement as Rust surfaces over the shared kernel, classified
`total: true` directly.

## STOPs

- **⛔ STOP-A — the arithmetic path** (above). Do not hang the rete surface on a grep.
- **⛔ The bare prefix.** `starts_with(":wat::rete::")` admits `fire-rules` in a `where`. The test is the
  **module set**.
- **⛔ Do not collapse the two form classes.** `if`/`let`/`do`/`when` are head-table entries;
  `cond`/`match`/`fn` are structural guards. Different edits.
- **⛔ The `_` wildcard.** `check.rs:5700`'s exhaustiveness error offers `"(or include `_` wildcard)"`.
  The `_`-arm-on-an-enum ban is doctrine whose checker rule is **unbuilt**, so nothing will stop a rider
  taking it. Taking it is a rejected strike.
- **⛔ Never a second implementation.** A rete op is a second *handler* on the shared kernel.
- **Do not arm before S6.** A refused `first` with nowhere to go locks a user out of arithmetic.
- **Do not claim the speedup.** Step 0 of the compiled-where stone owns that number.
- **Do not design a fallback for an unreachable arm.** Prove reachability before adding `:undefined`.
- **No IO, spawn, service, mutation or clock ops enter this vocabulary**, ever. The boundary is a rule
  *condition*, not general computation.

## ✅ RULED 2026-08-02 — the match is the shield, and it is not laid down

> *"our stance has always been the match verbosity is our shield… for rete forms we can have an
> `:undefined` value provided in our rete forms to make it more ergonomic — but the expressive match
> is our shield and we will not lay it down."*

This closes the guarded-`0.0` open and it closes it on **both** surfaces, exactly as the shared-kernel
law predicts:

- **Core `:wat::holon::cosine` (and its family) returns a matchable outcome.** Every domain gap —
  a dimension disagreement, a degenerate zero-magnitude vector — becomes a named variant the caller
  faces. The manufactured `0.0` that reads as *"orthogonal, unrelated"* dies. Verbosity is the price
  and it is the point.
- **`:wat::rete::holon::cosine` takes `:undefined`.** The seam's canonical one-liner stays a one-liner
  *inside a `where`*, because the fence has already constrained everything reachable there. `:undefined`
  buys ergonomics where the constraint earns it — **it does not replace the match at the core surface.**

One kernel, two terminal handlers: core hands you the outcome to face; rete substitutes the fallback
you declared.

### The strike this implies — 4 verbs, 56 call sites, ONE guard

`pair_values_to_vectors` (`runtime.rs:18506`) is the shared helper carrying the single dimension guard
for the whole family — one place to convert:

| verb | fn | wat-corpus callers |
|---|---|---|
| `cosine` | `eval_algebra_cosine:18587` | 22 |
| `presence?` | `eval_algebra_presence_q:18623` | 17 |
| `coincident?` | `eval_algebra_coincident_q:18677` | 16 |
| `coincident-explain` | `eval_algebra_coincident_explain:18723` | 1 |

### ⛔ TWO THINGS BLOCK DRAWING THE BRIEF

1. **The degenerate case's reachability is UNGROUNDED**, and it decides whether the outcome carries two
   variants or three. `Similarity::cosine` guards `norm < 1e-10 → 0.0`; a zero-magnitude Vector needs
   all-zero cells. **Suspicion, not a finding:** `vector-bundle` / `vector-blend` may cancel to zero,
   which would make the case reachable *through verbs this arc just converted*. Prove it with a
   disconfirming probe before minting a variant — an unreachable arm accumulates lies, and this stone
   already carries that STOP.
2. **The sigma/determinism finding is unruled and lands on 33 of the 56 sites.** `presence?` and
   `coincident?` compute their floor from an **ambient, user-settable** sigma
   (`sigma.rs:77-86` — `WatFnSigmaFn::sigma_at` `apply_function`s an arbitrary user wat fn installed via
   `set-presence-sigma!` / `set-coincident-sigma!`), yet both are classified `deterministic: true`
   (`purity.rs:372-373`) and the fence is armed on pure ∧ det **today**. If the ruling is that the
   setters go, their shape may not change the way `cosine`'s does; if the ruling is that they are
   non-deterministic, the fence refuses them regardless and their rete surface is moot. **Ruling sigma
   first may shrink or reshape this strike** — sequence it ahead.

## Open — the builder's

1. **The sigma setters** (blocker 2 above) — do `set-presence-sigma!` / `set-coincident-sigma!` survive?
   They make two ruled-pure seam verbs read ambient mutable state, which breaks pure replay (R5) and the
   oracle differential if either reaches a `where`.
2. **Does `=` on f64 belong at all?** Float equality is a hazard independent of NaN.
3. **The ingested-NaN limit** — accept as stated, or is there a wall for it?
4. **Naming inside a `where`** — `:wat::rete::i64::+` is faithful to FQDN-always but long for a form
   written constantly. A `where` is a DSL binder context, and the ruling is *bare means scoped* — so
   short names inside a `where` may be legitimate rather than a violation. Worth its own thought.
