# DESIGN — Stone 6a: purity inference (the shared fence for the capability tier)

**Status:** STRIKE-READY (draw)
**Depends on:** nothing new (reads `is_effectful_op`, the symbol table)
**Consumed by:** 6b (`where`/`:test`), 7 (negation conditions), 8 (accumulator `reduce`/`combine`/`init`/`retract`/`convert` fns). Built first — the block-and-build dependency of the whole capability tier.

## Why

Stones 6–8 let a rule embed a *function* in its conditions/aggregations: `(where (:my::suspicious? ?req))`, an accumulator's `reduce` fn, an expression-join `(> ?a ?b)`. The engine's fire phase is a **pure function of the facts** (load-bearing — truth maintenance and the oracle≡native differential both rest on it). So every embedded function must be **pure**, and that must be guaranteed *structurally*, not by convention.

**"Pure" here = a deterministic function of the facts.** Impurity has *two* sources the fence must deny: (1) **effects** (IO/spawn/mutation — `is_effectful_op`), and (2) **non-determinism** (randomness/clock/entropy). The second is the subtle one: a non-deterministic condition does no IO yet still breaks `WM = pure fn(facts × rules)` — it could match the same facts on one fire and not the next, destabilizing a derived fact's support and corrupting truth maintenance.

The builder's grounding: *"who is going to reach for a performant rete and not use their own fn for filtering? once we get to accumulators it's immediately necessary."* User-defined predicates are table stakes, not a follow-on — so the fence must reason about **user fns transitively**, not just intrinsics.

## The one contract decision — default-DENY (four-questions, decided)

A head is **pure** iff it can be *proven* pure; anything unproven is **rejected**. Weighed against the alternative (a complete deny-list, default-allow):

| | default-DENY (chosen) | default-allow (rejected) |
|---|---|---|
| Obvious? | YES | YES |
| Simple? | YES | YES |
| **Honest?** | **YES** — sound by construction; cannot silently admit an impure op; only failure is a loud compile error | **NO** — soundness leans on the deny-list staying complete (a maintenance convention); `:wat::core::Uuid/v4` is live proof it is *already* incomplete |
| Good UX? | YES — loud errors naming the op; our intrinsic set is ours, so the pure list is complete-able | (not weighed) |

`Uuid/v4`/`v5` (`string_ops.rs:818`, `new_uuid_v4()` — random) are non-deterministic yet sit *outside* the five `is_effectful_op` namespaces. Default-allow would admit them silently → non-deterministic conditions → broken truth maintenance. **Default-deny makes the silent-corruption direction unrepresentable**; the only failure mode is over-denial → a loud, fast-to-fix compile error. extirpare top rung.

## What it delivers

A purity classifier, homed at `src/rete/purity.rs` (rete is the only consumer today; lift to a general home if another appears — "let the need reveal it"):

```
pub(crate) fn is_pure_expr(ast: &WatAST, sym: &SymbolTable) -> bool
pub(crate) fn is_pure_fn(fqdn: &str,   sym: &SymbolTable) -> bool
```

**Per-head decision (default-deny):**
1. `is_effectful_op(head)` → **impure**. (The known-impure namespace seed — `:wat::kernel::`/`:wat::io::`/`:wat::eval-`/`:wat::load`/`:wat::config::`.)
2. head ∈ the **non-deterministic intrinsic** set (`:wat::core::Uuid/v4` — random) → **impure**. (The gap default-allow would have missed: it does no IO but is non-deterministic.) NOTE the sibling distinction — `Uuid/v5` (SHA1 of namespace+name), `Uuid/from-string`, `Uuid/to-string`, `Uuid/nil` are all **deterministic ⇒ pure**; only `v4` is random.
3. head ∈ `sym.functions` (a **user fn**) → recurse into its body: `is_pure_fn` = `is_pure_expr(body)`. Transitive.
4. head is a **known-pure intrinsic** (the allow-list, below) → pure; recurse into args.
5. otherwise (unknown head) → **DENY**.

Non-list ASTs: literals/keywords/symbols are pure (data). A `?var` is pure (a bound value). `quote`/quasiquote sub-forms are data — pure (not evaluated as calls). Vectors/maps recurse element-wise.

**The pure-intrinsic allow-list** (the curated, *complete-able* set — this is the load-bearing surface, enumerated from the `dispatch_keyword_head_value` table and weighed against it):
- pure namespaces by prefix: `:wat::core::string::`, `:wat::core::regex::` (cleanly pure), `:wat::math::` (if present).
- explicit pure `:wat::core::` ops (the mixed namespace — arithmetic `+ - * / mod …`, comparison `< > <= >= = not=`, boolean `and or not`, collection/map/vector readers+predicates `get contains? length empty? nth first … keys vals`, type predicates).
- **excludes** `Uuid/v4`/`v5` (step 2) and anything not listed.

**Cycle + memoization:** `is_pure_fn` carries a `seen: &mut HashSet<String>` of fqdns mid-evaluation. A back-edge to an fqdn already in `seen` contributes no new impurity (the recursive call's purity is decided by the fn's *other* ops) → treat the back-edge as pure (standard purity fixpoint: assume-pure on the cycle, falsify on any concrete impure leaf). Memoize decided fqdns to keep it linear.

**`is_effectful_op` → `pub(crate)`** (currently private in `runtime.rs:22519`). A forced-minimal megafile touch — one visibility change, no logic. Single source of truth for the effect surface; the purity classifier consumes it rather than re-listing the namespaces (no drift).

## The wat surface — `(:wat::rete::pure? <quoted-expr>)` → `:bool`

Four-questions decided 6a exposes a thin wat predicate (not Rust-internal-only): a diagnostic sibling to EXPLAIN, the exact shape of `alpha-match`/`eval-insert`/`step-payload` — evaluates its (quoted) arg to a `WatAST` and returns `is_pure_expr`. Two payoffs: (1) 6a has a **real consumer immediately** (no dead-classifier window before 6b), (2) the RED probe tests the real classifier *through the real surface* (vocare). Dispatched at `runtime.rs:~4001`, beside the sibling rete primitives.

```
(:wat::rete::pure? (:wat::core::quote (:wat::core::> (:wat::core::- 5 3) 1)))   ;; → true
(:wat::rete::pure? (:wat::core::quote (:wat::core::Uuid/v4)))                   ;; → false
```

## RED probe (`tests/probe_arc278_6a_purity.rs`)

Disconfirms at HEAD (`:wat::rete::pure?` has no dispatch arm → eval error). Asserts, against a `startup_from_source` world defining pure + impure user fns, each via `(:wat::rete::pure? (:wat::core::quote <expr>))`:
1. pure intrinsic expr `(:wat::core::> (:wat::core::- 5 3) 1)` → **true**.
2. `string::starts-with?` expr → **true**.
3. effectful expr `(:wat::io::IOReader/open-file "x")` → **false** (step 1, namespace seed).
4. **`(:wat::core::Uuid/v4)` → false** (step 2 — the load-bearing assertion; non-deterministic, the case default-allow gets wrong).
4b. **`(:wat::core::Uuid/v5 (:wat::core::Uuid/nil) "x")` → true** (the v4/v5 boundary — v5 is deterministic ⇒ pure; guards against over-denying the Uuid family).
5. pure user fn `(:test::pure-double 5)` → **true** (step 3, transitive).
6. user fn transitively calling `Uuid/v4` `(:test::impure-uuid)` → **false** (step 3 — transitive hole closed; load-bearing).
7. unknown head `(:not::a::real::op 1)` → **false** (step 5 — default-deny).
8. self-recursive pure user fn `(:test::countdown 3)` → **true** (cycle handled, terminates).

## Out of scope = rejected (not deferred)

- **The `where`/test node + `eval-test`** — that's 6b. 6a ships the classifier + the `pure?` predicate + the probe; 6b wires the classifier into `defrule`-compile and adds the end-to-end reject.
- **Lifting purity to a general `src/purity/` home** — stays in `src/rete/` until a non-rete consumer appears.
- **An effect/purity *type* system** (tracking purity in signatures) — a far larger horizon; this is a structural classifier over the call graph, nothing in the type checker.

## Files

- `src/rete/purity.rs` (NEW) — `is_pure_expr` / `is_pure_fn` + the allow-list + `eval_pure_predicate` (the `pure?` primitive entry point).
- `src/rete/mod.rs` — `mod purity;`.
- `src/runtime.rs` — `is_effectful_op` `fn` → `pub(crate) fn` (forced-minimal); one dispatch arm `":wat::rete::pure?" => crate::rete::purity::eval_pure_predicate(...)` at ~4001.
- `src/check.rs` — TypeScheme for `:wat::rete::pure?` (`:wat::WatAST -> :wat::core::bool`), beside the sibling rete primitives (forced-minimal).
- `tests/probe_arc278_6a_purity.rs` (NEW) — the RED probe.
