# NOTE — stdlib tooling must home under a top-level `wat.<home>/` namespace

**Surfaced 2026-06-11** during the arc-251 / 4.2b feasibility crawl (a `parse-i64` reach-stumble
→ a homing audit). Builder direction: *"we're going to move string and uuid to their own
wat.home/fn namespace"* and *"the thing i want to guard is [proper] wat.* prefixes for stdlib
tooling"* — i.e. each stdlib family gets a clean **top-level** `wat.<home>/fn` namespace, not a
burial under `wat.core.*` and not a stranding in the dead `wat.std.*`. Arc 109 is the perpetual
mass-name-refactor home, so this is **noted here, not fixed inline** (the rehome rides 109).

## The homing debt (grounded against the dispatch surface, 2026-06-11)

| Family | Current home | Defect | Target |
|---|---|---|---|
| **string** (12 verbs: `subs` `split` `concat` `join` `length` `trim` `contains?` `starts-with?` `ends-with?` `to-i64` `to-f64` `to-bool`) | `:wat::core::string::` = `wat.core.string/` | buried 3-level; **no `wat/string.wat` home file** (pure-Rust `string_ops.rs`) | `wat.string/` |
| **uuid** (5: `Uuid/v4` `Uuid/v5` `Uuid/from-string` `Uuid/to-string` `Uuid/nil`) | `:wat::core::Uuid/` = `wat.core/Uuid/…` | type-method style, not its own namespace (also in `string_ops.rs`) | `wat.uuid/` |
| **math** (30 refs) | `:wat::std::math::` | **`wat.std` = the namespace THIS arc kills** — directory died, verb namespace survived | `wat.math/` |
| **stat** (9 refs) | `:wat::std::stat::` | same dead namespace | `wat.stat/` |
| **list** (9 refs) | `:wat::std::list::` | same dead namespace — *and* a `wat/list.wat` home already exists → possible split-brain to reconcile | `wat.list/` |

The audit confirmed **no non-`wat::` escapees** (every dispatched head is `:wat::<ns>::…`); the
debt is entirely *mis-homing* under `wat.core.*` / `wat.std.*`, not un-prefixed verbs.

## The guard (the rule to uphold going forward)

New stdlib tooling homes under a **proper top-level `wat.<home>/`** namespace. Do NOT:
- bury a family under `wat.core.<family>/` (the `wat.core.string` mistake), or
- add to the dead `wat.std.<family>/` namespace.

A future enforcement option (extirpare top rung — make the mis-home un-expressible): a build
check that fails if a dispatched stdlib verb's namespace isn't an allow-listed top-level
`wat.<home>`. Not built now; noted as the structural version of the guard.

## Rider: the faithful `parse-<scalar>` family (folds into the string rehome)

A `parse-i64` reach-stumble surfaced that the capability ALREADY exists as
`:wat::core::string::to-i64` / `to-f64` (`runtime.rs:6454`, `s.parse::<i64>().ok()` →
`Option<i64>`) — it was just absent under the **name an LLM reaches for**. The faithful name is
`parse-i64` / `parse-f64` (Rust scalar names, NOT Clojure's `parse-long`/`parse-double`: wat's
scalar model is Rust's, and `parse-<scalar>` scales uniformly to the full set coming — `parse-i8`
… `parse-u128`, `parse-f32` — where `parse-long` has no home). When string rehomes to
`wat.string/`, add the faithful aliases there:

```
wat.string/parse-i64  → wat.string/to-i64    (Option<i64>; None on parse failure)
wat.string/parse-f64  → wat.string/to-f64    (Option<f64>)
```

**Contract (probe-verified shape, from the now-removed `tests/probe_arc251_parse_scalar.rs`):**
`(parse-i64 "42") → Some 42` · `(parse-i64 "-7") → Some -7` (Rust `i64::from_str` accepts a
leading sign) · `(parse-i64 "nope") → None` (garbage → `None`, never an error) · `(parse-f64
"3.5") → Some 3.5` · `(parse-f64 "") → None`. The alias mechanism is confirmed: `defalias`
Case 2 (`register_defalias`, `runtime.rs`) forwards both the type scheme and runtime dispatch for
an intrinsic target (the `concat → Vector/concat` precedent), so each is a one-line `defalias` in
the string home — zero Rust. `string::to-i64` has **zero** `.wat`/test callers (only the macro
purity allow-list references it), so retiring the old name at the hard-cut is nearly free.

Cross-ref: `feedback_absence_claim_needs_all_forms` (the `to-i64` vs `to-int` grep blind spot that
made me falsely declare `parse-i64` "missing"; the grounding caught it before a duplicate build).
