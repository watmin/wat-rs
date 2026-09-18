# DESIGN — A3: the constructor applies the discipline to the scalar and withholds it from the pair

> Drawn 2026-09-05 at HEAD `df8a1222e`. Source: vigilia 2026-09-05 A3 (`solvere` L1-3).
> Every line verified on disk at THIS HEAD, and the consequence chain traced end to end.

## The site argues the cure, then does not apply it

`CompiledCond` holds two arrays the docs call one thing:

- `slot_keys` (`compiled_cond.rs:159`) — *"Parallel to `output_slots`."*
- `output_slots` (`:165`) — *"`output_slots[i]` is the scratch-slot index whose value pairs with
  `slot_keys[i]` — **the two arrays together are the zip the design doc describes.**"*

`from_parts`' own doc (`:212-218`) explains why it exists rather than a struct literal:

> `has_seed_cmp` **must agree with the ops it summarises**, and a caller computing it by hand could
> get it wrong in the direction that is invisible… **Deriving it from the ops makes the two unable
> to disagree.**

**And in the same function it takes `slot_keys` and `output_slots` as two independent `Arc<[…]>` and
checks nothing.** The discipline is applied to the scalar and withheld from the pair beside it, with
the reason written out directly above.

## Two writers; one checks by hand

| writer | shape |
|---|---|
| `compiled_cond.rs:374-383` | builds both from ONE `order` vec — safe by construction, checks nothing |
| `export.rs:1357-1364` | the WIRE import: builds them from **two independently-parsed sequences** (`items[3]`, `items[4]`) |

`export.rs:1398-1408` then hand-checks `slot_keys.len() != output_slots.len()` — **immediately before
calling `from_parts`, the constructor that could enforce it for both.** That is the convention rung
this arc has now ruled against four times.

## The consequence: a rule silently stops matching

Traced this session:

1. `materialize_into:1094` iterates `output_slots`;
2. `:1105-1108` — `if i >= compiled.slot_keys.len() { pool.truncate(off); return None; }`
3. `exec_compiled_with_key_ids:933` — `if !exec_ops(…) { return None }` … so **`None` MEANS "the
   condition did not match"**;
4. a length mismatch therefore produces the **identical `None`**. The rule stops matching. No error,
   no diagnostic, no census.

Second consequence, same root: `intern_cond_keys:—` sizes `ids` as `fact_bind? + slot_keys.len()`
while `materialize_into` consumes one id per **`output_slots`** entry.

## ★ The tell: the same function guards one impossible condition and not the other

`materialize_into` has two "cannot happen" arms:

```rust
None => { debug_assert!(false, "compiled program guarantee violated: output slot {slot} unbound…");
          pool.truncate(off); return None; }          // SCREAMS in debug
…
if i >= compiled.slot_keys.len() { pool.truncate(off); return None; }   // SILENT
```

One is asserted, the other is not, and the silent one is the one a wire import can actually reach.

## The one contract decision, pinned

**`from_parts` takes the ZIP and splits it internally.** A `Vec<(Value, usize)>` — or a `SlotZip`
newtype with a private constructor — so two sequences of different lengths have no form. Then the
wire importer's hand-check stops being a post-hoc comparison and becomes a **parse into the zip**:
it must interleave the two parsed sequences, and a length mismatch fails there, once, with the
malformed error it already produces.

**The precedent is 250 lines from the defect, in the same file as the wire writer:**
`ClassIntern::intern` (`export.rs:1671-1679`) pushes to `names`, `fields` and `idx` **in one act**.

**And then the silent guard is provably dead.** Convert `:1105-1108` to a `debug_assert!` matching
its sibling — do not simply delete it. A guard that can no longer fire still records what it
prevented, and the sibling arm is the house form for exactly that.

## Scope

**IN:** the zip, both writers, the wire parse, the guard conversion, the proof. Floor GREEN.

**OUT, affirmatively cut:** `intern_cond_keys`'s sizing (it follows from the zip and needs no
separate change — say so in the SCORE rather than touching it); the other 12 `pack_`/`unpack_` pairs
(`solvere` L2-3, a real row and a different one); D2p, F2, A4.
