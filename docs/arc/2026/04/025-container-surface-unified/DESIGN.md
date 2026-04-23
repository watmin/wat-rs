# Arc 025 — unified get/assoc/conj/contains? surface

**Status:** opened 2026-04-22.

**Motivation.** 058-026-array's spec promised `(get vec i)` with
an integer index returning `:Option<T>`, but only HashMap +
HashSet support shipped; Vec indexing was a latent gap. Phase
3.4's rhythm encoder was the first real caller to surface it.

While closing the `get` gap, the symmetry question arose:
`get` works on three containers but `assoc` / `conj` /
`contains?` were each narrow. One name per operation, but the
container dispatch was inconsistent. Cave-quest scope expanded
to make the whole collection surface coherent.

Builder framing:

> you path is clear to make get and assoc great?
>
> contains? should support hash-set?..
>
> and member? can go in favor or contains? ...ya

---

## The unified surface

| Op | HashMap&lt;K,V&gt; | HashSet&lt;T&gt; | Vec&lt;T&gt; |
|---|---|---|---|
| `get`        | `Option<V>` by key        | `Option<T>` by element       | `Option<T>` by index       |
| `assoc`      | new-map (add/replace)     | **illegal** (use conj)       | new-vec (replace at index) |
| `conj`       | **illegal** (use assoc)   | new-set (add element)        | new-vec (append)           |
| `contains?`  | `bool` by key             | `bool` by element            | `bool` by index            |

Tuples are set aside — positional heterogeneous records don't
fit the "key → value" shape. `(:wat::core::first/second/third)`
already handles positional access.

## Why this is the coherent shape

- **get** asks: "can I retrieve something at this location?"
  Works on any container with a location concept (key, element,
  index).
- **assoc** asks: "can I replace/insert a value at this
  location?" Works on containers with key-value pairing
  (HashMap) or positional overwrite (Vec). Illegal on HashSet
  because sets are pure membership — no value to associate.
- **conj** asks: "can I add a new element to this growing
  collection?" Works on Vec (append) and HashSet (add). Illegal
  on HashMap because HashMap needs key + value — `assoc` is the
  verb for that.
- **contains?** asks: "is this location in the container?" Works
  on any container with a location concept (same coverage as
  `get`).

`get` and `contains?` cover the SAME containers (they're both
locate-only). `assoc` and `conj` cover OPPOSING containers
(assoc for key-value, conj for pure-element). The four verbs
cover the three container semantics cleanly.

## Retirements

- `:wat::std::member?` — redundant with polymorphic `contains?`.
  All callers migrate; no back-compat shim.

---

## Implementation

### Runtime (`src/runtime.rs`)

- `eval_get` — add Vec arm (i64 index → Option<T> via bounds check).
- `eval_assoc` — add Vec arm (i64 index, bounds-check, clone-set).
- `eval_conj` — add HashSet arm (canonical key + insert).
- `eval_contains_q` (renamed from `eval_hashmap_contains`) —
  dispatches on HashMap (has key), HashSet (has element), Vec
  (valid index).
- `eval_hashset_member` — deleted. Its dispatch arm in the
  evaluator removed.

### Check (`src/check.rs`)

- `infer_get` — add Vec arm (unify key with :i64, return Option<T>).
- `infer_assoc` — add Vec arm (unify key with :i64, unify value
  with T, return Vec<T>).
- `infer_conj` — NEW function (replaces narrow scheme). Dispatches
  on Vec and HashSet. Illegal on HashMap.
- `infer_contains_q` — NEW function. Dispatches on HashMap /
  HashSet / Vec. All three return :bool.
- Narrow scheme registrations for `contains?` and `conj`
  removed; replaced by dispatch arms that call the polymorphic
  inferrers.
- `:wat::std::member?` registration deleted.

---

## Tests

- Each polymorphic arm gets coverage in `src/runtime.rs`:
  - `get` on Vec (hit + out-of-range → None + negative → None).
  - `assoc` on Vec (replace, out-of-range errors, values-up
    preserves input).
  - `conj` on HashSet (add, idempotent re-add, cross-type
    distinctness preserved).
  - `contains?` on all three (hit / miss per container).
- Existing HashMap / HashSet / Vec tests continue to pass with
  no shape change.
- Two existing tests (`hashset_member_present_and_absent`,
  `hashset_int_and_string_keys_distinct`) migrate from
  `:wat::std::member?` to `:wat::core::contains?` — same
  assertions, new predicate name.

---

## Doc sweep

- `docs/CONVENTIONS.md` — `:wat::core::*` table row lists the
  unified surface.
- `docs/USER-GUIDE.md` — section on collections names all three
  containers + four primitives with the legal/illegal matrix.
- `docs/arc/2026/04/005-stdlib-naming-audit/INVENTORY.md` —
  `:wat::std::member?` row marked RETIRED; `:wat::core::contains?`
  row updated to show polymorphic dispatch.
- `holon-lab-trading/docs/proposals/.../FOUNDATION-CHANGELOG.md`
  — row covering arc 025's four primitives.

---

## Non-goals

- Tuple support for get/assoc/contains?. Different semantic
  (positional heterogeneous), handled by `first/second/third`.
- Deeper nesting (get-in / assoc-in style). Stdlib-as-blueprint
  — ship when a real caller demands.
- Polymorphic conj on HashMap (Clojure-style 2-tuple entries).
  Would require MapEntry concept that wat doesn't have; no
  caller demand.
