# Arc 021 — core/std audit: HashMap, HashSet, get, contains? move to core

**Status:** opened 2026-04-22.
**Motivation:** the rubric `CONVENTIONS.md` names was:
> `:wat::core::*` — evaluator primitives, primitive types,
> primitive-type operations, core collection constructors. Cannot
> be written in wat.
>
> `:wat::std::*` — stdlib built on primitives. Each entry should
> be expressible (in principle) in wat itself.

Four forms drifted into `:wat::std::*` that the rubric says
belong at `:wat::core::*`:

- `:wat::std::HashMap` (constructor) — fundamental collection,
  same tier as `:wat::core::vec` / `list` / `cons`.
- `:wat::std::HashSet` (constructor) — same.
- `:wat::std::get` (accessor) — reaches Rust bucket internals;
  can't write in wat.
- `:wat::std::contains?` (predicate) — same.

Builder framing mid-arc-020 (when adding `:wat::core::assoc`):
> i think we need an audit... std vs core... std::get .... idk....
> what's the rubric for core vs std?... i think everything in std
> needs to be expressed as wat?.. that's the rubric?...

The audit confirmed the user's read. Arc 020 placed `assoc` at
`:wat::core::*` (correct by the rubric, paired with
`:wat::core::conj`). Arc 021 corrects the drift on the other four
forms so the whole family lives together.

Small sweep. Pre-publish — clean rename, no back-compat shims.

---

## UX target

```scheme
;; Before (drift)
(:wat::core::let*
  (((m :rust::std::collections::HashMap<String,i64>)
    (:wat::std::HashMap :(String,i64) "a" 10)))
  (:wat::std::get m "a"))

;; After (post-arc-021)
(:wat::core::let*
  (((m :rust::std::collections::HashMap<String,i64>)
    (:wat::core::HashMap :(String,i64) "a" 10)))
  (:wat::core::get m "a"))
```

Constructor + accessor + predicate + assoc all at `:wat::core::*`.
The HashMap / HashSet / Vec family travels together.

---

## Non-goals

- **Changes to `:wat::std::math::*`.** Transcendentals (ln / log /
  sin / cos / pi) fit the "in principle expressible as Taylor
  series" clause of the std rubric. Stay at std.
- **Changes to `:wat::std::Circular` / `Log` / `Sequential` /
  etc.** Blend-idiom algebra macros — definitionally expressible
  in wat (they ARE wat macros). Stay at std.
- **Changes to `:wat::std::program::Console` / `Cache`,
  `:wat::std::stream::*`, `:wat::std::list::*`.** All wat-source
  implementations over kernel primitives. Stay at std.
- **Renaming `wat::std::math`, `wat::std::stream`, etc. subpaths.**
  Only the four drifted top-level forms move.
- **Retroactive rewrite of arc 001 / 002 / 005 doc history.** Arc
  INSCRIPTIONs record what shipped at the time; the drift was
  real pre-arc-021. Keep the historical record honest.
- **Adding new HashMap ops.** `dissoc` / `keys` / `values` / `len`
  etc. stay deferred per arc 020's non-goals.

---

## What this arc ships

One slice.

### Code renames

- `src/check.rs` — 4 dispatch keys renamed from `:wat::std::*` to
  `:wat::core::*`. Internal fn names (`infer_hashmap_constructor`
  etc.) retain the suffix but the public path changes.
- `src/runtime.rs` — 4 dispatch keys renamed; eval fn names
  updated to match.

### Caller sweep

- `tests/wat_typealias.rs` — existing integration test that uses
  one of the four forms. Update to new path.
- No wat-stdlib (`wat/std/*.wat`) uses the drifted forms — confirmed
  by grep. No baked-stdlib migration needed.
- No example (`examples/with-lru/`, `examples/with-loader/`) uses
  the drifted forms. No consumer sweep needed.
- wat-lru internals — confirmed clean. No migration.
- Trading lab — no use yet. Phase 3.3 (scaled-linear) will use the
  new paths directly when it lands.

### Doc sweep

- `docs/CONVENTIONS.md` — promote the rubric from implicit table
  rows to a named "Core vs stdlib rubric" subsection so future
  contributors can't miss it.
- `docs/USER-GUIDE.md` — update any HashMap examples.
- `docs/arc/2026/04/005-stdlib-naming-audit/INVENTORY.md` —
  correct the inventory entries for the 4 moved forms.
- `docs/arc/2026/04/020-assoc/{DESIGN,INSCRIPTION}.md` — adjust
  the naming-evolution note in arc 020 to reflect that
  `:wat::std::get` (cited as the symmetry target) itself moved in
  the audit.
- `docs/arc/2026/04/021-core-std-audit/INSCRIPTION.md` — closing
  marker.
- `holon-lab-trading/docs/proposals/.../FOUNDATION-CHANGELOG.md`
  — row.

### Historical docs left alone

- Arc 001 / 002 / 005 doc references to the drifted paths. These
  record what shipped at the time; rewriting them would be
  dishonest. The arc 021 record tracks the correction.

---

## Resolved design decisions

- **2026-04-22** — **Four forms move; others stay.** HashMap /
  HashSet / get / contains? fail the "expressible in wat" test;
  everything else under `:wat::std::*` passes it.
- **2026-04-22** — **Clean rename, no shims.** Pre-publish
  discipline.
- **2026-04-22** — **Historical arc docs unchanged.** The drift
  was real; recording it is honest.
- **2026-04-22** — **Promote rubric to a named CONVENTIONS
  subsection.** Implicit table rows weren't discoverable enough —
  arc 020 almost added a 5th drifted form before the builder
  caught it.

---

## What this arc does NOT ship

- Changes to std-math, algebra composition macros, services,
  stream combinators, list combinators.
- New HashMap operations beyond the existing surface + arc 020's
  assoc.
- Retroactive doc rewrites of arc 001 / 002 / 005.
