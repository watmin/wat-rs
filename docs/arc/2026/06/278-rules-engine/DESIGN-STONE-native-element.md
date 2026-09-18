# DESIGN-STONE — nativise `Element` (P11's treatment, applied to the populous one)

> **Origin (builder, 2026-07-31): "nativise Element."** Ruled after the binding premise was measured
> (`03ec041e`) and the measurement re-ordered the plan: the small-vec stone is *blocked* until Element
> stops being a wat `Value`.

## Why this and not the binding representation

The binding-representation stone is licensed by measurement — 100% of binding maps hold **1–3
entries**, no whole-map equality exists, and every iteration site is order-independent (`03ec041e`).
But it **cannot be built yet**: you cannot put an inline small-vec inside a
`Value::wat__core__PersistentMap`, and that is what an Element's bindings currently are.

```rust
fn make_element(fact: Value, bindings: rpds::HashTrieMapSync<Value, Value>) -> Value {
    Value::Aggregate(Arc::new(AggregateValue::record(          // alloc 1 (+ Arc)
        (*element_class_fqdn()).clone(),
        Arc::new(vec![fact, Value::wat__core__PersistentMap(bindings)]),   // alloc 2 (+ Arc)
    )))
}
```

**~3–4 heap allocations per Element, and alpha holds 80,200 of them** at `G=200 W=200`. That is what
`alpha:element` (10.5 ms), `alpha:push` (7.0 ms) and the bulk of `round:drop-memories` (61.0 ms, 34%
of fire) are paying for.

**Token already got this treatment and Element never did.** P11's own comment (`kernel.rs:41`):
*"Replaces the per-token `Value::wat__core__Record` + `VectorSync<Tuple>` allocation chain (~6 allocs
per token) with a single struct."* Element is the one there are 80,200 of.

## The shape — mirror the Token pair exactly

Token is the template, end to end. Element gets the same three pieces:

| Token (exists) | Element (build) |
|---|---|
| `struct Token { matches, bindings }` (`:44`) | `struct Element { fact: Value, bindings: rpds::HashTrieMapSync<Value,Value> }` |
| `native_token_to_value(tok)` (`:205`) | `native_element_to_value(el)` — today's `make_element` body |
| `value_to_token(..) -> Token` (`:188`) | `value_to_element(..) -> Element` |
| `beta_to_pm(HashMap<i64, Vec<Token>>)` (`:261`) | `alpha_to_pm(HashMap<i64, Vec<Element>>)` |

`wm.alpha` becomes `HashMap<i64, Vec<Element>>`. `hashmap_to_pm` **stays** — `production` genuinely
holds derived-fact `Value`s and is not part of this stone.

## ★ THE ONE CONTRACT DECISION

**`element_fact_bindings` keeps its exact signature — it is the narrow waist that makes this mechanical.**

```rust
fn element_fact_bindings(el: &Element) -> (&Value, &rpds::HashTrieMapSync<Value, Value>) {
    (&el.fact, &el.bindings)          // was: destructure a Value record, panic on malformed
}
```

**All 22 consumers then compile unchanged.** Every read of an Element in the kernel already goes
through this one accessor — that is the whole reason this stone is a day's work and not a rewrite. A
rider that "simplifies" by inlining field access at the 22 sites turns a mechanical change into a
22-site review surface for zero gain. Do not.

Two consequences fall out and both are wins: the function stops being able to panic (it currently has
two `panic!` arms for malformed input — a native struct cannot be malformed), and the decode moves to
the one boundary where a malformed Value can actually arrive (`value_to_element`, which returns a
`Result` like `value_to_token` does).

## Blast radius — one file

`src/rete/kernel/` only (both defined in `kernel/session.rs`). `make_element`/`element_fact_bindings` exist nowhere else; `matcher.rs`
never sees an Element (it takes `&rpds::HashTrieMapSync` bindings directly).

Sites that name the element type explicitly and change `Vec<Value>` → `Vec<Element>`:
`:797`/`:837` (hash-join right side), `:1240`/`:1274`/`:1287` (keyed join), `:2061` (`all_right`
rebuild), `:2230`/`:2334` (gather snapshots), the `GatherIndex`/`GatherCache` aliases, and the two
producers `:652`/`:1917`.

## The gate — this is a representation change with NO behaviour change

There is no honest RED here: nothing about the *result* should move. The gate is that the existing
differential surface stays green while the allocation phases fall. What would turn it red:

- **`probe_arc278_2b_insert_alpha`** — reads `Session/alpha-memory` off `fire-once'` and asserts the
  element count AND a binding's value (`?t = 25`). It exercises the **encode path** (Element → Value)
  precisely; a broken `native_element_to_value` fails it. This is the load-bearing gate.
- **`round_trip_fired_session`** — `to_persistent(to_transient(fired)) == fired` exercises **both**
  directions over a populated alpha.
- **`probe_arc278_alpha_is_fire_scoped`** — its `fire-once'` anchor asserts alpha `> 0`.
- **The ~24 count differentials** — any change to what fire derives.

**Measured expectation** (`accum_fire_phase_census`, `G=200 W=200`): `alpha:element`, `alpha:push` and
`round:drop-memories` (61 ms) should all fall. `round:drop-memories` is the big one — dropping a
`Vec<Element>` of plain structs instead of 80,200 `Arc<AggregateValue>` graphs.

**Honest bound:** the bindings map is still an rpds trie after this stone, so its allocation and drop
remain. This removes the *record wrapper* (2 of the ~3–4 allocations), not the trie. The trie goes in
the follow-on, which this unblocks.

## Out of scope = REJECTED (affirmative cuts)

- **The small-vec binding representation.** The next stone, unblocked by this one. Keeping them
  separate keeps this diff mechanical and its measurement attributable.
- **Nativising `production`.** It holds derived-fact `Value`s that genuinely cross to wat as facts.
- **Touching `hashmap_to_pm`.** It still serves production.
- **Inlining the accessor at the 22 call sites.** See the contract decision.
- **`wat/rete.wat`.** The oracle is never optimized and never adjusted to suit the kernel.
