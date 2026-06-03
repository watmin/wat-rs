# DISPATCH — clause vs intrinsic: the polymorphism partition

Substrate doctrine. wat has **two** mechanisms for polymorphic operations, and which one an op uses is **not a taste call** — it is decided by a *checkable property of the op's type*. This document is the rule. When you add a new polymorphic op, or reclassify an existing one, the answer comes from here, not from a hunch.

Sibling to `CONFORMARE.md` / `ZERO-MUTEX.md`: same shape of doctrine — a decision made *checkable* so the wrong choice is caught by a property, not argued by vibe.

## The two mechanisms

- **Clause (`defclause`)** — wat-level, declarative. A defclause is a set of clauses, each `[a <- :ConcreteType  b <- :ConcreteType] -> :ReturnType <body>`. Dispatch is **first-match-wins by per-position type match**: the call's arg types are checked against each clause's parameter types *independently, one position at a time*, against **fixed named types**. Numerics live here (`+`, `-`, `*`, `<`, `>`, `<=`, `>=`).

- **Intrinsic (custom Rust inference)** — a hand-written Rust inference function (`infer_<op>`) plus a Rust eval function. Reached for when the op's *type* cannot be expressed as a finite set of concrete-typed clauses. Collections (`get`, `conj`, `assoc`, `contains`, …) and **equality** (`=`, `not=`) live here.

## The principle

> **Intrinsic = the op's type requires type-level computation. Clause = it does not.** A clause is monomorphic: concrete argument types, a fixed return, and **no type variable flows anywhere**. The moment a type variable must flow — into the return, or *between the arguments* — a clause cannot express it, and the op is an intrinsic.

The discriminant is checkable because the clause matcher is mechanical and known: it does `assignable(arg_i, clause_param_i)` **per position, independently** (see *Where it's declared*). It has exactly two blind spots, and those two blind spots *are* the two flavors of intrinsic.

## The two flavors of type-level computation

A clause cannot compute a type. There are two ways an op's type demands that it must — and an op is an intrinsic if it hits **either**:

### Projective — a type flows from the arguments into the return

The return type is a *function of an argument's type parameters*.

```
get : Vector<T>          + i64  -> Option<T>
get : HashMap<K, V>      + K    -> Option<V>
```

The return `Option<T>` is computed from the container's `T`; the key argument's type *is* the container's `K`. A clause is monomorphic — no `∀` — so to cover this it would need one clause per concrete `(K, V)`, an **infinite open set** (users mint new types forever). Unexpressible. `infer_get` projects `T`/`K`/`V` out of the container and flows them into the key-arg and the return. **Projective ⇒ intrinsic.**

### Relational — a constraint flows between the arguments

The arguments are tied to *each other* by a type variable, for all `T`.

```
= : a:T, b:T -> bool          (for ∀T; same-type, or subtype-related)
```

The constraint is "arg1's type must equal arg0's type, **whatever that type is**." The clause matcher *cannot say this*: it checks each argument against a fixed named type, one position at a time, and **never unifies arg0's type with arg1's**. `infer_equality` does exactly that unification (`unify(a, b)`, admitting same-or-subtype pairs, including base-vs-holonic record cross-flavor). A finite clause list would force you to enumerate `[a <- :i64 b <- :i64]`, `[a <- :f64 b <- :f64]`, … — and would **regress** equality on records / composites / user types (which works today via `values_equal`) into `NoMatchingClause` the instant a type has no hand-written clause. **Relational ⇒ intrinsic.**

## The decision procedure

For any candidate polymorphic op, ask — **and check both sides**:

1. **Projective?** Does a type flow from an argument's type parameters into the return (or into another argument)? → **intrinsic**.
2. **Relational?** Are two or more arguments tied to each other by a type variable (`∀T`), such that a correct check must *unify their types*? → **intrinsic**.
3. **Neither** — concrete argument types, a fixed return, no type variable anywhere? → **clause**.

The fixed return type alone does **not** make an op a clause. Equality returns `bool` invariantly and is still an intrinsic, because the constraint lives *between the arguments*, not in the return. Read both sides before you call something monomorphic.

## Worked classifications

| Op | Type | Verdict | Why |
|---|---|---|---|
| `+`, `-`, `*`, `<`, `>` | `i64,i64 -> i64` / `f64,f64 -> f64` | **clause** | concrete args, fixed per-type return, no type-var flow; one clause per concrete numeric type |
| `get`, `conj`, `assoc`, `contains` | `Vector<T> -> Option<T>`, … | **intrinsic — projective** | return is a function of the container's type params |
| `=`, `not=` | `a:T, b:T -> bool` (∀T) | **intrinsic — relational** | cross-argument unification; a clause checks positions independently and cannot tie them |

## Where it's declared (the source markers)

The partition is marked **in the code**, at the dispatch sites, so a reader standing in the substrate sees it without leaving:

- **Inference (check-side):** `fn infer_list` in `src/check.rs` — the keyword-head inference dispatch. Its `":wat::core::<op>" => infer_<op>(...)` arms *declare* which ops are intrinsics (collections route to `infer_get`/`infer_conj`/`infer_assoc`/`infer_contains`). Equality routes to `fn infer_equality` — its `unify(a, b)` **is** the relational flavor, in code.
- **Runtime:** `fn dispatch_keyword_head` / `dispatch_keyword_head_value` in `src/runtime.rs` — routes to the per-container `eval_<container>_<op>` impls; equality routes to `eval_eq` / `eval_not_eq` (over `values_equal`).
- **The clause matcher** (the thing intrinsics are *not*): the defclause call-site checker, also in `infer_list` — first-match-wins, `assignable(arg_i, param_i)` per position. This is the mechanism whose two blind spots define the two intrinsic flavors above.

The collection intrinsic *implementations* lift to `src/collection/` (arc 246); the *declaration* arms stay at the dispatch sites and redirect. Equality stays where it is (no home move).

## The trap (a cautionary tale, recorded so it isn't re-walked)

The original form of this rule read only the **return**: *"equality returns `bool` invariantly → no type-var flow → it's monomorphic → make it a clause."* That mis-classification sent a whole sub-arc (a Clojure-faithful `map` flip + a generative-macro `for`-comprehension) down the road of **generating** equality's clauses with a macro — to consolidate ~22 "identical" per-type clauses.

The dig reversed it (2026-06-03/04): the clause matcher checks positions *independently* and never unifies arg0 with arg1, but equality *is* that cross-argument unification; a finite clause list would regress record / composite / user-type equality. Equality was an intrinsic all along — the relational flavor — and the existing `infer_equality` + `eval_eq` were already correct. The macro-comprehension tool (`for`) survives as a general per-Type-boilerplate generator; it was simply never the equality vehicle.

The lesson is the rule's second clause: **check both sides — projection AND relation — before you call something monomorphic.** A constraint between arguments is as disqualifying as a type in the return.

---

*Two mechanisms, one checkable line between them. A clause is monomorphic; the moment a type variable must flow — out to the return, or across to another argument — it is an intrinsic. Decide by the property, not the preference.*
