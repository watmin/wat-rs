# DESIGN — Stone P1: native working-memory + the transient/freeze boundary

The foundation of the Rust fire kernel (arc 278's closing condition). A native, mutable `WorkingMemory` that
the kernel will mutate during fire, plus the lossless boundary that converts a frozen `Session` value into it
(`to_transient`) and back (`to_persistent`). Mutation lives only inside this native rep, sealed in Rust —
**out of the user's hands** (the user calls `fire`, never the transient). No firing yet (P2); no keyed joins
(P3); no delta (P4). P1 ships the seam and proves it is lossless.

## Why
Clara's model (`CLARA-REF §5`): mutate a transient during fire, `to_persistent!` at the end → a new immutable
session. The wat oracle has no transient — every memory op is an O(log n) persistent `assoc` (the benched
O(N²) tree). The kernel needs O(1) native mutation during fire; this stone is the mutable rep + the freeze
boundary it rests on. Everything in P2–P5 mutates a `WorkingMemory` and freezes it back.

## What P1 delivers (Rust, internal — `src/rete/kernel/`, new)

A `WorkingMemory` struct — the native mirror of a `:wat::rete::Session`:
```rust
pub(crate) struct WorkingMemory {
    network:    Value,                       // passthrough (immutable input: id → Node)
    rules:      Value,                       // passthrough (immutable input)
    alpha:      HashMap<i64, Vec<Value>>,    // mutable mirror of alpha-memory  (node-id → [Element])
    beta:       HashMap<i64, Vec<Value>>,    // mutable mirror of beta-memory   (node-id → [Token])
    production: HashMap<i64, Vec<Value>>,    // mutable mirror of production-mem (node-id → [Record])
    facts:      Value,                       // passthrough (the asserted fact PV)
    next_id:    i64,
}
```
The three memories are the hot, mutated-during-fire maps (native `HashMap` = O(1) `entry().or_default().push`).
`network`/`rules`/`facts`/`next_id` are inputs the fire phase reads but does not restructure — held as-is.

- **`to_transient(session: &Value) -> Result<WorkingMemory, EvalBreak>`** — read the `Session` `wat__Record`'s
  `struct_form` (7 fields in declaration order: network, rules, alpha-memory, beta-memory, production-memory,
  facts, next-id; TypeMismatch if the value isn't a `:wat::rete::Session` record). Convert each memory
  `PersistentMap` (`HashTrieMapSync<Value,Value>`, key `Value::i64`, value `Value::wat__core__PersistentVector`)
  into a `HashMap<i64, Vec<Value>>`. Passthrough the rest.
- **`to_persistent(wm: WorkingMemory) -> Value`** — rebuild each memory `PersistentMap` from its `HashMap`
  (`Value::i64(k)` → `Value::wat__core__PersistentVector(VectorSync from Vec)`), then rebuild the `Session`
  `wat__Record` with `struct_form` in declaration order. (Empty memory map → empty `PersistentMap`.)

## The one contract decision (pinned)
**Round-trip identity: `to_persistent(to_transient(s)) == s`** for every compiled / fired `Session`. The
boundary is lossless — the differential test in P2 (Rust fire-once == wat fire-once) rests on this. Outer-map
ordering is content-equality (hash maps); per-node `Vec` order is preserved end to end (PV→Vec→PV).

## Verification — an in-crate round-trip unit test (NOT a wat probe)
The converters are internal Rust (the transient mutation is sealed; exposing `to_transient`/`to_persistent` to
wat would either be useless without exposing mutation, or put mutation in the user's surface — neither is
wanted). So P1 is tested by a `#[cfg(test)]` unit test in `src/rete/kernel/tests/`:
1. `startup_from_source` a world (Temperature/WindSpeed/ColdAndWindy + the cold-and-windy rule).
2. Build a fired `Session` via the **oracle** (`collect-rules`/`compile`/`insert`/`fire-rules`) through
   `eval_in_frozen` — a session with populated alpha/beta/production memories + facts.
3. `let wm = to_transient(&fired)?; let back = to_persistent(wm);`
4. `assert_eq!(back, fired)` — round-trip identity. Also round-trip a freshly-`compile`d (empty-memory)
   session → identity (the empty case).

RED before the converters exist (won't compile). GREEN when the seam is lossless.

## Files touched
- `src/rete/kernel/session.rs` (new) — `WorkingMemory` + `to_transient` + `to_persistent` + the round-trip unit test.
- `src/rete/mod.rs` — `pub(crate) mod kernel;` + a note.

## Out of scope = REJECTED
- **Firing on the WorkingMemory** — P2 (Rust fire-once, differential-tested vs the oracle).
- **Keyed joins** (`join-bindings` map keying) — P3 (the O(N²)→O(match) win; P1's `Vec` mirror is the flat
  form, refined there).
- **Delta propagation / TM cascade** — P4. **Wat surface for `fire` / the bench** — P5.
- No wat-exposed transient. No mutation primitive in the language. No change to the oracle or any prior stone.
