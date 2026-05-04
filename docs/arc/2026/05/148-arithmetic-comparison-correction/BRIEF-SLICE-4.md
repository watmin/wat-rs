# Arc 148 Slice 4 — Sonnet Brief — Numeric arithmetic migration

**Drafted 2026-05-03.** The big slice. Substrate-informed:
orchestrator pre-flighted post-arc-150 state — variadic `:wat::core::define`
SHIPS (per `tests/wat_arc150_variadic_define.rs:16/16`); per-Type
arithmetic leaves at `,2`-suffixed names (slice 2); `values_compare`
extended (slice 3); `infer_polymorphic_compare` retired/renamed
(slice 5).

FM 9 baseline confirmed pre-spawn (post-slice-5 + arc 150):
- `wat_arc146_dispatch_mechanism` 7/7
- `wat_arc144_lookup_form` 9/9
- `wat_arc144_special_forms` 9/9
- `wat_arc144_hardcoded_primitives` 17/17
- `wat_arc143_define_alias` 3/3
- `wat_polymorphic_arithmetic` 20/20
- `wat_arc148_ord_buildout` 46/46
- `wat_arc150_variadic_define` 16/16
- `wat_variadic_defmacro` 6/6

= **133/133** across the substrate-foundation tests.

**Goal:** ship the locked DESIGN's arithmetic architecture. Each
of `+`, `-`, `*`, `/` becomes a polymorphic variadic surface
backed by a binary Dispatch entity routing to per-Type Rust leaves.
Same-type variadic wat fns (i64::+, f64::+, etc.) use arc 150's
variadic define syntax. Retire `eval_poly_arith` + `infer_polymorphic_arith`
+ runtime dispatch arms + freeze redex entries.

**Working directory:** `/home/watmin/work/holon/wat-rs/`

## Required pre-reads (in order)

1. **`docs/arc/2026/05/148-arithmetic-comparison-correction/DESIGN.md`**
   — full arc design. Pay attention to: § "Arithmetic — three layers";
   § "Naming convention"; § "Full enumeration — NUMERIC arc 148 surface";
   § "Arity rules — Lisp/Clojure tradition for arithmetic"; § "Slice 4".
2. **`docs/arc/2026/05/148-arithmetic-comparison-correction/AUDIT-SLICE-1.md`**
   — § "Handler — `infer_polymorphic_arith`". Source of truth for what
   the current handler accepts + how the runtime routes.
3. **`docs/arc/2026/05/148-arithmetic-comparison-correction/SCORE-SLICE-2.md`**
   — what the rename slice shipped (per-Type leaves at `,2` names).
4. **`docs/arc/2026/05/148-arithmetic-comparison-correction/SCORE-SLICE-5.md`**
   — comparison cleanup precedent (path (a) rename + simplify for
   `infer_polymorphic_compare` → `infer_comparison`). Pattern for
   arithmetic's check-side decision.
5. **`docs/arc/2026/05/150-variadic-define/INSCRIPTION.md`**
   — variadic define: shape, syntax, semantics. Your equipment.
6. **`tests/wat_arc150_variadic_define.rs`** — reference test file
   showing variadic define + `:wat::core::foldl` over rest pattern.
7. **`docs/COMPACTION-AMNESIA-RECOVERY.md`** — § FM 9 (baselines done);
   § 12 (foundation work; eliminate failure domains; don't bridge).

## What ships

### 1. 8 mixed-type Rust primitives (NEW substrate registrations)

Per the comma-typed-leaf rule (DESIGN § "The comma-typed-leaf rule"):
arithmetic needs comma-typed mixed leaves because Rust impls genuinely
differ per type-pair (i64+i64 vs i64+f64 are different functions).

Register in `register_builtins`:

```
:wat::core::+,i64-f64    :wat::core::+,f64-i64
:wat::core::-,i64-f64    :wat::core::-,f64-i64
:wat::core::*,i64-f64    :wat::core::*,f64-i64
:wat::core::/,i64-f64    :wat::core::/,f64-i64
```

Each is a binary Rust function: takes (i64, f64) → f64 (or
(f64, i64) → f64); does the type promotion + arithmetic; returns
the result. The current `eval_poly_arith`'s mixed-type arms at
`src/runtime.rs:4810-4860+` are the implementation pattern (the
specific lines you'll find via grep — orchestrator hasn't pinpointed
exact ranges; trust your audit).

Plus: each mixed-type leaf gets a TypeScheme registration in
`src/check.rs` (mirror the per-Type leaf registrations from slice 2).
Plus: each gets a freeze-pipeline pure-redex entry in
`src/runtime.rs:15689+` area.

Total NEW substrate registrations: 8 dispatch arms + 8 TypeSchemes
+ 8 freeze entries.

### 2. 4 binary Dispatch entities (declared in wat/core.wat)

Mirror the arc 146 Dispatch declaration pattern (`wat/core.wat:12+`).
For each op `<v>` ∈ {`+`, `-`, `*`, `/`}:

```scheme
(:wat::core::define-dispatch :wat::core::<v>,2
  ((:wat::core::i64 :wat::core::i64)  :wat::core::i64::<v>,2)
  ((:wat::core::f64 :wat::core::f64)  :wat::core::f64::<v>,2)
  ((:wat::core::i64 :wat::core::f64)  :wat::core::<v>,i64-f64)
  ((:wat::core::f64 :wat::core::i64)  :wat::core::<v>,f64-i64))
```

The `,2` suffix marks "binary form" — sibling to the variadic surface
at the bare name. 4 arms per Dispatch (4 type-pair combinations).

### 3. 8 same-type variadic wat fns (uses arc 150's variadic define)

For each type `T` ∈ {`i64`, `f64`} × each op `<v>` ∈ {`+`, `-`, `*`, `/`}:

For `+` and `*` (associative, identity exists, 0-ary returns identity):

```scheme
(:wat::core::define
  (:wat::core::<T>::<v> & (xs :wat::core::Vector<wat::core::<T>>) -> :wat::core::<T>)
  (:wat::core::cond
    ((:wat::core::empty? xs) <identity>)              ; 0 for +; 1 for *
    ((:wat::core::empty? (:wat::core::rest xs)) (:wat::core::first xs))
    (:else (:wat::core::foldl (:wat::core::rest xs)
                              (:wat::core::first xs)
                              :wat::core::<T>::<v>,2))))
```

For `-` and `/` (NOT 0-ary; 1-ary inserts identity-on-left for negation/reciprocal):

```scheme
(:wat::core::define
  (:wat::core::<T>::<v> & (xs :wat::core::Vector<wat::core::<T>>) -> :wat::core::<T>)
  (:wat::core::cond
    ((:wat::core::empty? xs) <ARITY-ERROR>)           ; - and / require >=1
    ((:wat::core::empty? (:wat::core::rest xs))
     (:wat::core::<T>::<v>,2 <identity> (:wat::core::first xs)))   ; identity-on-left
    (:else (:wat::core::foldl (:wat::core::rest xs)
                              (:wat::core::first xs)
                              :wat::core::<T>::<v>,2))))
```

Where `<identity>` is `0` (for `+`/`-`) or `1` (for `*`/`/`); the
literal type matches `<T>` (so `0:i64` for `:i64::+`, `0.0:f64` for
`:f64::+`, etc.).

`<ARITY-ERROR>` shape: use `:wat::core::error` if it exists, OR
trigger an arity-mismatch via a substrate primitive that panics
honestly, OR shape it however the existing substrate idioms do
arity errors in wat (audit the substrate to find the pattern;
`:wat::test::should-panic` tests + your investigation).

These 8 variadic wat fns live in `wat/core.wat` (next to the Dispatch
declarations).

### 4. 4 polymorphic variadic wat fns (the BIG architectural call)

For each op `<v>` ∈ {`+`, `-`, `*`, `/`}: the polymorphic variadic at
`:wat::core::<v>` (bare name) needs to handle MIXED-NUMERIC inputs
like `(:wat::core::+ 1 2.0 3)`.

**OPEN QUESTION (Q1 — your call):** how to type the polymorphic
variadic. Three paths:

**Path A — parametric Vector<T>:**
```scheme
(:wat::core::define
  (:wat::core::<v> & (xs :wat::core::Vector<T>) -> :T)
  ...)
```
T unifies based on actual args. Same-type works (`(:+ 1 2 3)` → T=i64).
**Mixed args FAIL type-check** (`(:+ 1 2.0)` — T can't unify i64
and f64). Loses the user's `(:wat::core::+ 0 40.0 2) => :f64 42.0`
example.

**Path B — untyped/dynamic Vector:**
```scheme
(:wat::core::define
  (:wat::core::<v> & (xs :wat::core::Vector<???>) -> :???)
  ...)
```
Use whatever untyped-Vector shape the substrate supports. Type
checking permissive at the variadic surface; runtime dispatch verifies
each pair via the binary Dispatch. Mixed works at runtime; less
compile-time safety. **Investigate: does the substrate have an
"any-type Vector" or "Value-type Vector" shape that wat-defines can
declare?**

**Path C — KEEP custom inference (rename + simplify per slice 5 precedent):**
Don't make the polymorphic `:wat::core::<v>` a wat fn at all. Keep
`infer_polymorphic_arith` as the check-side custom rule (rename to
`infer_arithmetic` to drop the anti-pattern framing). The runtime
side: NEW Rust impl that does variadic foldl + per-pair routing via
the Dispatch. The function is registered as a substrate primitive
(SpecialForm or custom), NOT a wat-defined variadic.

Slice 5 chose Path C for comparison (`infer_polymorphic_compare` →
`infer_comparison` rename + body kept). The discipline applies:
custom inference IS honest when the substrate's TypeScheme system
can't express the polymorphism cleanly.

**Recommend Path C unless you find a clean Path A/B.** Surface the
choice in your report with rationale.

### 5. RETIRE: `eval_poly_arith` + dispatch arms + freeze entries

Delete:
- `src/runtime.rs:2744-2747` — the 4 polymorphic dispatch arms
  (`:wat::core::+ → eval_poly_arith` etc.)
- `eval_poly_arith` function entirely
- `PolyOp` enum if no other consumer
- `src/runtime.rs:15889-15892` — the 4 polymorphic freeze pipeline
  pure-redex entries

Keep the per-Type leaf eval fns (`eval_i64_arith`, `eval_f64_arith`)
— they're the binary Rust primitives the per-Type `,2` leaves call.

### 6. RETIRE: `infer_polymorphic_arith` (or rename per Path C)

If Path A or B chosen: retire `infer_polymorphic_arith` entirely +
the dispatch site at `src/check.rs:3326`.

If Path C chosen: rename to `infer_arithmetic`, simplify if useful,
keep the dispatch site routing the 4 op keywords to it. Body
unchanged from slice 5 precedent's reasoning.

## What this slice does NOT do

- NO change to per-Type `,2` leaves (slice 2's work; UNCHANGED)
- NO change to `eval_i64_arith` / `eval_f64_arith` BODIES (only
  registration arms change)
- NO change to comparison family (slice 5 territory)
- NO change to time-arith / holon-pair (DEFERRED parallel track)
- NO new test files (existing `wat_polymorphic_arithmetic.rs` covers
  the surface; you may need to update some test cases that change
  shape under the new architecture — surface in honest deltas)
- NO substrate addition for `:numeric` type union or untyped Vector
  (Path B is risky; Path C avoids this)

## STOP at first red

If Path A and B both turn out blocked (parametric T can't handle
mixed; untyped Vector doesn't exist as a usable wat type), Path C is
the safe pivot. Surface the constraint + your choice clearly.

If retiring `eval_poly_arith` breaks tests in non-obvious ways
(e.g., a test path uses the polymorphic eval directly via Rust call),
STOP and report.

If the polymorphic variadic's runtime impl needs to do per-pair
dispatch via the wat-level `:+,2` Dispatch entity from a Rust impl,
investigate `eval_dispatch_call` at `src/runtime.rs:3221` — there's
likely a pattern for "call a Dispatch entity from inside a Rust
primitive."

## Source-of-truth files

- `src/runtime.rs:2642-2683` — per-Type `,2` registration arms (slice 2's work)
- `src/runtime.rs:2744-2747` — polymorphic arms to retire
- `src/runtime.rs:4761+` — `eval_poly_arith` to retire (find the
  exact line range)
- `src/runtime.rs:15889-15892` — freeze-pipeline polymorphic entries
- `src/check.rs:3326` — dispatch site routing arithmetic ops
- `src/check.rs:6732` — `infer_polymorphic_arith` body
- `src/check.rs:8718-8750+` — TypeScheme registrations for per-Type leaves
- `wat/core.wat:12+` — Dispatch declaration template (arc 146)
- `tests/wat_arc150_variadic_define.rs` — variadic define + foldl-over-rest pattern
- `tests/wat_polymorphic_arithmetic.rs` — existing tests that may need shape updates

## Honest deltas

If you find:
- An existing test case that asserts the OLD architecture in a way
  the NEW shape can't preserve (e.g., an error message that referenced
  the polymorphic handler's name)
- A coupling between `eval_poly_arith` and another substrate path
  the audit didn't catch
- A wat-side caller that uses the polymorphic arithmetic in a way
  that breaks under Path C (more strict; less permissive)

Surface as honest delta. These are signals.

## Report format

After shipping:

1. Path chosen for polymorphic variadic (A/B/C) + rationale
2. Total NEW substrate primitives (should be 8 mixed-type leaves)
3. Total NEW Dispatch entities (should be 4)
4. Total NEW wat-defined fns (8 same-type variadics + 4 polymorphic
   variadics if Path A/B; 8 + 0 if Path C)
5. Total RETIRED substrate code (eval_poly_arith + infer fn if Path A/B
   + dispatch arms + freeze entries)
6. Test results: list which tests changed; confirm all baselines green
7. Workspace failure profile (per FM 9: should be the documented arc
   130 HologramCacheService noise plus the pre-existing
   `call_stack_populates_on_assertion` panicking-test)
8. Any honest deltas surfaced

Time-box: 120 min wall-clock. Predicted Mode A 60-90 min. The
substrate work spans multiple files; arc 146 + arc 150 templates
provide the patterns.

## What this unlocks

**Arc 148 slice 6** — closure paperwork (INSCRIPTION + 058 row +
USER-GUIDE entry showing the variadic arithmetic surface + arc 146
slice 5 unblock note).

**Arc 146 slice 5** — closure (BLOCKED on arc 148 completion;
unblocks at slice 6).

The polymorphic-handler anti-pattern for arithmetic is RETIRED. Every
arithmetic op is a first-class entity at runtime — discoverable via
`signature-of`, addressable directly, queryable by reflection. The
substrate's variadic surface for arithmetic ships per the locked
DESIGN.

LLMs writing wat code can call `(:wat::core::+ 1 2 3)`,
`(:wat::core::i64::+ 1 2 3)` for type-locked, OR `(:wat::core::i64::+,2 1 2)`
for direct binary — same name conventions throughout; no special-cases
to learn.
