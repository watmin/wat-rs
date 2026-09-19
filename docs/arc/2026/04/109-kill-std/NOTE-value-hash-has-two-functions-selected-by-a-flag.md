# NOTE — `Hash for Value` has two functions, selected by a stamped flag

**Found 2026-09-06** while grounding arc 278's A4 (`seen_insert`'s split dedup set). **NOT worked
there** — that session is rete-only, and this lives in `src/value/`. Parked here, where this tree
collects substrate findings that are surfaced but not worked.

## The shape

`src/value/value.rs:848-857`, the `Aggregate` arm of `Hash for Value`:

```rust
Value::Aggregate(a) => {
    if a.identity != 0 {
        a.identity.hash(state);       // the construction fingerprint
    } else {
        a.nature.hash(state);
        a.class.hash(state);
        a.fields.hash(state);         // the EDN data
    }
}
```

**One type, two hash functions, selected by a flag on the value.** They do not agree: `identity` is
itself `FxHasher(nature, class, fields)` finished, so the two branches feed the caller's hasher
completely different bytes for the same logical content.

`Hash`/`Eq` requires equal values to hash equally. That holds today **only because the branch is a
function of the content**, and it is a function of the content **only because there is exactly one
stamping site.**

## Why it is safe right now

`AggregateValue::from_parts` (`:1064-1083`) is the sole place `identity` is assigned, and all four
constructors — `struct_`, `record`, `record_arc`, `holon_record` — funnel through it:

```rust
let identity = if fields.iter().all(value_is_shallow) {
    …FxHasher over (nature, class, fields)…
    if id == 0 { 1 } else { id }      // sentinel guard: a real hash of 0 must not read "unstamped"
} else { 0 };
```

Equal aggregates therefore have equal fields, take the same `all(value_is_shallow)` decision, and
land in the same `Hash` branch. **No violation exists at HEAD.** The `id == 0` guard shows the
obvious sentinel collision was already thought about.

## Why it is worth writing down anyway

**A fifth constructor that sets `identity` any other way — or skips `from_parts` — breaks
`Hash`/`Eq` for `Value`.** Not loudly: two equal values would land in different hash buckets, so
`HashSet`/`HashMap` would hold duplicates and lookups would miss. The blast radius is every hashed
container in the substrate, and the symptom is a silent wrong answer.

The invariant is real, load-bearing, and currently protected by *"there happens to be one
constructor."* Nothing states it at `from_parts`, and nothing enforces it.

## A second, smaller thing in the same neighbourhood

`value_is_shallow` (`:1031-1047`) decides shallowness for most variants structurally, but for an
Aggregate it returns `a.identity != 0` — **it reports shallowness by reading the stamp**, while the
stamp is assigned by testing shallowness of the fields. Self-consistent and, for nested aggregates,
the only cheap answer — but it means "shallow" and "stamped" are one fact wearing two names, and a
reader meeting `value_is_shallow` first will not see that.

## Shape of a cure (not prescribed)

The honest options are a doc that states the invariant at `from_parts`, or a shape that enforces
it — `identity` private with assignment only through `from_parts`, so a future constructor cannot
set it another way. This note deliberately does not choose; whoever owns `src/value/` should.

## Who else depends on this

`src/rete/kernel/fire/delta.rs`'s `seen_insert` splits the fixpoint's dedup set on the same flag
(`identity() != 0`), so **rete's termination correctness currently rests on this invariant** — a
property of a file rete does not own. Arc 278 is curing rete's side to be robust against a
violation rather than to depend on its absence; that work does not fix this note.
