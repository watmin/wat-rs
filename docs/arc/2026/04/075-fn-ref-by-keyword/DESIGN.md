# Arc 075 — Keyword-as-function-reference resolution + default filter funcs

**Status:** **CLOSED 2026-04-28** — Change 1's premise didn't hold; the gap is real but sits in a different layer. See `CLOSURE.md` (sibling file) for the full forensic + the actual concern that surfaced.

**TL;DR for future readers:** keyword-as-fn-ref resolution (Change 1) **already ships** under arc 009 ("names are values"). Both `runtime.rs:2082-2084` (eval-time) and `check.rs:426-433` (infer-time) already do exactly what this arc proposed. The infrastructure exists.

The actual gap arc 075's surfacing-symptom pointed at: **`deftest`'s sandbox isolation.** `:wat::test::deftest` macroexpands the body into a `run-sandboxed-ast` sub-world; the sandbox carries only the spliced forms, not outer-file defines. When the test tries to reference `:my::tight-filter`, the sandbox's `sym` doesn't have it. The type-check fail at parameter #4 reads as a type-checker miss; it's actually a sandbox visibility gap.

**Change 2 (substrate-shipped default filter funcs) was load-bearing and survives** — but doesn't need its own arc; it's a small slice that fits inside arc 074 slice 1's follow-up.

The original draft below remains as a historical artifact. CLOSURE.md explains the diagnosis error and what the substrate session learned from it.

---

**Status (original draft):** drafted 2026-04-28, awaiting infra. Pre-implementation reasoning artifact.
**Predecessors:** arc 071 (lab-harness enum-method parity — same shape of "registration gap surfaced at consumer site"), arc 074 (`HolonHash` slice 1 — the immediate surface where the gap shows up).
**Surfaced by:** arc 074 slice-1 review. The `:wat::holon::HolonHash/get` filter parameter (`:fn(f64)->bool`) accepts inline `lambda` constructors but not defined-function-by-keyword-path. The SLICE-1 doc already documents the workaround; this arc closes the gap.

Builder direction (2026-04-28, after the SLICE-1 doc surfaced the workaround):

> "explain this.. why is passing a func not supported?.... this is remarkably confusing.... and the contract for user is pass a function who impl :fn(f64)->bool -- what they do in that func is their issue. we should expose the default presence? and coincidence? values so they can use them (and our opinionated filter fucns directly use them as an example on how to make these)"

> "new arc - we need this capability"

Two changes in one arc, named together because they're felt together at the call site:

1. **Uniform keyword-as-function-reference resolution.** Defined functions become first-class values when passed at any `:fn(...)` argument position.
2. **Substrate-shipped default filter funcs.** `filter-coincident`, `filter-present`, `filter-accept-any` ride on (1) — passable by keyword path once it lands.

---

## The gap, shown

Today:

```scheme
;; Define a filter — signature matches what HolonHash/get wants.
(:wat::core::define
  (:my::tight-filter (cos :f64) -> :bool)
  (:wat::core::< (:wat::core::- 1.0 cos) 0.01))

;; Try to pass it as the filter argument:
(:wat::holon::HolonHash/get store pos probe :my::tight-filter)
;;                                          ^^^^^^^^^^^^^^^^^
;; type-mismatch: parameter #4 expects :fn(f64)->bool;
;; got :wat::core::keyword
```

Workaround:

```scheme
(:wat::holon::HolonHash/get store pos probe
  (:wat::core::lambda ((cos :f64) -> :bool)
    (:my::tight-filter cos)))
```

The lambda is a thin trampoline that calls the same function. The user paid for the indirection in source-code verbosity. Multiplied across every fn-typed argument site, this is real cost.

## Why it happens

The wat parser emits `:my::tight-filter` in argument position as a `Keyword(":my::tight-filter")` token. The type checker's default arg-resolution treats keywords as keyword literals (type `:wat::core::keyword`).

To resolve as a function reference, the checker must:
1. Notice the arg position expects `:fn(args...) -> ret`.
2. Look up `:my::tight-filter` in the function registry.
3. Find a registered function with matching signature.
4. Treat the keyword as the value of that registered function.

Today this resolution is opt-in per dispatch entry (and probably implicit through case-by-case handling). It works for `:wat::eval::walk`'s visitor arg because the substrate's check.rs and runtime have specific handling there. It doesn't work for `:wat::holon::HolonHash/get`'s filter arg because that handling wasn't replicated.

This is the same shape of gap that arc 071 closed for `register_enum_methods` — uniform registration that should run at one canonical setup point but had been done per-site. The pattern repeats: substrate ships a capability; the type checker has the metadata; some surfaces wire it up; others don't.

---

## Change 1 — uniform resolution rule

**Type-checker rule:**

> When the type checker encounters a `WatAST::Keyword(name)` in an argument position whose expected type is `:fn(args...) -> ret`, AND the function registry contains a registered function at `name` whose signature unifies with `:fn(args...) -> ret`, treat the keyword as a function-value reference resolving to that registered function.

**Failure mode unchanged:** if no function with matching signature is registered, type-check fails with a clear message naming what was expected vs. what was found. Today's behavior is the same — the user sees `expected :fn(f64)->bool; got :wat::core::keyword`. After this change, the user might see something more pointed:

> `expected :fn(f64)->bool; got :wat::core::keyword (`:my::tight-filter` is registered but its signature `:fn(i64)->bool` doesn't unify with the expected type)`

That's a UX bonus; not load-bearing.

**Where this applies:** every dispatch entry whose check.rs signature has a `:fn(...)` arg type. Today these include (non-exhaustive):
- `:wat::eval::walk` (visitor)
- `:wat::holon::HolonHash/get` (filter)
- Any future cache `get`, `match`, or filter primitive
- User-side function-passing in `let*`, function args, etc.

After arc 075, the rule fires uniformly. New surfaces inherit the behavior automatically — no per-site wiring.

---

## Change 2 — substrate-shipped default filter funcs

The arc 074 slice-1 doc already exposes `presence-floor` and `coincident-floor` as raw f64 accessors, with example lambdas built from them. The substrate should ship the canonical filter funcs themselves, so users compose by passing them by name (after change 1 lands):

| Name | Signature | Behavior |
|------|-----------|----------|
| `:wat::holon::filter-coincident` | `:fn(f64)->bool` | `(1 - cos) < coincident-floor(ambient-d)` |
| `:wat::holon::filter-present` | `:fn(f64)->bool` | `(1 - cos) < presence-floor(ambient-d)` |
| `:wat::holon::filter-accept-any` | `:fn(f64)->bool` | `true` always — for "give me whatever is best from candidate set" |

**Usage after both changes land:**

```scheme
;; Strict: only return val if cosine clears coincident floor
(:wat::holon::HolonHash/get store pos probe :wat::holon::filter-coincident)

;; Looser: return val if there's any presence
(:wat::holon::HolonHash/get store pos probe :wat::holon::filter-present)

;; Pure population readout: return whatever scored highest, no gate
(:wat::holon::HolonHash/get store pos probe :wat::holon::filter-accept-any)

;; Or compose your own:
(:wat::core::define
  (:my::twice-as-loose (cos :f64) -> :bool)
  (:wat::core::let* (((floor :f64) (:wat::holon::coincident-floor 10000)))
    (:wat::core::< (:wat::core::- 1.0 cos) (:wat::core::* 2.0 floor))))

(:wat::holon::HolonHash/get store pos probe :my::twice-as-loose)
```

**Implementation question — ambient d.** The default filters need to compute floor at SOME d. Two options:

- **(a)** Read `d` from the ambient encoder/router at filter call time. Filters then carry an implicit dependency on the runtime config; this matches how `presence?` and `coincident?` already work (they read ambient sigma/d via the encoder context).
- **(b)** Ship as filter *constructors* — `(:wat::holon::filter-coincident d)` returns a `:fn(f64)->bool` curried at d. Caller passes `d` explicitly:
  ```scheme
  (:wat::holon::HolonHash/get store pos probe (:wat::holon::filter-coincident 10000))
  ```

(a) keeps the surface minimal and matches existing predicate semantics. (b) is more explicit but the user repeats `d` at every call site. **Recommend (a)** — same discipline as the existing predicates; simpler call site; consistent with the substrate's "ambient encoder" pattern.

---

## What's already there (no change)

| Surface | Status |
|---------|--------|
| Function registry — registered functions accessible by keyword path | shipped (since the language existed) |
| `:wat::core::lambda` — inline anonymous functions | shipped |
| `:wat::eval::walk`'s visitor arg accepts keyword paths | shipped (works case-by-case in arc 070's runtime) |
| `:wat::holon::presence?`, `:wat::holon::coincident?` predicates | shipped (arc 023) |
| `:wat::holon::presence-floor`, `:wat::holon::coincident-floor` accessors | shipped (arc 074 slice 1) |
| Substrate's encoder context (ambient `d` access) | shipped |

Six pieces in place. The new surface is one type-checker rule + three function definitions.

## What's missing (this arc)

| Op / change | What it does |
|----|----|
| Type-checker resolution rule (change 1) | Makes keyword-paths-to-defined-functions resolve as function-value references in all `:fn(...)` argument positions, uniformly. Replaces per-site special-casing. |
| `:wat::holon::filter-coincident` | Substrate-shipped default filter that gates on coincident floor at ambient `d`. |
| `:wat::holon::filter-present` | Substrate-shipped default filter that gates on presence floor at ambient `d`. |
| `:wat::holon::filter-accept-any` | Substrate-shipped null-filter that always returns true. |
| USER-GUIDE row + filter-passing example | Doc surface for the cleaner pattern. Updates the SLICE-1-HOLON-HASH doc's "use inline lambda" workaround language to "or pass a defined-function keyword directly." |

Five pieces. The type-checker change is the only structural work; the three filter funcs are short bodies on top.

---

## Open questions for the substrate session

1. **Module-scoped resolution.** Defined functions can live in nested keyword paths (`:my::scope::sub::fn`). The resolution rule should walk the registry just like any other keyword-path lookup. Recommend: yes — uniform with how the registry is read elsewhere.

2. **Polymorphic / parametric functions.** Some functions are parametric over a type variable (e.g., `:my::id<T> :: T -> T`). In the arg position, the expected `:fn(...)` is monomorphic. The resolution should attempt to unify the parametric function's signature against the expected concrete signature. Recommend: yes; if unification fails, type-error as today.

3. **Recursive functions.** A function passed by keyword that calls itself works as long as the registry resolves the same name to the same callable. Should be a non-issue. Recommend: confirm via test.

4. **Existing ad-hoc keyword args.** Some dispatch entries already accept keyword args meant as keywords (not function references). The resolution rule should only fire when the *expected type* is `:fn(...)` — keywords expected as keywords still resolve as keywords. Recommend: keep the rule signature-typed so it doesn't false-match.

5. **Filter func ambient-d implementation.** Per the recommendation above, filter funcs read ambient encoder `d`. The substrate already exposes encoder context to predicate primitives; the filters use the same path. Recommend: option (a).

6. **Backward compat for `:wat::eval::walk`'s visitor.** That arg currently works via case-by-case handling. After arc 075 the uniform rule should subsume it. Recommend: verify the existing walk tests still pass; remove any per-site handling now made redundant.

---

## Test strategy

Three layers, building on each other:

**Substrate-side unit tests** (`tests/wat_arc075_*.rs`):
- `defined_fn_passable_by_keyword_to_walk_visitor` — `:my::test::count-visit` passable to `:wat::eval::walk` directly
- `defined_fn_passable_by_keyword_to_holon_hash_filter` — `:my::tight-filter` passable to `:wat::holon::HolonHash/get`
- `signature_mismatch_errors_clearly` — wrong-signature defined fn errors with the new pointed message
- `filter_coincident_returns_true_for_close` / `_returns_false_for_distant` — substrate-shipped filter behavior at d=10000
- `filter_present_returns_true_for_close` / `_returns_false_for_distant`
- `filter_accept_any_returns_true_unconditionally`

**Lab-side regression** (in the existing trader's wat-tests):
- Update proof 018 / 019's filter args to pass defined functions by keyword path; confirm tests still pass.
- Update USER-GUIDE example to use the cleaner shape.

**Acceptance gate:**
- All existing wat-rs unit tests pass.
- The three new filter funcs load and execute.
- The keyword-as-fn-ref resolution covers at least walk/visit and HolonHash/get/filter.
- The SLICE-1-HOLON-HASH doc's "Filter funcs in wat" section updates to remove the inline-lambda workaround language (or keeps it as one of two valid patterns).

---

## What this arc deliberately does NOT do

- **Does not introduce function overloading.** wat does not have signature-based overloading. The keyword path resolves to ONE function in the registry. If multiple registrations exist (which they shouldn't — registry is single-entry per keyword), that's a substrate bug, not this arc's concern.
- **Does not redesign `:fn(...)` syntax.** The function-type syntax stays as it is.
- **Does not add closures-with-captured-state.** A function reference passed by keyword is the same function value the registry holds; no closure environment is created.
- **Does not optimize the dispatch path.** Every fn-typed-arg call goes through the same lookup as today; the change is checker-only.

---

## What this arc unblocks

- **Arc 074 slice 1's filter ergonomics close cleanly.** Users write filters as defined functions; pass by keyword; no lambda boilerplate.
- **Arc 074 slice 2 (`HolonCache`) ships the same shape.** Filter args throughout the cache surface are uniform.
- **Lab umbrella 059's slice 1 lands cleanly.** Trader writes domain-specific filters (`:trading::filter-rsi-tolerance`, etc.), passes them by keyword to the cache.
- **Future fn-arg surfaces inherit the behavior.** Any new `:fn(...)` arg in a future dispatch entry just works — no per-site wiring needed.
- **The SLICE-1-HOLON-HASH doc's "open" item (filter func ergonomics) closes.**

---

## The thread

- **Arc 023** — `coincident?` predicate.
- **Arc 071** — lab-harness enum-method parity. Same shape of registration gap; closed by uniform setup.
- **Arc 074 (slice 1)** — `HolonHash`. Surfaces this gap explicitly in its doc.
- **Arc 075 (this)** — keyword-as-fn-ref resolution + default filter funcs.
- **Next** — arc 074 slice 2 (`HolonCache`); lab umbrella 059 slice 1 unblocks.

PERSEVERARE.
