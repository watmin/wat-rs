# Arc 059 — `:wat::core::concat` (Vec concatenation)

**Status:** opened 2026-04-26.
**Predecessor:** arc 058 (HashMap completion) — same small-arc shape; same builder direction ("if it's missing it shouldn't be").
**Consumer:** `holon-lab-trading` experiment 008 (Treasury service `loop-entry` builds a Vec<Receiver<Event>> by combining `[tick-rx]` with `broker-rxs` for the select loop). Reached for `:wat::core::concat`; not there.

Builder direction (2026-04-26, mid-experiment 008):

> what do you want concat to do .... you reached for it... we have
> ::string::concat but not a generic concat - you reaching for
> something that's not there is indicative that we should have it

The substrate has `:wat::core::string::concat` (variadic strings → string) and `:wat::core::conj` (Vec append-one) but NO Vec concat. Lisp's `append`, Clojure's `concat` (vectors), Rust's `[a,b].concat()` — all expressible primitives that wat is currently missing.

This arc adds it. ~30 LOC of Rust + ~30 LOC of tests. Mirrors arc 020's small-arc shape exactly.

---

## What's already there (no change needed)

| Op | Status | Coverage |
|----|--------|----------|
| `:wat::core::string::concat` | shipped | variadic; N strings → 1 string |
| `:wat::core::conj` | shipped | append ONE element to a Vec |
| `:wat::core::take` / `drop` | shipped | Vec slicing |
| `:wat::core::reverse` | shipped | Vec reversal |
| `:wat::core::map` / `foldl` | shipped | Vec traversal |

`string::concat` is the closest sibling — same name, same variadic shape, just for strings.

`conj` adds one element; building a concat-of-N-vecs out of conj requires a foldl over N×M elements — workable but verbose (and quadratic if naively chained). The substrate-provided concat is one allocation + N memcpy, much cleaner at any call site.

## What's missing (this arc)

| Op | Signature |
|----|-----------|
| `:wat::core::concat` | `∀T. (Vec<T>)+ → Vec<T>` (variadic, ≥1 args; all same T) |

Same variadic shape as `string::concat`. All args must unify on the same `Vec<T>`.

---

## Decisions resolved

### Q1 — Variadic vs 2-arity

**Variadic, matching `string::concat`.** Single-arg `(concat v)` returns `v` (or a clone). Multi-arg `(concat a b c d)` returns `a ++ b ++ c ++ d`. Empty case `(concat)` ambiguous on T → reject at type-check time (no way to infer T from zero args; same as `(:wat::core::vec)` ambiguity).

The variadic shape is more useful at call sites that already know exactly how many vecs they're combining — common case in service setup (e.g., `[tick-rx]` + `broker-rxs` is 2-arity; `[a]` + `b` + `c` could appear in N-broker setups).

### Q2 — Order

Left-to-right. `(concat [1,2] [3,4])` → `[1,2,3,4]`. Matches every other variadic concat in every other Lisp/ML/Rust library.

### Q3 — Ownership / values-up

Allocate a fresh Vec; copy elements from each input. Inputs unchanged. Matches the substrate's values-up discipline (per arc 020 `assoc` precedent — clone, mutate clone, return clone).

### Q4 — Type inference

Variadic + same-T = unify all args' inner T against a single fresh type variable. Mismatch → `TypeMismatch` at the offending arg position. No coercion (a `Vec<i64>` and `Vec<f64>` don't concat).

Same shape as `string::concat`'s loop:
```rust
for arg in args {
    if let Some(ty) = infer(arg, env, locals, fresh, subst, errors) {
        if unify(&ty, &string_ty, subst, env.types()).is_err() { ... }
    }
}
```
Replace `string_ty` with `Vec<T>` where T is a fresh var; unify each arg's type against it.

### Q5 — Why no HashSet `union` / Map `merge` in this arc

Different operations, different names. HashSet `union` and HashMap `merge` (or `into`) have collision-policy questions (HashSet trivially de-dupes; HashMap merge needs left/right precedence rules). Out of scope; separate arcs when consumers surface.

`concat` for Vec is unambiguous: append, no dedup, no collision. Smallest possible scope.

---

## What ships

One slice. One commit. Mirrors arc 020.

### `src/check.rs`

Dispatch arm + `infer_concat`:

```rust
":wat::core::concat" => return infer_concat(args, env, locals, fresh, subst, errors),
```

```rust
fn infer_concat(
    args: &[WatAST],
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    if args.is_empty() {
        errors.push(CheckError::ArityMismatch {
            callee: ":wat::core::concat".into(),
            expected: 1,  // "at least 1"
            got: 0,
        });
        return Some(TypeExpr::Parametric {
            head: "Vec".into(),
            args: vec![fresh.fresh()],
        });
    }
    let elem_ty = fresh.fresh();
    let vec_ty = TypeExpr::Parametric {
        head: "Vec".into(),
        args: vec![elem_ty.clone()],
    };
    for arg in args {
        if let Some(ty) = infer(arg, env, locals, fresh, subst, errors) {
            if unify(&ty, &vec_ty, subst, env.types()).is_err() {
                errors.push(CheckError::TypeMismatch {
                    callee: ":wat::core::concat".into(),
                    param: "arg".into(),
                    expected: format_type(&apply_subst(&vec_ty, subst)),
                    got: format_type(&apply_subst(&ty, subst)),
                });
            }
        }
    }
    Some(apply_subst(&vec_ty, subst))
}
```

### `src/runtime.rs`

Dispatch arm + `eval_concat`:

```rust
":wat::core::concat" => eval_concat(args, env),
```

```rust
fn eval_concat(args: &[WatAST], env: &Environment) -> Result<Value, RuntimeError> {
    let mut combined: Vec<Value> = Vec::new();
    for arg in args {
        let v = eval(arg, env)?;
        match v {
            Value::wat__core__Vec(items) => {
                combined.extend((*items).clone());
            }
            other => {
                return Err(RuntimeError::TypeMismatch { /* ... */ });
            }
        }
    }
    Ok(Value::wat__core__Vec(Arc::new(combined)))
}
```

(Adjust to match actual Vec internal representation. The pattern is "drain each arg's items into a single accumulator; return a fresh Vec".)

### Unit tests

5 tests (`tests/wat_concat.rs`), mirroring arc 020:

1. **Two-arg basic.** `(concat [1,2] [3,4])` → `[1,2,3,4]`.
2. **N-arg variadic.** `(concat [1] [2] [3] [4])` → `[1,2,3,4]`.
3. **Empty Vec args.** `(concat [] [1,2])` → `[1,2]`. `(concat [] [] [])` → `[]`.
4. **Single-arg returns clone.** `(concat [1,2,3])` → `[1,2,3]`; original unchanged.
5. **Type mismatch rejected.** `(concat [1,2] ["a","b"])` → `TypeMismatch` (i64 vs String).
6. **Empty arity rejected.** `(concat)` → `ArityMismatch`.

### Doc

- `docs/arc/2026/04/059-vec-concat/INSCRIPTION.md` post-ship.
- `docs/CONVENTIONS.md` rubric: append `concat` row under "core collection ops".
- `docs/USER-GUIDE.md`: add the entry under `:wat::core::*` Vec section.

---

## Implementation sketch

Single slice, one PR. ~80 LOC + 6 tests. Mirrors arc 020 / arc 058 shape.

```
src/check.rs:    +35 LOC  (1 dispatch arm + 1 infer fn)
src/runtime.rs:  +25 LOC  (1 dispatch arm + 1 eval fn)
tests/wat_concat.rs: +60 LOC (6 tests)
docs/arc/.../INSCRIPTION.md:  post-ship
docs/CONVENTIONS.md:  +1 LOC
docs/USER-GUIDE.md:   +3 LOC
```

**Estimated cost:** ~125 LOC. **~1 hour** of focused work.

---

## What this arc does NOT add

- **HashSet `union` / HashMap `merge`.** Different ops, different collision questions. Future arcs.
- **Vec `splice` / `insert-at`.** Different mutation shape. Future arc when consumer surfaces.
- **Lazy concat (chain iterator).** Out of scope; build it when a consumer needs lazy ops over Vec sequences.
- **Variadic `cons` / prepend.** Out of scope. `(concat [x] vs)` covers it.

---

## What this unblocks

- **`holon-lab-trading` experiment 008** (Treasury service `loop-entry`):
  ```scheme
  (:wat::core::concat
    (:wat::core::vec :EventRx tick-rx)
    broker-rxs)
  ```
  Direct call site, replaces a workaround `foldl` over broker-rxs starting from `[tick-rx]`.
- **All future programs that compose Vec inputs** — service setup with mixed-source receivers, pipeline stages combining vec-chunked outputs, regime observers building chain.regime_facts from per-source vecs (per archive's pattern).

PERSEVERARE.
