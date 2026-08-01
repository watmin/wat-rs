# BRIEF — nativise `Element`

## The work

Every rete `Element` is currently a wat `Value` record — `Value::Aggregate(Arc::new(...record(..,
Arc::new(vec![fact, Value::wat__core__PersistentMap(bindings)]))))` — about **3–4 heap allocations
each, and alpha holds 80,200 of them**. Make it a plain native struct, exactly the way `Token` already
is (P11). Building the elements and dropping them is currently ~78 ms of a ~180 ms fire.

This is a **representation change with no behaviour change.** Nothing about what fire derives may move.

## Read in order (the rooms, and why you are being sent to each)

1. **`docs/arc/2026/06/278-rules-engine/DESIGN-STONE-native-element.md`** — the measurement, the
   contract decision, the affirmative cuts.

2. **`src/rete/kernel.rs:38-50` — `struct Token`.** THE EXEMPLAR. Read its doc comment: it says
   outright that it replaced a per-token `Value::Record` allocation chain with a single struct. Your
   `Element` is the same move. Note `bindings` stays `rpds::HashTrieMapSync` — do **not** change the
   binding type in this stone (that is the next one, and keeping it out is what makes this diff
   mechanical).

3. **`src/rete/kernel.rs:188-215` — the Token conversion PAIR.** `value_to_token` (decode, returns a
   `Result`) and `native_token_to_value` (encode). You are writing `value_to_element` /
   `native_element_to_value` as exact mirrors.

4. **`src/rete/kernel.rs:261-272` — `beta_to_pm`.** The map-level encoder for a native-struct memory.
   Your `alpha_to_pm` mirrors it. Note `hashmap_to_pm` (`:343`) is NOT this and must stay — production
   still holds `Value`s.

5. **`src/rete/kernel.rs:528-536` — `make_element`.** Its body becomes `native_element_to_value`; the
   function itself becomes a plain struct construction.

6. **`src/rete/kernel.rs:556-567` — `element_fact_bindings`.** The narrow waist. See the contract
   decision — its signature does not change, and that is why 22 call sites need no edit.

7. **`src/rete/kernel.rs:2455-2470`** — where `to_transient` decodes the Session's alpha field, and the
   `to_persistent` side at `:343-360`. These are the two places a Value must still be produced/consumed.

## ★ THE ONE CONTRACT DECISION

**`element_fact_bindings` keeps its exact signature:**

```rust
fn element_fact_bindings(el: &Element) -> (&Value, &rpds::HashTrieMapSync<Value, Value>) {
    (&el.fact, &el.bindings)
}
```

All 22 consumers then compile untouched. **Do not inline field access at the call sites** — it turns a
mechanical change into a 22-site review surface for zero gain. If a call site does not compile after
this, the fix is at that site's *type* (`Vec<Value>` → `Vec<Element>`), never the accessor.

## Implementation sketch

```rust
/// An alpha-memory element: a fact that passed an AlphaNode, with the bindings that match produced.
/// Native for the same reason Token is (P11): the Value-record form cost ~3-4 heap allocations each,
/// and alpha holds tens of thousands. `bindings` stays rpds — matcher.rs reads it directly.
#[derive(Clone)]
pub(crate) struct Element {
    pub(crate) fact:     Value,
    pub(crate) bindings: rpds::HashTrieMapSync<Value, Value>,
}

fn make_element(fact: Value, bindings: …) -> Element { Element { fact, bindings } }
fn native_element_to_value(el: Element) -> Value { /* today's make_element body */ }
fn value_to_element(v: &Value) -> Result<Element, EvalBreak> { /* mirror value_to_token */ }
fn alpha_to_pm(alpha: HashMap<i64, Vec<Element>>) -> Value { /* mirror beta_to_pm */ }
```

Then `wm.alpha: HashMap<i64, Vec<Element>>`, `to_transient` decodes via `value_to_element`,
`to_persistent` encodes via `alpha_to_pm`, and the explicitly-typed element sites become
`Vec<Element>` (the DESIGN lists them; the compiler will name the rest).

## Blast radius

`src/rete/kernel.rs` only. `make_element` and `element_fact_bindings` exist nowhere else, and
`matcher.rs` never sees an Element (it takes `&rpds::HashTrieMapSync` bindings directly).

## STOP triggers (each is a rejection: ship nothing for it, report the gap)

1. **STOP-1** — if `Element` needs to leave `kernel.rs` (any other file must name the type), STOP and
   report. That would mean the boundary is not where this brief assumes.
2. **STOP-2** — do not change the `bindings` field type. It stays `rpds::HashTrieMapSync` in this
   stone. If a small-vec looks tempting, that is the next stone; report and stop.
3. **STOP-3** — if `probe_arc278_2b_insert_alpha` or `round_trip_fired_session` goes red, STOP and
   report the assertion. Those two exercise the encode and round-trip paths; red there means the
   Value boundary is wrong, which is the one thing this stone must not break.
4. **STOP-4** — if any of the ~24 count differentials moves, STOP. The RESULT must not change.

## Definition of done

- `cargo nextest run --release -E 'binary_id(wat::rete)'` — all pass.
- `cargo nextest run --release` — the whole floor (4213 today).
- `cargo clippy --all-targets --release` — silent.
- `cargo nextest run --release -E 'test(accum_fire_phase_census)' --no-capture` — report the
  `alpha:element`, `alpha:push` and `round:drop-memories` rows before/after. `round:drop-memories`
  (61 ms today) is the one to watch.
- Report `git diff --stat`.

Leave the tree dirty and uncommitted. Do not commit, push, or stash — the orchestrator weighs by its
own re-run and commits.

## A prior result to copy for shape

P11 did this exact thing to `Token`. Its struct, its conversion pair, and `beta_to_pm` are all in the
same file — you are adding the second member of a family that already has one.
