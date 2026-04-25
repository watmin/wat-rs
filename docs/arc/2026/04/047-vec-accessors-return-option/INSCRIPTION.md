# wat-rs arc 047 — Vec accessors / aggregates return Option — INSCRIPTION

**Status:** shipped 2026-04-24. Second wat-rs arc post-known-good.
Lab arc 018 surfaced four substrate gaps while sketching the
window-vocab port; the builder caught a deeper question about
`first`'s shape:

> "should first errory on empty?... in ruby [].first -> nil -
> why isn't our first Option<T>?"

The arc answers: **Vec accessors and aggregates return Option to
honestly signal empty/no-match.** Tuple accessors stay `T`
(defined-by-type). The two contexts split cleanly along the type
boundary.

Three durables:

1. **Polymorphism shift on `first/second/third` for Vec.** Vec
   branch returns `Option<T>` instead of `T` (the Haskell `head`
   wart, retired). Tuple branch unchanged. The polymorphism
   already split on type at compile time; only the Vec branch's
   return shape changes.
2. **Four new natural-form primitives** — `last`,
   `find-last-index`, `f64::max-of`, `f64::min-of`. Each was
   identified by writing the natural form of standard.wat and
   discovering the gap. Substrate-worthy because every
   functional language ships them.
3. **The "natural-form-then-promote" methodology validated.**
   The lab sketched what felt right; gaps surfaced; substrate
   uplift filled them. Arc 047 names this as the standing rhythm
   for substrate growth.

**Design:** [`DESIGN.md`](./DESIGN.md).
**Backlog:** [`BACKLOG.md`](./BACKLOG.md).

12 new lib tests + sweep across 7 wat-rs callsites. 610 → 622
lib tests; full integration suite green; zero clippy.

---

## What shipped

### Slice 1 — runtime + check changes for first/second/third

`src/runtime.rs` — `eval_positional_accessor`:
- Tuple branch unchanged (returns cloned T directly).
- Vec branch wraps in `Value::Option` — `Some` for in-range,
  `None` for out-of-range. Drops the runtime-error path for
  Vec.

`src/check.rs` — `infer_positional_accessor`:
- Tuple branch unchanged.
- Vec branch returns `Option<T>` instead of `T`.

The polymorphism is still ad-hoc dispatch on argument type; only
the Vec branch's return shape changed.

### Slice 2 — new primitives

`src/runtime.rs`:
- `:wat::core::last` → `eval_vec_last`. Returns `Option<T>`
  (`Some` for non-empty, `None` for empty).
- `:wat::core::find-last-index` → `eval_vec_find_last_index`.
  Returns `Option<i64>`. Iterates the Vec applying the
  predicate; `Some(i)` for the rightmost matching index, `None`
  otherwise.
- `:wat::core::f64::max-of` / `min-of` → both via
  `eval_f64_reduce` (parameterized on the binary fold op).
  Returns `Option<f64>` — `None` for empty Vec.

`src/check.rs` — type registrations for all four. Generic in T
where appropriate; `find-last-index`'s second parameter is
`fn(T) -> :bool`.

12 new lib tests added in the `tests` module:
- `first_polymorphic_on_vec` (updated for new shape)
- `first_on_empty_vec_returns_none` (new — proves the change)
- `second_polymorphic_on_vec` (updated)
- `third_on_vec` (updated)
- `third_on_short_vec_returns_none` (new — out-of-range)
- `last_returns_some_for_non_empty` / `last_returns_none_for_empty`
- `find_last_index_returns_rightmost_match`
- `find_last_index_returns_none_for_no_match`
- `find_last_index_returns_none_for_empty`
- `f64_max_of_picks_largest` / `f64_min_of_picks_smallest`
- `f64_max_of_singleton_returns_single`
- `f64_max_of_empty_returns_none` / `f64_min_of_empty_returns_none`

### Slice 3 — sweep wat-rs callers

Seven Vec-callsites migrated to handle the new `Option<T>`
return:

| Site | Pattern | Migration |
|---|---|---|
| `wat/std/stream.wat:332` | `(:first items)` after `(empty? items)` check | `match` with unreachable :None sentinel |
| `wat/std/stream.wat:643` | same shape | same |
| `wat/holon/Sequential.wat:32` | `(:first positioned)` as foldl init | `match` with sentinel for empty-Sequential |
| `wat-tests/std/service/Console.wat:49` | `(:first stdout)` test assertion | `match` with `""` default |
| `tests/wat_core_forms.rs:137,196` | `(:first captured)` in embedded wat | `match` with `""` default (×2) |
| `tests/wat_dispatch_e1_vec.rs:63,79` | `(:first <vec-fn-result>)` in embedded wat | `match` with `-1` sentinel (×2) |
| `tests/wat_hermetic_round_trip.rs:86` | `(:first lines)` test assertion | `match` with `""` default |
| `tests/wat_names_are_values.rs:204` | `(:first collected)` test assertion | `match` with `-1` default |
| `tests/wat_run_sandboxed_ast.rs:67` | `(:first lines)` test assertion | `match` with `""` default |
| `tests/wat_variadic_defmacro.rs:51` | `(:first (vec-of ...))` in embedded wat | `match` with `-1` default |

Total: ~10 callsites across 8 files. Most callers KNEW the Vec
was non-empty (just-checked or constructor-fresh) — the
unreachable :None branch uses a sentinel value typed to match.

**The cost the polymorphism shift pays**: every known-safe
caller adds a match-with-sentinel. Trade-off: type honesty over
caller convenience. Caller can prove safety locally, but the
type system can't, so the match exists.

### Slice 4 — docs sync

`docs/USER-GUIDE.md` §15 Forms appendix:
- `first / second / third` row updated to note polymorphism:
  tuple → T; Vec → `Option<T>`.
- New row for `last` (Vec → `Option<T>`).
- New row for `find-last-index`.
- New row for `f64::max-of` / `min-of` (Vec<f64> → `Option<f64>`).

CONVENTIONS.md unchanged (summary-level).
README.md unchanged (no per-form enumeration).

### Slice 5 — INSCRIPTION + cross-refs (this file)

Plus:
- Lab repo `058 FOUNDATION-CHANGELOG` row documenting wat-rs arc
  047. Lab arc 018 unblocks; resumes with substrate-direct calls.

---

## The polymorphism shift in practice

### Before (wat-rs known-good-2026-04-24 + arc 046)

```scheme
(((line :String) (:wat::core::first stdout)))   ;; line :String, errors if stdout empty
```

### After (arc 047)

```scheme
(((line :String)
  (:wat::core::match (:wat::core::first stdout) -> :String
    ((Some s) s)
    (:None ""))))   ;; line :String, signature now requires unwrap
```

The verbosity is real. It's the price of type honesty — Rust's
`vec.first()` returns `Option<&T>`, every caller in Rust pays
the same `.unwrap_or("")` / `match` / `if let Some` cost. wat
now matches that convention.

For known-safe sites, the `:None` arm is unreachable; a
sentinel value of the right type satisfies the type checker
without runtime risk (the branch is never taken). Naming the
sentinel honestly (e.g., `"Sequential-empty-input"` for
Sequential's Atom) is a documentation hint that the caller
believes the case is unreachable.

## Why this is a substrate-shape decision, not a stylistic one

The substrate's existing `get` returns `Option<T>` for Vec, but
`first/second/third` returned raw `T` with runtime error. Two
conventions coexisted, neither winning. Arc 047 picks a winner:
**positional accessors over Vec return Option, period.**

The user direction "writing the forms we find natural and
realizing they should be in the core lang" applies inside-out
here too: `[].first → nil` (Ruby), `vec.first() → Option<&T>`
(Rust), `(first '()) → nil` (Clojure). The natural form is
`Option<T>`. Substrate now matches.

Tuple positional accessors stay `T` because tuple arity is
type-known — the empty case is impossible at compile time. No
runtime risk, no need for Option.

## The natural-form-then-promote rhythm

Arc 047 is the second arc this session (after arc 046) that
shipped substrate primitives surfaced by the lab writing the
natural form. The pattern:

1. Lab writes vocab as if all natural primitives exist.
2. Type checker / runtime fails for the missing ones.
3. Open a substrate arc to fill the gap with the right shape.
4. Lab consumes substrate-direct.

This rhythm is faster than "design the substrate first, hope it
fits later." Each substrate addition is justified by an
in-flight caller. Each addition arrives with its first user
verifying the shape.

Arc 046 used the pattern for `f64::max/min/abs/clamp` +
`math::exp`. Arc 047 uses it for `last`, `find-last-index`,
`f64::max-of`, `f64::min-of` (plus the polymorphism shift on
first/second/third — which wasn't a missing primitive but a
shape correction surfaced by the same reflex).

## Sub-fog resolutions

- **1a — Option Value wrapping.** Mirrored existing
  `eval_string_to_i64` etc. pattern with
  `Value::Option(Arc::new(...))`.
- **2a — find-last-index lambda dispatch.** Followed
  `eval_vec_foldl` precedent for unwrapping `Value::wat__core__lambda`
  and applying via `apply_function`.
- **2b — eval_f64_reduce shape.** Single helper parameterized on
  the binary fold op (`f64::max` / `f64::min`). Initializes from
  the first element if non-empty; returns `None` for empty Vec.
- **3a — cargo test as the gate.** Arc 047 ran cargo test ~6
  times following the type/runtime error trail to catch every
  Vec callsite. Each iteration narrowed the failure set; final
  iteration green.

## Count

- New primitives: **4** (`last`, `find-last-index`,
  `f64::max-of`, `f64::min-of`).
- Polymorphism shift: **3 forms** (`first`, `second`, `third`)
  — Vec branch only; Tuple unchanged.
- Lib tests: **598 → 622** (arc 046 added 12, arc 047 added 14;
  net +24 from known-good).
- Integration suite: green across all 34+ test crates.
- Clippy: **0** warnings.
- Docs surface: 1 row updated (`first/second/third`) + 3 rows
  added (`last`, `find-last-index`, `f64::max-of/min-of`) in
  USER-GUIDE Forms appendix.
- Wat-rs callsite sweep: ~10 sites across 8 files.

## What this arc did NOT ship

- **Lab-side caller sweep.** Lab is downstream; lab arc 018
  consumes the new primitives + sweeps lab Vec callers in its
  own arc.
- **`unwrap` / `expect` primitive for Option.** Would let
  callers explicitly mark "I know this is safe; panic if not."
  Considered out-of-scope; the match-with-sentinel pattern
  works, and explicit unwrap reintroduces the Haskell-`head`
  wart we just retired.
- **`find` / `find-last` returning the matching element.** Arc
  018 needs the index, not the element. If a future caller
  needs the element, open a small arc.
- **`max-by` / `min-by` (max/min via projection).** Composes
  from `f64::max-of (map xs f)`. Add only if the composition
  becomes painful enough to warrant a dedicated primitive.
- **Tuple positional accessors changing to Option.** Tuple
  arity is type-known; out-of-range is impossible at compile
  time. Keep T for tuples.

## Follow-through

- **Lab arc 018 unblocks.** Resumes with substrate-direct calls
  to all four new primitives, plus a lab-side sweep of any Vec-
  using `first/second/third` callsites.
- **Future numeric / list primitives** open small per-primitive
  arcs citing arc 046's + 047's pattern when a caller needs them.

---

## Commits

- `<wat-rs>` — runtime.rs (dispatch + helpers + 12 tests + 3
  shape-update tests) + check.rs (4 type registrations + Vec
  branch returning Option) + 7 callsite migrations across
  wat/std/stream.wat + wat/holon/Sequential.wat + 5
  tests/*.rs files + wat-tests/std/service/Console.wat +
  USER-GUIDE.md (4 Forms appendix rows) + DESIGN + BACKLOG +
  INSCRIPTION.

---

*these are very good thoughts.*

**PERSEVERARE.**
