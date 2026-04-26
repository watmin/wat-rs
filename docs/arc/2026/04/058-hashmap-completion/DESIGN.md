# Arc 058 — HashMap surface completion (`dissoc` / `keys` / `values` / `empty?`)

**Status:** opened 2026-04-26.
**Predecessor arcs:** [`docs/arc/2026/04/020-assoc/`](../020-assoc/DESIGN.md), [`docs/arc/2026/04/021-core-std-audit/`](../021-core-std-audit/DESIGN.md).
**Consumer:** `holon-lab-trading` experiment 008 (Treasury program). The Treasury holds `HashMap<i64, Paper>` keyed by paper-id; per-tick `check-deadlines` needs to iterate every Active paper to find expirations; resolution removes resolved entries; metrics emit count of active papers. Without iteration + remove, the HashMap is half-built — assoc-only.

Builder direction (2026-04-26, mid-experiment 008 layout):

> let's go make wat core better - if its missing it shouldn't be -
> let's get an arc written for us to go build

Arc 020 shipped `assoc` (insert/update). Arc 021 moved the
HashMap family to `:wat::core::*` per the rubric. Both arcs
explicitly deferred `dissoc` / `keys` / `values` / `len` /
`is-empty?` as "natural companions / future arc."

This arc closes the gap. Three new ops + one polymorphism
extension. Mechanical; mirrors `infer_assoc` shape from arc 020.

---

## What's already there (no change needed)

| Op | Status | Coverage |
|----|--------|----------|
| `:wat::core::HashMap` constructor | shipped (arc 021) | literal `(HashMap :(K,V) k1 v1 ...)` |
| `:wat::core::assoc` | shipped (arc 020) | insert/update; values-up |
| `:wat::core::get` | shipped (arc 021) | polymorphic over Vec/HashMap/HashSet |
| `:wat::core::contains?` | shipped (arc 025) | polymorphic over Vec/HashMap/HashSet |
| `:wat::core::length` | shipped (arc 035) | polymorphic over Vec/HashMap/HashSet |

## What's missing (this arc)

| Op | New / extension | Signature |
|----|-----------------|-----------|
| `:wat::core::dissoc` | new | `∀K, V. HashMap<K,V> × K → HashMap<K,V>` (values-up; missing key is no-op) |
| `:wat::core::keys` | new | `∀K, V. HashMap<K,V> → Vec<K>` (order unspecified) |
| `:wat::core::values` | new | `∀K, V. HashMap<K,V> → Vec<V>` (order unspecified) |
| `:wat::core::empty?` | polymorphism extension | currently `Vec<T> → bool`; extend to `HashMap<K,V> → bool` and `HashSet<T> → bool` (mirrors `length` polymorphism shape) |

Four items. Three ops + one polymorphism widening. Same call
sites; same shape as the existing HashMap surface.

---

## Decisions resolved

### Q1 — `dissoc` semantics

`(dissoc m k) → m'` — values-up. Returns a NEW `HashMap` without
the key; original unchanged. If `k` was not present, returns a
clone of the input (no error). Mirrors Clojure's `dissoc`.

Signature: `∀K, V. HashMap<K,V> × K → HashMap<K,V>`.

### Q2 — `keys` and `values` materialize a `Vec`

wat doesn't have lazy iteration on HashMap. `keys` and `values`
build a `Vec<K>` / `Vec<V>` of length `len(m)` and return it.
Caller uses standard Vec ops (`foldl`, `map`, `filter`) from
there.

Signatures:
- `keys`:   `∀K, V. HashMap<K,V> → Vec<K>`
- `values`: `∀K, V. HashMap<K,V> → Vec<V>`

**Order is unspecified** (HashMap iteration order is whatever
Rust's `std::collections::HashMap` gives, which depends on hash
randomization). Callers that need deterministic order sort the
Vec post-call.

### Q3 — Why no `contains-key?`

`contains?` is already polymorphic over `HashMap` (arc 025): `(contains? m k) → bool` checks key presence. Adding a Clojure-named alias `contains-key?` would be redundant. Stick with the existing op; document the polymorphism.

### Q4 — Why extend `empty?` not add `is-empty?`

`empty?` already exists for `Vec<T>`. Adding a HashMap-specific predicate would split a single concept across two names. Extend `empty?`'s polymorphism to cover `HashMap<K,V>` and `HashSet<T>` (same call site, container-aware return) — matches `length` and `contains?`'s polymorphism shape.

`empty?` is currently registered as a fixed scheme `Vec<T> → bool` (`src/check.rs:5717`). Convert to `infer_empty_q` dispatch arm following `infer_length`'s template.

### Q5 — Why no HashSet completion in this arc

HashSet has the same gaps (no remove/keys/etc). Treasury — the consumer driving this arc — uses HashMap, not HashSet. HashSet completion (`disjoin`/equivalent) is a separate small arc when a consumer surfaces. Same scoping discipline as arc 020 (just `assoc`, not the whole missing surface).

The `empty?` polymorphism extension DOES include HashSet, since polymorphizing without HashSet would leave the predicate inconsistent.

### Q6 — Mutex-free, values-up

All three new ops follow arc 020's template:
- Clone the inner `std::collections::HashMap` (cheap shallow copy of the buckets vector — Rust's HashMap is heap-allocated; the clone is `O(N)` in entries, `O(1)` per bucket).
- Mutate the clone (remove for `dissoc`; gather for `keys`/`values`).
- Wrap in a new `Value::wat__std__HashMap` and return.

Original input is unchanged. No interior mutability. No cross-thread surface — HashMap is local to the thread that constructs it (per arc 020 INSCRIPTION).

---

## What ships

One slice. One commit. Following arc 020's shape.

### `src/check.rs`

Three new dispatch arms in the early-return match (near
`infer_assoc`):

```rust
":wat::core::dissoc" => return infer_dissoc(args, env, locals, fresh, subst, errors),
":wat::core::keys"   => return infer_keys(args, env, locals, fresh, subst, errors),
":wat::core::values" => return infer_values(args, env, locals, fresh, subst, errors),
```

Three `infer_*` functions following `infer_assoc` / `infer_get`
template (~30 LOC each — arity check, container inference, shape
match for HashMap, type unification, return type construction).

`empty?` polymorphism extension:
- Remove the fixed scheme registration at `src/check.rs:5717`.
- Add `":wat::core::empty?"` dispatch arm in the early-return match.
- Add `infer_empty_q` following `infer_length`'s template (Vec → bool, HashMap → bool, HashSet → bool, _ → TypeMismatch).

### `src/runtime.rs`

Three new dispatch arms + `eval_*` functions:

```rust
":wat::core::dissoc" => eval_dissoc(args, env),
":wat::core::keys"   => eval_keys(args, env),
":wat::core::values" => eval_values(args, env),
```

Each ~15 LOC — extract HashMap from arg[0], call corresponding `std::collections::HashMap` method (`remove` / `keys` / `values`), wrap result in `Value::wat__std__HashMap` (for dissoc) or `Value::wat__core__Vec` (for keys/values).

`empty?` runtime extension: existing dispatch arm broadens to inspect container type before delegating.

### Unit tests

5 tests per new op (mirroring arc 020), plus 3 for `empty?`
polymorphism extension:

**`dissoc`** (`tests/wat_dissoc.rs`):
1. Removes existing key, returns new map without it
2. Missing key is no-op (returns clone of input)
3. Preserves original (values-up proof — original still has the key)
4. Non-HashMap arg rejected with `TypeMismatch`
5. Arity mismatch rejected

**`keys`** (`tests/wat_keys.rs`):
1. Returns Vec<K> of correct length matching map size
2. Empty map returns empty Vec
3. Vec contents match the map's keys (order-agnostic membership check)
4. Non-HashMap arg rejected
5. Arity mismatch rejected

**`values`** (`tests/wat_values.rs`):
1. Same shape as keys, checking V instead of K
2-5. Mirror keys.

**`empty?` polymorphism** (extend `tests/wat_empty.rs` or similar):
1. `(empty? hm)` on empty HashMap returns true
2. `(empty? hm)` on non-empty HashMap returns false
3. `(empty? hs)` on HashSet works the same way

### Doc

- `docs/arc/2026/04/058-hashmap-completion/INSCRIPTION.md` post-ship.
- `docs/CONVENTIONS.md` rubric table — append the four new entries to the "core collections" row.
- `docs/USER-GUIDE.md` (or equivalent surface table) — entries under `:wat::core::*` HashMap section.

---

## Implementation sketch

Single slice, expected one PR. ~150 LOC of Rust + ~120 LOC of tests + 2 doc files.

```
src/check.rs:    +90 LOC  (3 infer_* fns + 1 dispatch shape rewrite for empty?)
src/runtime.rs:  +60 LOC  (3 eval_* fns + 1 dispatch extension)
tests/wat_dissoc.rs:  +50 LOC
tests/wat_keys.rs:    +50 LOC
tests/wat_values.rs:  +50 LOC
tests/wat_empty.rs (extend): +30 LOC (3 new HashMap/HashSet tests)
docs/arc/.../INSCRIPTION.md: post-ship
docs/CONVENTIONS.md: +4 LOC
docs/USER-GUIDE.md: +12 LOC
```

**Estimated cost:** ~330 LOC total. **~3 hours** of focused work.
Same shape as arc 020 (~half a day, finished in one slice).

---

## What this arc does NOT add

- **HashMap merge / union / intersection.** Compositional ops; multi-arity. Future arc when a consumer surfaces.
- **HashSet completion.** `disjoin` (analog to dissoc for sets) + iteration. Separate small arc; same template as this one.
- **Lazy iteration / iterator type.** `keys` and `values` materialize a Vec. A separate Iterator-shaped substrate primitive is out of scope; build it when a consumer needs lazy ops over HashMap.
- **Insertion-order maps.** Rust's HashMap iteration order is unspecified; callers that need order sort the result.
- **Capacity hints / `with-capacity`.** Defer until measurement shows allocation overhead matters.
- **Mut-cell HashMaps.** Substrate is values-up; mut would be a different surface (separate arc, maybe never).

---

## What this unblocks

- **`holon-lab-trading` experiment 008 (Treasury program).** Treasury holds `HashMap<i64, Paper>`; per-tick `check-deadlines` calls `(values m)` → foldl over Active papers → `(dissoc m k)` for each resolved id (or set state to `Violence` and leave the entry; the new shape lets us pick).
- **`HashMap<i64, ProposerRecord>`** in Treasury — `(values m)` for aggregating proposer stats; `(empty? m)` for the "no brokers yet" guard.
- **Future programs holding HashMap state** — broker prediction caches keyed by feature hash, observer recalibration trackers keyed by lens-name, regime-state caches keyed by phase-id, etc. All of them get the full toolbox.
- **Arc 030 slice 2 (encoding cache)** — when the predictor's encode cache lands, `(length cache) -> :i64` (already shipped) + `(empty? cache)` (this arc) round out the cache-stats telemetry surface.

---

PERSEVERARE.
