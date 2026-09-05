# BRIEF — A3: make the zip a zip

Cure **and** prove it. **Floor GREEN when you are done.**

## Read in order

1. **`DESIGN.md`** — the contract is the ZIP at the constructor, and the guard becomes a
   `debug_assert!` rather than a deletion.
2. **`src/rete/compiled_cond.rs:155-168`** — the two fields and the docs calling them one thing.
3. **`compiled_cond.rs:212-240`** — `from_parts`, and **read its doc before you touch it**: it argues
   the exact discipline it withholds from the pair.
4. **`compiled_cond.rs:372-386`** — the safe writer (one `order` vec).
5. **`src/rete/export.rs:1357-1364`** (two independent parses) and **`:1398-1408`** (the hand-check).
6. **`compiled_cond.rs:1094-1116`** — `materialize_into`, both guards. Note the asymmetry.
7. **`src/rete/export.rs:1671-1679`** — `ClassIntern::intern`, the cured shape already in this tree.

## Implementation sketch

```rust
// compiled_cond.rs — the pair has one form
pub(crate) struct SlotZip { pairs: Arc<[(Value, usize)]> }   // private
impl SlotZip {
    pub(crate) fn from_pairs(pairs: Vec<(Value, usize)>) -> Self { … }
    pub(crate) fn len(&self) -> usize; 
    pub(crate) fn key(&self, i: usize) -> &Value;
    pub(crate) fn slot(&self, i: usize) -> usize;
}
```

`from_parts` takes `SlotZip`. The safe writer builds it from `order` directly (it already has the
pairs). The wire importer **interleaves** its two parsed sequences into pairs and fails there on a
length mismatch, keeping the malformed error it already emits.

Keep `slot_keys()` / `output_slots()` accessors if the packer needs them — `pack_compiled_cond`
(`export.rs:1322-1324`) reads both — but they must be **derived views, not stored arrays**.

## The proof

**A compiler error**, the shape this arc has produced four times: constructing a `CompiledCond`
with mismatched sequences must not be writable. Quote what the compiler says.

If the type still permits `SlotZip::from_pairs(vec![])` alongside a longer op list, say what remains
checkable and what does not — do not claim more than the shape gives.

## Blast radius

`src/rete/compiled_cond.rs` + `src/rete/export.rs`. No wat corpus change.

## STOP triggers

1. **If the wire ABI would change, STOP.** `pack_compiled_cond` writes `slot_keys` and
   `output_slots` as two sequences at indices 3 and 4. The zip is an in-memory shape; **the wire
   stays two sequences** unless you surface the ABI question first.
2. **If a `*_cost` gate moves, STOP and report.** `materialize_into` is on the production hot path
   (`accum_cost.rs:1251` benches it directly). An `Arc<[(Value, usize)]>` changes locality versus two
   arrays; if that shows up, say so rather than adjusting a gate.
3. **If you find yourself deleting the `i >= slot_keys.len()` guard rather than converting it,
   STOP** — see the DESIGN.
4. **On any RED: DO NOT RE-RUN.** Capture whole, name the arm, surface it.

## Prior result to copy for shape

`../strike-class-plan-door/` — private state, one door, a derived predicate rather than a stored
one, proof is a compiler error, and the hot-path constraint respected and measured.
