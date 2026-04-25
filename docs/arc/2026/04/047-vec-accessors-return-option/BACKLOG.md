# wat-rs arc 047 — Vec accessors / aggregates return Option — BACKLOG

**Shape:** five slices. Substrate change + new primitives + sweep
+ docs + INSCRIPTION. The polymorphism shift on `first/second/third`
is the load-bearing change; new primitives ship alongside.

---

## Slice 1 — runtime + check changes for first/second/third

**Status: ready.**

`src/runtime.rs` — `eval_positional_accessor`:
- Tuple branch unchanged (returns cloned T directly).
- Vec branch wraps in Option: `Value::Option(Arc::new(Some(...)))`
  for in-range index, `Value::Option(Arc::new(None))` for out-of-range
  (drops the runtime-error path).

`src/check.rs` — `infer_positional_accessor`:
- Tuple branch unchanged (returns element type).
- Vec branch returns `Option<T>` instead of `T`.

The polymorphic dispatch already inspects argument type; only
the per-branch return / wrap shape changes.

**Sub-fogs:**
- **1a — find-or-die-default for the Some/None Value wrapping.**
  Already exists per Option support in scalar conversions
  (`eval_string_to_i64` etc). Mirror that pattern.

## Slice 2 — new primitives

**Status: ready** (independent of slice 1; can run in parallel).

Add to `src/runtime.rs` dispatch + helper functions + tests:

- `:wat::core::last` → `eval_vec_last(args, env, sym)`
- `:wat::core::find-last-index` →
  `eval_find_last_index(args, env, sym)`
- `:wat::core::f64::max-of` →
  `eval_f64_reduce(args, env, sym, "max-of", |a, b| a.max(b))`
- `:wat::core::f64::min-of` →
  `eval_f64_reduce(args, env, sym, "min-of", |a, b| a.min(b))`

Add to `src/check.rs`:

- `last : ∀T. Vec<T> -> Option<T>`
- `find-last-index : ∀T. Vec<T> × fn(T)->bool -> Option<i64>`
- `f64::max-of`, `f64::min-of : Vec<f64> -> Option<f64>`

Inline tests for each:
- `last`: empty → None; single → Some(x); multiple → Some(last)
- `find-last-index`: empty → None; no match → None; one match →
  Some(i); multiple matches → Some(rightmost)
- `f64::max-of` / `min-of`: empty → None; single → Some(x);
  multiple → Some(extreme); equal → Some(value)

**Sub-fogs:**
- **2a — find-last-index lambda dispatch.** Mirror `eval_vec_foldl`'s
  pattern for unwrapping `Value::wat__core__lambda` and applying.
- **2b — `eval_f64_reduce` factor.** Both max-of and min-of share
  shape (require Vec<f64>, fold with binary op, return Option<f64>).
  Single helper parameterized by the op closure.

## Slice 3 — sweep wat-rs callers

**Status: gated by slice 1** (compiles only after polymorphism shift
lands).

Each existing `(:wat::core::first|second|third xs)` callsite where
`xs` is a Vec: migrate to handle `Option<T>`.

Sweep sources:
- `wat-tests/**/*.wat`
- `crates/wat-lru/**/*.{wat,rs}`
- `tests/*.rs` — embedded wat strings
- `examples/**/*.wat`
- `wat/holon/*.wat`, `wat/std/*.wat`
- `src/**/*.rs` — embedded wat strings (rare)

Survey-found likely Vec sites (need verification per call):
- `wat-tests/std/service/Console.wat:49` — `(:first stdout)`
- `tests/wat_hermetic_round_trip.rs:86` — `(:first lines)`
- `tests/wat_dispatch_e1_vec.rs:63,79` — explicit Vec tests
- `tests/wat_names_are_values.rs:204` — `(:first collected)`
- `crates/wat-lru/wat-tests/lru/CacheService.wat:49,85` — likely
  `(:first stdout-lines)`-style

Migration pattern: wrap with `match`, error path uses a sentinel
or `assertion-failed!` for "should never be empty" cases.

```scheme
;; Before
((line :String) (:wat::core::first stdout))

;; After
((line :String)
  (:wat::core::match (:wat::core::first stdout) -> :String
    ((Some s) s)
    (:None "")))   ;; or assertion-failed!, depending on context
```

**Sub-fogs:**
- **3a — `cargo test` is the gate.** After slice 1+2 land, run
  `cargo test --release` and follow type-error / runtime-error
  trail to find every Vec callsite. Substrate doesn't ship green
  until sweep complete.
- **3b — Tuple sites stay untouched.** Verify by inspecting
  context — if the binding is to a tuple type, the call still
  returns T. Only Vec sites move.

## Slice 4 — docs sync

**Status: ready** (independent).

`docs/USER-GUIDE.md`:
- §15 Forms appendix — update `:wat::core::first` / `second` /
  `third` row to note polymorphism: T on Tuple, Option<T> on Vec.
  Add row for `:wat::core::last` (Option<T> on Vec).
  Add row for `:wat::core::find-last-index`.
  Update the f64 row to include `max-of` / `min-of`.

`docs/CONVENTIONS.md`: minor — the §1 namespace description
mentions positional accessors briefly; no enumeration to update.

`README.md`: no per-form enumeration; no edit.

## Slice 5 — INSCRIPTION + cross-refs

**Status: obvious in shape** (once slices 1 – 4 land).

- `docs/arc/2026/04/047-vec-accessors-return-option/INSCRIPTION.md`.
  Records: the polymorphism shift framing; the new primitives;
  the sweep count; the namespace-consistency rule extension
  (Vec aggregates return Option).
- Lab repo `docs/proposals/.../058 FOUNDATION-CHANGELOG` row.
- Lab arc 018 unblocks; resumes with substrate-direct calls
  (and lab-side caller sweep as part of consuming the new
  primitives).

---

## Working notes

- Opened 2026-04-24 mid-arc-018 sketch. Builder asked the
  framing question that surfaced the polymorphism wart; arc 047
  fills the gap before arc 018 ships.
- Substrate breaking change with bounded blast radius (most
  callers are on Tuple, unaffected). Sweep is mechanical.
- The "natural form" reflex held — the lab needed `last`,
  `find-last-index`, `max-of`, `min-of`. Substrate didn't
  have them. Arc 047 ships them with the right convention from
  the start.
