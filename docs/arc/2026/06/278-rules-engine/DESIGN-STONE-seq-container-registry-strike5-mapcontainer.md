# DESIGN — strike 5: the MapContainer registry (the keyed-collection sibling waist)

**Status:** STRIKE-READY draft (contract decisions pinned below; awaiting go before sonnet).
**Arc:** 278 · narrow-waist registry · strike 5 of {4 ✅ depth, 5 MapContainer, 6 route mixed ops both waists}.
**Why now:** the dependency strike 6 needs — `get`/`contains?`/`length`/`empty?` route their MAP arms through a
registry that doesn't exist yet. Build it standalone (route the map-only op `assoc`), then strike 6 routes the
mixed ops through both waists in one pass.

## Why (the guarantee, the map half)

R14 / builder: *the next primitive we introduce isn't allowed a partial/wrong impl.* Strike 4 closed this for the
**seq** family. The **map** family — `HashMap`, `PersistentMap` — is still hand-rolled per-op, per-side
(`assoc`, `get`, `contains?`, `length`, `empty?` each list `Value::wat__std__HashMap(..) => …,
Value::wat__core__PersistentMap(..) => …` independently). A new keyed primitive (a `BTreeMap`, an ordered map,
the persistent-map that started all this) can be added half-wrong with no compile error — the same O(ops)×2-sides
drift the seq registry killed. MapContainer is the sibling waist that makes it compile-forced.

### Recon evidence (to run in the strike)

The map ops have no central classifier today: a throwaway `Value::ProbeMap` variant would error only at the
`Value` enum's own derives (PartialEq/Hash) — **not** at any of the 5 map-op dispatches (they all carry
`_ => …TypeMismatch` / explicit arms over `Value`). Post-strike (assoc routed): adding a `MapContainer`
variant errors at `assoc`'s dispatch — the forcing the registry installs. (Strike 6 extends it to the other 4.)

## What ships (strike 5)

1. **`src/collection/map_container.rs`** — the keyed sibling of `seq_container.rs`:
   ```rust
   pub(crate) enum MapContainer { HashMap, PersistentMap, Record }   // keyed collections (Record = ordered tagged-map)
   impl MapContainer {
       fn of_value(v: &Value) -> Option<MapContainer>                // pure: HashMap / PersistentMap / Record(both variants)
       fn of_type(reduced: &TypeExpr, types: &TypeEnv) -> Option<MapContainer> // HashMap/PersistentMap Parametric; Record via is_subtype
       // capability table (current truth):
       fn can_assoc(self) -> bool   // HashMap ✓ PersistentMap ✓ Record ✓
       fn keyed_lookup(self) -> bool// HashMap ✓ PersistentMap ✓ Record ✗ (○gap: get-by-keyword not built — strike 6+)
       fn has_key(self) -> bool     // HashMap ✓ PersistentMap ✓ Record ✗ (○gap)
       fn measurable(self) -> bool  // HashMap ✓ PersistentMap ✓ Record ✗ (○gap)
       // `Record` is also ORDERED (declaration order; struct_form is a Vec) — a real property with no op
       // consumer yet; promoted to an `ordered()` capability when keys/vals/seq-over-pairs is built.
   }
   ```
2. **Route `assoc` through it, both sides, Form 1** (strike-4 pattern): `eval_assoc` runtime dispatch becomes
   `match MapContainer::of_value(coll) { Some(m) if m.can_assoc() => match m { HashMap => …, PersistentMap => …,
   Record => <field-update body> } }`; `infer_assoc` checker classification goes through `of_type` (+ `can_assoc`).
   Behavior byte-identical. Strike 5 USES `can_assoc`; `keyed_lookup`/`has_key`/`measurable` are defined now
   (current truth) and consumed in strike 6 — the same staged fill SeqContainer did across strikes 1-3.
3. No `get`/`contains?`/`length`/`empty?` changes — that's strike 6.

## CONTRACT DECISION 1 — MapContainer = `{HashMap, PersistentMap, Record}` (Record IS a member)

*(Reversed 2026-06-20 after the builder challenged an earlier "exclude Record" draft. The reversal is grounded;
the original draft conflated Record's internal Rust repr with its semantic family.)*

Record IS a keyed collection — grounded on disk:
- **It is a map on the wire:** `edn_shim.rs:2157` decodes a *"base-record tagged-map"* → `Value::wat__Record`.
  Records serialize as tagged maps; Clojure treats records as maps. The "struct, not a map" framing was wrong.
- **It is ordered:** `wat__Record.struct_form: Arc<Vec<Value>>` = *"Ordered field values in declaration order."*
  An ordered keyed collection — mapping its pairs yields declaration order.
- The `struct_form` Vec is Record's **internal repr**, exactly like `Vec`/`LinkedList`/`VectorSync` are the inner
  reprs of Seq members — handled by a **named arm** (`record_assoc` vs `hashmap_assoc_inner`), NOT a reason to
  split the family. Excluding it would conflate inner-storage difference with family difference (the audit's R4
  warning, inverted).

So `assoc` routes ALL its members — HashMap, PersistentMap, Record — through **one** `MapContainer` dispatch; no
separate "record family" arm. Record's heterogeneity (assoc ✓ today; get/contains/length ○gap; ordered) lives in
the **capability table** (Decision 2) — which is exactly why the table earns its place from day one.

The cost, accepted honestly: `of_type` takes `&TypeEnv` (Record is classified by `is_subtype(p, ":wat::Record")
|| is_subtype(p, ":wat::holon::Record")`, covering user records). This diverges from `SeqContainer::of_type(&TypeExpr)`
— but the divergence is driven by a **real difference** (records have a subtype lattice; seq members are matched
by head/structure), so taking the lattice is honest, not ceremony. `of_value` stays pure.

Four-questions (hard constraint = no partial impl + honest modelling + the evolutionary waist): **Obvious?** YES
(ordered tagged-map → keyed registry; Clojure records-are-maps). **Simple?** YES at the system level (one keyed
family, one registry; the `&TypeEnv` is one honest, necessary complication). **Honest?** YES (models Record as
what it is on the wire; inner-repr handled by a named arm; capability table records its true profile). **Good
UX?** YES (Record's future ops compile-forced by the registry — evolution engineered *inside* the waist).
Excluding Record fails *Honest* (inner-repr ≠ family) and the evolutionary goal (Record's growth left outside
the waist). A future `RecordContainer` is NOT needed — Record sits in MapContainer.

## CONTRACT DECISION 2 — build the capability table (mirror `SeqContainer`)

*(Reversed 2026-06-20 — the builder caught that omitting it "breaks symmetry because we haven't found a counter
case yet." Including Record (Decision 1) IS the counter case: its profile differs from the maps today.)*

The capability table is the **evolutionary slot** — the uniform place a new keyed member declares its profile —
and it is the narrow waist's whole point (a new primitive slots in low-friction, forced complete). Omitting it
would make MapContainer a *different shape* from `SeqContainer`, so the first heterogeneous member would force a
restructure-or-style-divergence instead of a clean variant-add. And it is **not** all-true ceremony: with Record
in the family, the table differentiates immediately —

| member          | can_assoc | keyed_lookup (get) | has_key | measurable | (ordered) |
|-----------------|-----------|--------------------|---------|------------|-----------|
| HashMap         | ✓         | ✓                  | ✓       | ✓          | ✗         |
| PersistentMap   | ✓         | ✓                  | ✓       | ✓          | ✗         |
| Record          | ✓         | ○ gap              | ○ gap   | ○ gap      | ✓         |

`○ gap` = fillable, not yet built (get-by-keyword / contains-field / field-count on a Record — a future op).
`ordered` has no op consumer yet → documented on the variant, promoted to a method when keys/vals/seq-over-pairs
lands. The table encodes **current truth**, exactly as `SeqContainer.mappable` did (`○ gap` for List/WatAstList).

Four-questions: **Obvious?** YES (a sibling registry IS the same pattern). **Simple?** YES at the system level
(one uniform mechanism, not two shapes). **Honest?** YES (the table records a real, current differentiation —
Record vs the maps; documented `○ gap`/`ordered`). **Good UX?** YES (the next heterogeneous member slots in like
a seq container — no restructure).

## Dispatch shape (Form 1 + capability gate, mirrors strike 4)

```rust
// eval_assoc, runtime — behavior byte-identical to today
match MapContainer::of_value(&arg0_val) {
    Some(m) if m.can_assoc() => match m {                    // exhaustive over the closed enum, no `_`
        MapContainer::HashMap       => hashmap_assoc_inner(&arg0_val, &arg1_val, &arg2_val),
        MapContainer::PersistentMap => persistentmap_assoc_inner(&arg0_val, &arg1_val, &arg2_val),
        MapContainer::Record        => /* existing record field-update body (wat__Record + wat__holon__Record) */,
    },
    Some(_) => Err(type_mismatch(OP, &arg0_val)),            // can_assoc()==false (none today; the slot)
    None    => Err(type_mismatch(OP, &arg0_val)),            // not a keyed collection
}
```
All `assoc` members route through ONE `MapContainer` dispatch — Record is in the registry, not a side arm. The
Record arm keeps its exact field-update body (named arm preserves the field-update-vs-K→V-insert divergence).

## Disconfirming evidence + permanent guard

- **Disconfirming (recon):** add a throwaway `MapContainer` variant → after strike, `cargo build` errors at
  `eval_assoc`'s `match m` **and** the 4 capability methods (proof the dispatch + table are compiler-forced).
  Remove the variant; confirm green.
- **Permanent guard:** (1) the compiler (exhaustive `match map_container` + exhaustive capability methods);
  (2) a `tests/probe_map_container.rs` reachability test — every `MapContainer` variant (incl. `Record`, both
  `wat__Record` + `wat__holon__Record`) produced by `of_value`, and `assoc` round-trips on each;
  (3) a checker≡runtime parity assertion for assoc's accepted set; (4) floors 941/36.

## Out of scope (affirmative cut)

- `get`/`contains?`/`length`/`empty?` routing → **strike 6** (mixed ops, both waists, one pass). Record's
  `keyed_lookup`/`has_key`/`measurable` are `○ gap` (false) until a get-by-keyword/field-count op is built — a
  future capability fill, not this strike.
- No `RecordContainer` registry — Record sits in `MapContainer` (it's an ordered tagged-map; Decision 1).
- No behavior change to `assoc` (no live drift — checker ≡ runtime, verified). No `get`/etc. routing this strike.
