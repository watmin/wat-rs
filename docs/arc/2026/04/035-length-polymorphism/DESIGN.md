# Arc 035 — length polymorphism

**Status:** opened 2026-04-23. Cave-quest from lab arc 007. Same
shape as arc 025 (container surface unified) at a different op.

**Motivation.** Lab arc 007's fibonacci test reached for
`(:wat::core::length updated-scales)` to count entries in the
returned `Scales` (`HashMap<String, ScaleTracker>`). The
substrate refused — `length` is Vec-only per the arc 021 audit.

Arc 025 unified `get`/`assoc`/`conj`/`contains?` across
HashMap / HashSet / Vec. `length` was not part of that slice
because no caller had yet demanded it; today the fibonacci caller
surfaced the gap. Same coherence-with-siblings pressure; same
resolution.

---

## Shape

One slice. Promote `:wat::core::length` from Vec-only to
polymorphic over the three containers:

```
:wat::core::length :Vec<T>       -> :i64    (existing — elements)
:wat::core::length :HashMap<K,V> -> :i64    (new — entries)
:wat::core::length :HashSet<T>   -> :i64    (new — elements)
```

Implementation mirrors arc 025's `contains?`:

- `runtime.rs` — rename `eval_vec_length` → `eval_length`;
  dispatch on `Value` variant (`Vec` / `wat__std__HashMap` /
  `wat__std__HashSet`).
- `check.rs` — remove the narrow Vec-typed scheme; add
  `infer_length` polymorphic function; add dispatch arm in
  `infer_list`.

---

## Why Tuple is excluded

`:(A,B,C)` is a type with known arity. `(length (:wat::core::tuple
a b c))` can only ever return 3 — that's already knowable from the
type signature at check time. Adding runtime tuple-length would
permit generic tuple walking that the rest of the language
doesn't otherwise support (no `nth` over tuple positions, no
tuple iteration). Stdlib-as-blueprint: wait for a caller that
needs it. None has surfaced.

The table the substrate owes its users:

| Op | HashMap<K,V> | HashSet<T> | Vec<T> | Tuple |
|---|---|---|---|---|
| `length` | entries | elements | elements | — (structural) |
| `get` | `Option<V>` by key | `Option<T>` by element | `Option<T>` by index | — |
| `assoc` | new-map | illegal | new-vec (replace) | — |
| `conj` | illegal | new-set | new-vec (append) | — |
| `contains?` | `bool` | `bool` | `bool` | — |

Tuple's dashes are category markers — the op is not meaningful
at that container, not "coming later."

---

## Non-goals

- **No string::length change.** Arc 007 (wat-tests-wat) shipped
  `:wat::core::string::length` separately. That's a different
  primitive at a different namespace; already works.
- **No empty? polymorphism.** `:wat::core::empty?` is Vec-only
  today. No caller has demanded the polymorphic form. Defer.
- **No integration with the scaled-linear path.** The fibonacci
  test will use `(:wat::core::length updated)` directly after
  this arc ships.
