# Arc 073 — Term store: HolonAST as Prolog term, Thermometer as tuning curve

**Status:** PROPOSED 2026-04-27. Pre-implementation reasoning artifact.

**Predecessors:** arc 023 (`coincident?`), arc 037 (per-d encoders + sigma machinery), arc 057 (typed HolonAST leaves), arc 058 (`HashMap<HolonAST, V>`), arc 067 (flat default dim router with `DEFAULT_TIERS = [10000]`).

**Surfaced by:** holon-lab-trading umbrella 059 slice 1's L1/L2 cache, after the proof session (proof 018) shipped a flat-fuzzy `Vec<(HolonAST, V)>` reference impl. Reviewing it pre-implementation surfaced that the all-fuzzy `coincident?`-over-whole-form approach lets template-divergent forms accidentally cosine-match when their fuzzy leaves happen to align — a class of false positive the substrate already has the information to eliminate.

Builder direction (2026-04-27, mid-cache-slice review):

> "we only do fuzzy lookups if the surface form has fuzzy terms... we can know what parts actually bear measurement... the surface is a template"

> "yes — just like prolog — do you see it?"

> "this is more than cache — this is a holon query interface"

> "each thermometer /is/ a coordinate — can have at most 100 peers in 10k dims — holy shit"

> "did we just model neurons into the system?"

The recognition (load-bearing): **HolonAST is a Prolog term; Thermometer is a logic variable; coincident-floor is a constraint predicate. And — duals over the same structure — the template is a neural cell type; the Thermometer is a tuning curve; sqrt(d) is the population's resolution.** The substrate (VSA / HDC) was designed by people thinking about cognition; this isn't a metaphor we're projecting onto the algebra. It's what the algebra has always been.

Arc 073 makes that explicit.

---

## What this arc is, and is not

**Is:** a substrate primitive that exposes term decomposition (template + slots) for any HolonAST, plus a `TermStore<V>` parametric data structure with `put` / `get` / `len` whose lookup is structural-unification + slot-tolerance. The lab cache slice 1 (umbrella 059) is the immediate consumer; future query / recall / population-code systems land on the same primitive.

**Is not:** a query language. No range queries, no pattern unification beyond exact-template + tolerance-slots, no set operations. Those are conceivable extensions; this arc is the foundation they'd compose from.

**Is not:** a redesign of the algebra grid. The substrate's existing types (HolonAST + Thermometer + sigma + dim router) carry every piece of information this arc needs. The arc shapes that information into a callable surface.

---

## The core decomposition

A HolonAST decomposes into `(template, slots)`:

- **Template** — the AST with each `Thermometer { value, min, max }` leaf replaced by a *slot marker* that retains `(min, max)` (the receptive field) and discards `value` (the tuning point).
- **Slots** — the `value` fields plucked from each Thermometer leaf, in pre-order traversal sequence.

Two thoughts have the same template iff they're structurally identical except at Thermometer-leaf positions, and at every Thermometer position they have matching `(min, max)`. Different `value` doesn't break template equality; that's what slots are for.

Why Thermometer specifically: it's the only HolonAST leaf variant that's locality-preserving (tuning curve). All other leaves — `Atom`, `Symbol`, `String`, `I64`, `Bool`, `F64` — are exact-identity (template). `F64` is quasi-orthogonal at the encoder; two close F64s produce uncorrelated vectors, so they're not "near" each other in any useful sense and belong to different templates. The substrate's leaf-type taxonomy IS the slot/template distinction.

---

## The store

```
TermStore<V> :: HashMap<Template, Vec<(Slots, V)>>
```

- Keyed by template (exact `HashMap` lookup; arc 058 makes `HashMap<HolonAST, _>` honest).
- Each bucket is a population of cells sharing a cell type (template), each tuned to a specific point in the slot space.

### `put(form, v)`
1. `(template, slots) = decompose(form)`.
2. `buckets[template].push((slots, v))`.
3. If `buckets[template].len() > sqrt(d)` (the population's resolution at the encoder's d), evict the oldest (FIFO). Beyond sqrt(d) the receptive fields overlap; storing more entries than the algebra grid can resolve is dishonest.

### `get(form) -> Option<V>`
1. `(query_template, query_slots) = decompose(form)`.
2. `bucket = buckets[query_template]`. If empty, return None — the cell type doesn't exist in the population.
3. For each `(stored_slots, v)` in bucket: check that all `i: |query_slots[i] - stored_slots[i]| / range[i] < sigma_floor`. First match wins. (The substrate's `coincident?` reduces to this exact check when the templates match — using `coincident?` directly over the whole form is mathematically equivalent in this case, but doing it per-slot makes the dependency explicit and avoids the encode-and-cosine cost.)
4. No match → None. The stimulus didn't fire any cell in the population.

Forms with **zero Thermometer slots** decompose to `(template, [])`. The bucket has at most one entry; the slot-tolerance check reduces to "true if the template existed." `TermStore<V>` with non-Thermometer forms degenerates cleanly to an exact `HashMap<HolonAST, V>`. Same primitive, both populations.

---

## What this gives the consumer for free

The lab cache slice (umbrella 059 slice 1) becomes:

```
TermCache (next-form direction)     :: TermStore<HolonAST>
TermCache (terminal-value direction):: TermStore<HolonAST>
EncodeCache                          :: TermStore<wat::holon::Vector>
```

Three caches; one primitive. Lookup semantics are identical across all three. The cache slice doesn't write decomposition logic, doesn't re-derive tolerance, doesn't choose between exact and fuzzy — the substrate exposes one query function and the consumer composes.

Future query / recall / population-code consumers (lab umbrella 059 phase 2's thought iteration, the trader's reckoner, the engram library, MTG / truth-engine domains) reach for the same `TermStore<V>` and the substrate's invariants propagate.

**No user-facing tolerance knob.** The slot's `(min, max)` is part of the form (the consumer who built the Thermometer chose them). The d is decided by the ambient router. The sigma is the ambient sigma function. Nothing for `TermStore` callers to configure beyond `V` and an optional cap override. *Hidden assumed behavior.*

---

## Surface

### Substrate primitives (Rust + wat wrappers)

| Op | Signature | What it does |
|----|-----------|--------------|
| `:wat::holon::term::template` | `(form :HolonAST) -> :HolonAST` | Returns the form with each Thermometer-leaf's `value` replaced by a sentinel (the substrate-internal slot marker — see § slot-marker shape below). Identical structure otherwise. |
| `:wat::holon::term::slots` | `(form :HolonAST) -> :Vec<f64>` | Pre-order vector of Thermometer `value` fields. Empty for forms with no Thermometer leaves. |
| `:wat::holon::term::ranges` | `(form :HolonAST) -> :Vec<(f64,f64)>` | Pre-order vector of Thermometer `(min, max)` pairs, parallel to `slots`. Used by `TermStore::get`'s slot-tolerance check. |
| `:wat::holon::term::matches?` | `(query :HolonAST) (stored :HolonAST) -> :bool` | Convenience predicate: same template AND all corresponding slots within `sigma_floor` of each other. Composes the three above primitives + the substrate's sigma machinery. |

### `TermStore<V>` (registered built-in struct, parametric)

```scheme
(:wat::core::struct :wat::holon::TermStore<V>
  ;; Internal shape — opaque to consumers, who reach through
  ;; the put/get/len primitives.
  (buckets :HashMap<wat::holon::HolonAST, Vec<(Vec<f64>, V)>>)
  (capacity-per-bucket :i64))   ;; default sqrt(d) at instantiation

(:wat::holon::TermStore::new<V>
  (cap :Option<i64>) -> :wat::holon::TermStore<V>)

(:wat::holon::TermStore::put<V>
  (store :wat::holon::TermStore<V>)
  (form :HolonAST)
  (value :V)
  -> :wat::holon::TermStore<V>)            ;; values-up — returns new store

(:wat::holon::TermStore::get<V>
  (store :wat::holon::TermStore<V>)
  (form :HolonAST)
  -> :Option<V>)

(:wat::holon::TermStore::len<V>
  (store :wat::holon::TermStore<V>)
  -> :i64)                                  ;; total entries across all buckets
```

`TermStore` is a value-up immutable structure (returns new store on `put`); the lab cache slice that needs thread-owned mutable state composes it inside a `LocalCache<Template, …>` or service program of its own choosing — *not* this arc's concern.

### Slot-marker shape (substrate-internal)

The template's slot marker must:
1. Be a HolonAST (so `:wat::holon::term::template` returns one).
2. Carry the slot's `(min, max)` (so two templates with different ranges don't collide).
3. NOT carry the slot's `value` (so two thoughts with same range and different values share a template).
4. Be distinguishable from any user-constructed leaf (so user forms can't spoof it).

Recommendation: a new HolonAST variant `SlotMarker { min: f64, max: f64 }`, or — to avoid growing the algebra — a `Bundle([Symbol(":wat::holon::slot"), F64(min), F64(max)])` shape that's recognizable structurally. The substrate-internal marker IS NOT user-constructible (the `:wat::holon::slot` keyword is reserved); user code that tried to forge one would get rejected at the type checker layer.

**Decision Q1 below:** new variant vs reserved-keyword Bundle.

---

## Decisions to resolve

### Q1 — Slot marker as new HolonAST variant or reserved-keyword Bundle?

**Option a — new variant.** Add `HolonAST::SlotMarker { min: f64, max: f64 }`. Pro: structurally distinguishable from user content; no spoofing risk; type checker can reject user-constructed `SlotMarker`. Con: grows the closed algebra (arc 057 named the closure as load-bearing); 12-variant HolonAST instead of 11. Every site that pattern-matches HolonAST grows an arm.

**Option b — reserved-keyword Bundle.** `Bundle([Symbol(":wat::holon::slot"), F64(min), F64(max)])`. Pro: no algebra growth; uses existing variants. Con: relies on the type checker / parser blocking user code from constructing the reserved keyword (the prefix-reservation mechanism already exists for `:wat::*`); a slightly larger blast radius for "what does this Bundle mean."

**Recommendation: Option a.** The slot marker IS a new kind of leaf — semantically it's a logic variable, structurally it's a placeholder. Adding it to the algebra honors that. Arc 057's "closed under itself" claim isn't violated because the algebra still composes (Bundles of slot markers compose just like Bundles of anything else); the closure adds a leaf, doesn't break the recursion. The type checker can refuse user-constructed `SlotMarker` (the literal syntax for it never gets exposed; users only see slot markers via `:wat::holon::term::template` output).

### Q2 — Eviction policy when bucket exceeds sqrt(d)?

**Recommendation: FIFO (drop the oldest entry).** Move-to-front-on-hit can land in a future arc if profiling demands. FIFO is the cheapest invariant that respects sqrt(d) without imposing access-pattern semantics consumers may not want.

### Q3 — Cross-tier `<` (multi-tier dim router) behavior?

When the ambient router has multiple tiers, different forms encode at different d. A `TermStore<V>` instantiated against the d=10000 tier has cap 100; a different store against d=1024 has cap 32. **Recommendation: `TermStore::new` reads the router at construction, picks the d for the typical form shape (or the consumer specifies), caps from there.** Future arc revisits if multi-tier consumers actually surface; arc 067 simplified to flat single-tier so this is a non-issue today.

### Q4 — Per-Thermometer tolerance override?

A consumer might want to widen tolerance on a specific slot ("RSI within 5%, not 0.5%"). **Recommendation: defer.** Slice-1 ships with substrate-derived `sigma/sqrt(d)` everywhere. Per-slot overrides land in a future arc if Phase-2 thought iteration surfaces a consumer who needs them. The `:wat::config::set-coincident-sigma!` knob (arc 024) already exists for global overrides; that's the layer where calibration happens today.

### Q5 — Should `term::matches?` also handle pre-encoded Vector pairs?

`coincident?` (arc 023, polymorphic since arc 061) handles `(HolonAST, Vector)` mixes. `matches?` operates at the term level; Vectors don't have templates. **Recommendation: HolonAST/HolonAST only for `matches?`. Pre-encoded Vector callers use `coincident?` directly — they've already crossed the encoding boundary and don't have term structure to unify.**

---

## What this arc deliberately does NOT do

- **Range queries / pattern unification beyond exact-template-+-tolerance-slots.** The substrate's primitives are sufficient for the cache use case and for any consumer whose query shape is "give me the value associated with this form." Querying for "all forms whose RSI slot is between 50 and 70" would compose differently — each template-bucket scanned with a per-slot range predicate. Future arc when a consumer needs it.
- **Set operations on TermStores.** Union / intersection / difference are conceivable; not in scope.
- **The "engram library" / SimHash bucketing for sub-linear lookup.** Already named on paper (BOOK Ch.55); deferred.
- **Mutable-state TermStore.** This arc ships values-up (`put` returns a new store). The lab cache slice composes a thread-owned mutable cell around it if it needs that.

---

## What this unblocks

- **Lab umbrella 059 slice 1's L1/L2 cache** — directly. The flat-fuzzy `Vec<(HolonAST, V)>` proof 018 prototyped becomes `TermStore<HolonAST>` (for the dual coordinate caches) + `TermStore<Vector>` (for EncodeCache). One primitive, three caches.
- **Phase-2 thought iteration** — Thermometer slots ARE the substrate's tuning surface. Consumers reaching for "I want to retrieve thoughts close to this thought" land on `TermStore::get` directly.
- **Future engram library / reckoner integration** — the population-code framing makes the engram's per-bucket exemplars natural: each bucket's `Vec<(Slots, V)>` IS an exemplar set for that cell type.
- **Cross-domain reuse** — MTG / truth-engine domains compile to the same primitive without re-deriving the cache architecture.

---

## Test strategy

Reuse proof 018's tests verbatim; they become the substrate-tier regression suite for `TermStore`. Specifically:
- **T0/T0b/T0c** — already-terminal handling (arcs 070+071+072 made this honest; T0 hits trader-shape thoughts as AlreadyTerminal).
- **T1** — `TermStore::put` then `TermStore::get` round-trips a stored entry.
- **T2** — byte-identical query hits (degenerate fuzzy = exact at sigma=1).
- **T3** — near-equivalent template-matching thought hits via slot tolerance. **The load-bearing test.**
- **T4** — distant template-matching thought misses (slot delta exceeds tolerance).
- **T5** — fills to sqrt(d) entries without neighborhood interference; FIFO eviction past cap.
- **T6** — different templates produce different buckets (no spurious cross-template hits — the proof 018 flat-fuzzy approach couldn't guarantee this; arc 073's structural keying does).

Add new substrate-tier tests:
- **W1** — `term::template` round-trip: same template extracted from two thoughts that share structure-with-different-slot-values.
- **W2** — `term::slots` parallel to `term::ranges`: same length; values pluck from Thermometer leaves in pre-order.
- **W3** — non-Thermometer-bearing forms: empty slots, single-entry bucket, exact lookup degenerates correctly.
- **W4** — spoofing rejection: user-constructed slot-marker syntax (option a) gets rejected at type check / parse.

---

## The thread

- **Arc 023** — `coincident?`. The substrate's "are these the same point" predicate. Foundation.
- **Arc 037** — per-d encoders + sigma machinery. The receptive-field-width / tuning-curve-width that this arc surfaces as "tolerance."
- **Arc 057** — typed HolonAST leaves, algebra closed under itself. The leaf-type taxonomy this arc reads to find slots.
- **Arc 058** — `HashMap<HolonAST, V>` at user level. The structural index this arc keys on.
- **Arc 069** — `coincident-explain`. The diagnostic surface a consumer reaches for when a `TermStore::get` returns None unexpectedly.
- **Arc 070-072** — walker + parametric type plumbing + lexer fixes. The substrate consumption infrastructure.
- **2026-04-27 (mid-lab-cache build)** — the recognition: the surface is a template; Thermometer is a coordinate; this is Prolog; this is neurons.
- **Arc 073 (this)** — the population-code primitive made explicit.
- **Next** — lab umbrella 059 slice 1 consumes `TermStore<V>` directly; cache becomes a 3-line wrapper rather than a from-scratch fuzzy datastructure.

PERSEVERARE.
