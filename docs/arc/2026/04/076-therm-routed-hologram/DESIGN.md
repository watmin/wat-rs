# Arc 076 — Therm-routed Hologram + filtered-argmax as the unifying primitive

**Status:** PROPOSED 2026-04-28. Pre-implementation reasoning artifact.

**Predecessors:**
- Arc 074 — `Hologram<V>` + `HologramLRU<V>`. Established the cell-bucketed coordinate-cell store with cosine readout and a construction-time filter func. This arc supersedes the explicit `pos` parameter pattern.
- Arc 057 — typed HolonAST leaves. Form-structure inspection at substrate level (this arc does therm-form predicate against the leaf shape).
- Arc 067 — flat default dim router (`DEFAULT_TIERS = [10000]`).
- holon-rs scalar encoders — thermometer encoding's two-slot bleed pattern (`$log` / `$linear` / `$circular`). This arc surfaces the same bleed structure at the cache layer.

**Surfaced by:** Lab umbrella 059 slice 1 mid-build (2026-04-28). After 5 incremental Service tests proved request/reply, the recognition surfaced: prolog `?slot` was scaffolding; what we actually want is for the form structure itself to dictate the slot. From the conversation:

> "the form is `(thermometer :some-float 0 :dim-capacity)` ... the only dynamic part of the user is the :some-float"
>
> "the user must normalize the float between 0 and the dim-capacity"
>
> "is this what argmax is meant to do the whole time... we need filtered-argmax?"

Two recognitions, one arc:

1. **The slot is in the form, not in a separate parameter.** Therm-shaped forms self-describe their slot via `floor(value)`; non-therm forms always slot 0. The substrate inspects the form on every put / get and routes accordingly. Caller never passes a `pos`.
2. **Filtered-argmax was always the primitive.** Plain argmax is the degenerate "filter always accepts" case. The substrate's lookup is filtered-argmax over the gathered candidate vec. Slot-routing produces the vec; filtered-argmax discriminates within it.

---

## What this arc is, and is not

**Is:** a refinement of arc 074's `Hologram<V>` and `HologramLRU<V>`:
- `pos` parameter eliminated from `put` / `get`. Slot is derived from the form.
- New substrate helper `:wat::holon::therm-normalize` — capacity-aware float normalizer. Caller's domain bounds in, in-range float out. Makes "value out of bounds" a category error at the construction site, not a runtime concern.
- New substrate helper `:wat::holon::therm-form` — convenience constructor that calls `therm-normalize` and assembles `(thermometer normalized-float 0 dim-capacity)`. Eliminates hand-rolled forms.
- Filtered-argmax surfaces as the substrate's unified lookup primitive — the existing get path renamed and reframed; plain argmax disappears as a separate concept.

**Is not:**
- A new store type. Hologram and HologramLRU stay; their internals shift.
- A change to the parametric `<V>` story. Same generic; same composition rules.
- A change to filter-func conventions (`presence-filter`, `coincidence-filter`). Those stay; what changes is when they fire (always, including the trivial-one-candidate path).
- An eviction-policy change. HologramLRU's two-bound LRU stays.

---

## The shift

### Before (arc 074)

```scheme
(:wat::holon::Hologram/put store pos key val)      ; caller computes pos in [0, 100]
(:wat::holon::Hologram/get store pos probe filter) ; bleed-pair = (floor pos, ceil pos)
```

The caller computes `pos` from the form (cosine readout against a reference, SimHash, domain projection). The substrate doesn't pick a default. Different consumers compute differently. Putting and getting must agree on the pos function.

### After (arc 076)

```scheme
(:wat::holon::Hologram/put store key val)      ; substrate inspects key, routes
(:wat::holon::Hologram/get store probe filter) ; substrate inspects probe, routes
```

`pos` is gone. The substrate looks at the input form's shape:
- If the form is `(thermometer :float 0 :dim-capacity)` — extract the float, slot = `floor(float)`. Bleed-pair is `(floor, floor+1)` clamped to `[0, dim-capacity-1]`.
- Otherwise — slot = 0. No bleed (single-slot lookup).

Capacity is fixed at construction (`:wat::holon::Hologram/make capacity`). The form's `:dim-capacity` arg must match the store's capacity; a substrate check at put / get time makes the mismatch a `RuntimeError::InvalidArgument`.

### Why this is a real simplification

- **One source of truth.** The form encodes its own coordinate. Putting and getting can never disagree because both inspect the same form.
- **OOB collapses to a category error.** With `therm-normalize` as the only path, malformed therm values can't enter the system. A bad bound is caught at the construction site, not at lookup time.
- **Slot isn't lab-specific.** Cosine readout, SimHash, domain projection — all of those needed each consumer to invent a pos function. Therm-routing covers the case the actual consumers (caches, engrams, expansion chains) all share: continuous-valued forms with bracketing semantics.
- **Filtered-argmax names the primitive honestly.** "Get the candidate vec; filter; argmax" is what every consumer wanted; the substrate stops pretending plain argmax was a separate thing.

---

## The model

### Storage

```rust
Hologram<V> {
  capacity: usize,                              // user's pick at construction
  slots:    Vec<HashMap<HolonAST, V>>,          // length == capacity
  filter:   FilterFn,                           // bound at construction
}
```

Each slot is a HashMap keyed by the full form (including the float for therm). Keys never collide across distinct floats — slot 42 holds `(therm 42.42 ...)`, `(therm 42.50 ...)`, `(therm 42.99 ...)` as three separate hashmap entries.

`HologramLRU<V>` composes Hologram the same way arc 074 establishes — adds `lru: LruCache<HolonAST, SlotIdx>` for the global-cap eviction trigger plus per-slot-cap fallback.

### Slot derivation

```
slot_for(form):
  if therm-form?(form):
    let f = extract-float(form)
    return clamp(floor(f), 0, capacity - 1)
  else:
    return 0
```

`therm-form?` matches the substrate-canonical leaf shape `(thermometer :float 0 :dim-capacity)`. Recognition criteria:
- Three-element AST — head leaf `:thermometer`, then float, then `0`, then `:dim-capacity` (or its int equivalent).
- The fourth arg matches the store's capacity. Mismatch → `RuntimeError::InvalidArgument`.
- Hand-rolled forms that don't match the canonical shape are non-therm by definition; they go to slot 0. (Caller's choice — they get the non-therm fast path even if conceptually they wanted bracketing.)

### put

```
put(form, val):
  s = slot_for(form)
  slots[s].insert(form, val)
  ;; HologramLRU only:
  ;;   lru.push(form, s)
  ;;   if slots[s].len() > per_slot_cap: scan and drop the oldest by LRU rank
```

Idempotent on existing key (overwrite is fine). Returns `:()`.

### get (filtered-argmax)

```
get(probe, filter):
  if therm-form?(probe):
    f      = extract-float(probe)
    floor_ = clamp(floor(f), 0, capacity - 1)
    ceil_  = clamp(ceil(f),  0, capacity - 1)
    candidates = slots[floor_].entries() ++ slots[ceil_].entries()  ; dedup if floor_ == ceil_
  else:
    candidates = slots[0].entries()

  return filtered_argmax(candidates, probe, filter)

filtered_argmax(candidates, probe, filter):
  probe_vec = encode(probe)
  best = None
  for (key, val) in candidates:
    cos = cosine(encode(key), probe_vec)
    if filter(cos):                ; predicate over cosine
      if best is None or cos > best.cos:
        best = (cos, val)
  return best.map(|b| b.val)
```

Notes:
- For non-therm with at most one candidate (the typical exact-form-keyed case), the filter still runs. If the input is identical to the stored key, cosine = 1.0; standard filters accept. If the filter rejects (high threshold), the answer is None — even with one candidate. **The filter is not a no-op, ever.**
- For therm in the interior, the bleed-pair produces a union of candidates; argmax across both slots picks the closest by cosine. Edge values (slot 0 alone or slot capacity-1 alone) work the same way with one slot's contents.
- **Self-cosine is always 1.0** (encode is deterministic for the same form). A get of a previously put form hits 1.0 and passes any reasonable filter.

### Filter function

Same shape arc 074 ships: `:fn(:f64) -> :bool`. The substrate provides:
- `(:wat::holon::Hologram::presence-filter capacity)` — looser. `λ cos. cos > presence_floor(capacity)`.
- `(:wat::holon::Hologram::coincidence-filter capacity)` — stricter. `λ cos. (1 - cos) < coincident_floor(capacity)`.

Filters are now parameterized by `capacity` (the store's cell count) since `d` is an indirect proxy. For backward compatibility the substrate maps capacity to the d that produces it (`d = capacity²` under `floor(sqrt(d))` cell budgeting), so the filter math stays identical to arc 074's.

User-supplied filters compose as before.

---

## The therm normalizer (mandatory entry point)

```scheme
(:wat::holon::therm-normalize
  (low :f64)        ; user's domain low bound
  (high :f64)       ; user's domain high bound
  (value :f64)      ; the value to map
  (capacity :i64)   ; the target Hologram's capacity
  -> :f64)          ; normalized into [0, capacity]
```

Behavior:
- `clamp(value, low, high)` first — out-of-range domain values clamp silently. The contract is "tell us your bounds; we'll respect them." A value above `high` saturates at `high`, mapping to slot `capacity - 1`.
- Linear remap to `[0, capacity]` — `(value - low) / (high - low) * capacity`.
- The result fits the form `(thermometer normalized 0 capacity)` cleanly.

The convenience constructor:

```scheme
(:wat::holon::therm-form
  (low :f64)
  (high :f64)
  (value :f64)
  (capacity :i64)
  -> :wat::holon::HolonAST)
;; returns (thermometer (therm-normalize low high value capacity) 0 capacity)
```

Users who hand-roll `(thermometer ...)` without going through `therm-form` can produce floats outside `[0, capacity]`. The Hologram's slot-derivation clamps defensively, so misuse degrades to "your float landed at the boundary." But the canonical entry is `therm-form`.

---

## API summary

### Renamed / reshaped surfaces

| Before (arc 074) | After (arc 076) |
|---|---|
| `Hologram/make` | `Hologram/make capacity filter` (capacity is now a substrate parameter, not derived from d) |
| `Hologram/put store pos key val` | `Hologram/put store key val` |
| `Hologram/get store pos probe filter` | `Hologram/get store probe` (filter bound at construction; arity drops) |
| `Hologram/coincident-get` / `present-get` | Same shape, `pos` removed; or **deprecated** if `Hologram/get` with construction-time filter subsumes them. Decide in slice 1. |
| n/a | `:wat::holon::therm-normalize`, `:wat::holon::therm-form` |
| `presence-filter d` / `coincidence-filter d` | `presence-filter capacity` / `coincidence-filter capacity` |

### Unchanged

- `Hologram/len` — same.
- HologramLRU's eviction policy — same two-bound LRU.
- Enum-wrapping composition pattern (`AnyStore<V>`).

---

## Slice plan

### Slice 1 — `Hologram<V>` substrate refactor

Touches:
- `wat-rs/src/runtime.rs` — therm-form predicate, slot extraction, bleed-pair gather, filtered-argmax dispatch. `eval_hologram_make`, `eval_hologram_put`, `eval_hologram_get` reshape; `pos`-taking variants removed.
- `wat-rs/src/check.rs` — type schemes for new arities.
- `wat-rs/wat/holon/Hologram.wat` — top-level signatures match new arities; `coincident-get` / `present-get` either rewire to construction-time filter or drop.
- `wat-rs/wat/holon/Filter.wat` — `presence-filter` / `coincidence-filter` factories take `capacity` instead of `d`.
- `wat-rs/wat-tests/holon/Hologram.wat` — full sweep. New tests: therm-form put/get, edge-slot routing, bleed-pair lookup, OOB float clamping, non-therm fast path, filter-rejects-single-candidate path.

### Slice 2 — `therm-normalize` + `therm-form`

Touches:
- `wat-rs/src/runtime.rs` — two new dispatch entries (pure functions; no state).
- `wat-rs/wat/holon/Hologram.wat` — wat-side wrappers if needed (likely just runtime entries).
- `wat-rs/wat-tests/holon/therm-normalize.wat` — tests across asymmetric domains, OOB clamping, capacity boundary correctness.

### Slice 3 — `HologramLRU<V>` ride-along

Touches:
- `wat-rs/crates/wat-hologram-lru/wat/holon/HologramLRU.wat` — composition adapts to the new Hologram signatures. Same shape, signatures forward.
- `wat-rs/crates/wat-hologram-lru/wat-tests/holon/HologramLRU.wat` — test sweep mirrors slice 1's.

### Slice 4 — Lab call-site sweep

Touches:
- `holon-lab-trading/wat/cache/L1.wat` — drop `pos` from put/get sites; lookups become `:trading::cache::L1/lookup l1 form` (no pos).
- `holon-lab-trading/wat/cache/walker.wat` — visitor's recorded forms drop the pos arg; walker signature drops pos throughout.
- `holon-lab-trading/wat/cache/Service.wat` — Request enum's Get / Put variants drop pos field; the embedded reply-tx pattern stays identical.
- Lab tests update to match.

---

## Test strategy

### Slice 1 (Hologram)

- **T1** — `Hologram/make 100 (presence-filter 100)`; put `(thermometer 42.5 0 100)` → val. get same form → val. (Round-trip at slot 42, exact cosine = 1.0.)
- **T2** — Bleed-pair pickup: put `(thermometer 42.0 0 100)` → val_42; put `(thermometer 43.0 0 100)` → val_43; get `(thermometer 42.7 0 100)` → val_43 (closer by cosine).
- **T3** — Edge slot 0: put `(thermometer 0.42 0 100)` → val; get `(thermometer 0.42 0 100)` → val. (No bleed past slot 0.)
- **T4** — Edge slot capacity-1: same at the top.
- **T5** — Non-therm form: put `(price :open 50000)` → val; get same → val. (Slot 0 hashmap exact lookup.)
- **T6** — Filter rejection with single candidate: stricter filter; non-therm get returns None even though one entry exists when cosine threshold isn't met. (Forces a synthetically distorted probe; relies on cosine being slightly < 1 for a different-but-similar form.)
- **T7** — Capacity mismatch: store `cap=100`; put `(thermometer 50.0 0 64)` → `RuntimeError::InvalidArgument` (the form's capacity arg disagrees with the store's).
- **T8** — `Hologram/len` after multi-slot puts.

### Slice 2 (normalizer)

- **T9** — `therm-normalize 0 100 50 100` → `50.0`.
- **T10** — `therm-normalize 0 100 -10 100` → `0.0` (clamps low).
- **T11** — `therm-normalize 0 100 110 100` → `100.0` (clamps high).
- **T12** — Asymmetric: `therm-normalize 200 600 400 100` → `50.0` (linear remap).
- **T13** — `therm-form` round-trip: out is a `(thermometer ... 0 100)` AST whose float arg matches `therm-normalize`'s output.

### Slice 3 (HologramLRU)

- Re-runs slice 1's T1-T8 against HologramLRU.
- **T14** — Global-LRU eviction at `global_cap`. Drops the LRU-oldest key from its slot.
- **T15** — Per-slot eviction at `per_slot_cap`. Drops the slot's oldest-by-LRU-rank entry when one slot fills before global.

### Slice 4 (lab integration)

- All existing lab tests stay green after the call-site sweep. No new probe tests required (the substrate change is mechanical from the lab's perspective).

---

## What this arc deliberately does NOT do

- **Add a new sub-bucketing layer for slot-0 pile-up.** If many distinct non-therm forms accumulate at slot 0 and lookup gets slow, that's a follow-up arc (likely SimHash or hash-of-form-shape secondary indexing). Slice 1 ships linear scan inside slot 0; we measure before optimizing.
- **Change the bleed direction policy.** Bleed-pair is `(floor, ceil)` strictly. No three-slot bleed, no slot-of-slot recursion.
- **Provide non-therm coordinate functions.** Cosine readout, SimHash, domain projection — those are consumer-side. The substrate covers therm-routing because therm forms are universal across consumers (continuous values are everywhere); the rest is consumer territory.
- **Rename `Hologram` itself.** Name stays. The semantic shift is internal.

---

## What this unblocks

- **Lab umbrella 059 slice 1 simplification.** L1.wat / walker.wat / Service.wat all drop their pos plumbing. The lab's coordinate function (currently fixed at 50.0 in the walker) becomes obsolete; the form structure carries the slot.
- **Engram libraries with continuous-valued indexers.** Engrams keyed by therm-encoded latitude / time-of-day / price-zone slide into Hologram naturally.
- **Cross-domain consumers** (MTG, truth-engine) that have continuous attributes (mana cost, confidence score) get the slot-routing for free.
- **The "out-of-bounds is a category error" stance becomes substrate policy.** Other primitives can adopt the same discipline (capacity-aware constructors, no runtime OOB).

---

## Open questions

### Q1 — Do `coincident-get` and `present-get` survive?

Currently they're separate entry points with built-in filter selection. If `Hologram/make` takes a filter at construction, the two-name distinction collapses — every `Hologram/get` is filtered-by-its-construction-filter. Slice 1 picks one of:

- (a) Drop both; one `get` with one construction-time filter.
- (b) Keep both as construction-time filter overrides — useful when one Hologram needs two filtering modes (rare).

Default position: (a). Decide in slice 1.

### Q2 — Capacity = dim-capacity = sqrt(d)?

Arc 074 chose `num_cells = floor(sqrt(d))` to match the algebra grid's resolution. Arc 076's `capacity` is user-chosen at construction — orthogonal to d. Q: does the substrate enforce `capacity == floor(sqrt(d))` for any reason, or is it free?

Default: **free**. Capacity is a hologram-level decision. The substrate's d is encoder-level. They can differ — Hologram doesn't depend on d for slot math anymore.

### Q3 — Therm-form recognition strictness

What counts as a therm-form? Strictest: `(:thermometer :float :int :int)` exactly. Looser: any 4-element AST with `:thermometer` head. Loosest: any leaf-tagged form named `:thermometer` (variant tags from elsewhere also accepted).

Default: **strict**. Slot 0 fallback for everything else. A consumer who wants slot routing on a non-thermometer form opens a follow-up arc.

### Q4 — Capacity validation at put/get

Form's capacity arg must match store's capacity. Mismatch is `RuntimeError::InvalidArgument`. But is this always — or only when `:strict` is on? Default: **always**. The arc's "OOB is a category error" stance demands it.

---

## Risks

- **Slot-0 hot-pile-up.** If a workload puts thousands of distinct non-therm forms, slot 0's HashMap grows large and filtered-argmax does a linear cosine scan. Mitigated by: most realistic workloads either use therm-forms heavily (cache key spreads across slots) or have small non-therm vocabularies. If profiling shows pile-up, slice-N adds secondary bucketing.
- **Capacity choice.** Wrong capacity (too small → too many collisions; too big → many empty slots → no real difference but memory). Mitigated by: documentation on choosing capacity; the trader's slice-1 picks 100 and that's a reasonable default for most workloads.
- **Therm-form misidentification.** A form that looks like a therm-form but isn't (head `:thermometer` but with weird structure) routes badly. Mitigated by: strict shape check (Q3) — any deviation from canonical shape goes to slot 0.

---

## Slice 1 acceptance

- T1-T8 green.
- Existing arc 074 tests adapted to new signatures all green.
- One commit at the slice boundary; lab integration follows in slice 4.

PERSEVERARE.
